import {
    type NodeModel,
    type ConnectionModel,
    type FlowProject,
    type ProjectIdentifier
} from "~/repository/modules/projects";
import {FlowrsNode} from "~/rete/flowrs/flowrsNode";
import {NodeEditor} from "rete";
import {type ItemDefinition} from "rete-context-menu-plugin/_types/presets/classic/types";
import {useProjectsStore} from "~/store/projectStore";
import {ContextMenuPlugin, Presets as ContextMenuPresets} from "rete-context-menu-plugin";
import type {Schemes} from "~/rete/flowrs/editor";
import {Connection} from "~/rete/flowrs/editor";
import type {Type} from "~/repository/modules/packages";
import {usePackagesStore} from "~/store/packageStore.js";
import {navigateTo} from "#app";

/**
 * Class responsible for creating, saving, and manipulating the state of a Rete.js flow editor.
 */
export class ContextCreator {
    private static editor: NodeEditor<Schemes> | undefined;

    private static nodeNameCount: Map<string, number> = new Map<string, number>();

    private static constructableNodeList: ItemDefinition<Schemes>[];

    /**
     * Saves the current state of the Rete.js editor as a new project in the system.
     * @throws Error if something goes wrong
     */
    public static async saveBuilderStateAsProject() {
        const packagesStore = usePackagesStore();
        if (!this.editor) {
            throw new Error("Editor is undefined!");
        }
        let selectedProject = this.getCurrentlySelectedProject();
        if (!selectedProject) {
            throw new Error("No project selected!");
        }

        let flowProject: FlowProject = JSON.parse(JSON.stringify(selectedProject));
        console.log("Selected Project", flowProject);

        let allNodes = this.editor.getNodes();
        console.log(allNodes);
        let allConnections = this.editor.getConnections();
        let typeDefinitionsMap = await useNuxtApp().$api.packages.getFlowrsTypeDefinitionsMap(packagesStore.currentActive);

        this.setNodesAndDataInProject(flowProject, allNodes, typeDefinitionsMap);
        this.setPackagesInProject(flowProject);
        this.setConnectionsInProject(flowProject, allConnections);

        let original_name = flowProject.name;

        // try creating the new setup and clean it up on success
        flowProject.name = "tmp_" + flowProject.name;
        try {
            await useNuxtApp().$api.projects.createProject(flowProject);
        } catch (e) {
            console.error("Error occurred on save", e);
            throw new Error("Save was unsuccessful 🐛 Please check your configuration 🔧");
        }
        try {
            await useNuxtApp().$api.projects.deleteProject({project_name: flowProject.name});
        } catch (e) {
            console.log("Delete failed", e);
        }

        // delete the old & create the new setup, knowing that it will succeed
        flowProject.name = original_name;
        try {
            await useNuxtApp().$api.projects.deleteProject({project_name: flowProject.name});
        } catch (e) {
            console.log("Delete failed", e);
        }
        console.log("New Project", flowProject);
        try {
            await useNuxtApp().$api.projects.createProject(flowProject);
        } catch (e) {
            console.error("Error occurred on save", e);
            throw new Error("Save was unsuccessful 🐛 Please check your configuration 🔧");
        }
    }

    /**
     * Sets the packages in the given flow project based on the active packages in the store.
     * @param flowProject - The flow project to update with packages.
     */
    private static async setPackagesInProject(flowProject: FlowProject) {
        const packagesStore = usePackagesStore();

        const packages: string[] = [];
        const toRemovePackages: string[] = [];
        flowProject.packages.forEach(element => {
            packages.push(element.name)
        });

        console.log(packagesStore.packages);
        packagesStore.currentActive.forEach((element) => {
            if (element != "flowrs" && element != "primitives" && !packages.includes(element)) {
                const foundObject = toRaw(packagesStore.packages).find((obj) => obj.name === element);
                if (foundObject) {
                    let flowPackage = {
                        name: foundObject.name,
                        version: foundObject.version,
                        path: undefined,
                        git: undefined,
                        branch: undefined,
                    };
                    flowProject.packages.push(flowPackage)
                }
            }
        });
        packages.forEach(element => {
            if (!packagesStore.currentActive.includes(element)) {
                toRemovePackages.push(element);
            }
        });
        toRemovePackages.forEach(element => {
            flowProject.packages = flowProject.packages.filter(obj => obj.name !== element);
        });
    }

    /**
     * Sets the nodes and data in the given flow project based on the Rete.js editor's nodes.
     * @param flowProject - The flow project to update with nodes and data.
     * @param allNodes - Array of all nodes in the Rete.js editor.
     * @param typeDefinitionsMap - Map of type definitions for nodes.
     */
    private static setNodesAndDataInProject(flowProject: FlowProject, allNodes: Schemes["Node"][], typeDefinitionsMap: Map<string, Type>) {
        flowProject.flow.nodes = {};
        flowProject.flow.data = {};
        for (const node of allNodes) {
            this.addNodeToProject(typeDefinitionsMap, node, flowProject);

            this.addDataToProject(node, flowProject);
        }
    }

    /**
     * Adds a node to the given flow project based on a Rete.js node.
     * @param typeDefinitionsMap - Map of type definitions for nodes.
     * @param node - The Rete.js node to add to the project.
     * @param flowProject - The flow project to update with the added node.
     */
    private static addNodeToProject(typeDefinitionsMap: Map<string, Type>, node: FlowrsNode, flowProject: FlowProject) {
        let typeDefinitionOfCurrentNode = typeDefinitionsMap.get(node.fullTypeName);
        if (!typeDefinitionOfCurrentNode) {
            throw new Error(`Node ${node.label} is currently not in the package list`);
        }

        let flowNode: NodeModel = {
            node_type: "",
            type_parameters: {},
            constructor: "",
        };
        flowNode.node_type = node.fullTypeName;
        flowNode.constructor = node.constructor_type;

        if (typeDefinitionOfCurrentNode.type_parameters && typeDefinitionOfCurrentNode.type_parameters.length != node.typeParameters.size) {
            throw new Error(`Not all TypeParameters are set for node ${node.label}`);
        }
        // Convert Map to object
        const convertedTypeParameters: { [key: string]: string } = {};
        node.typeParameters.forEach((value, key) => {
            convertedTypeParameters[key] = value;
        });
        flowNode.type_parameters = convertedTypeParameters;

        flowProject.flow.nodes[node.label] = flowNode;
    }

    /**
     * Adds data to the given flow project based on a Rete.js node.
     * @param node - The Rete.js node to add data from.
     * @param flowProject - The flow project to update with the added data.
     */
    private static addDataToProject(node: FlowrsNode, flowProject: FlowProject) {
        if (node.node_data) {
            try {
                let parsedJson = JSON.parse(node.node_data);
                flowProject.flow.data[node.label] = {value: parsedJson};
            } catch (e) {
                throw new Error(`Check the input field of node ${node.label}. The input field could not be parsed as JSON`);
            }
        }
    }

    /**
     * Sets the connections in the given flow project based on the Rete.js editor's connections.
     * @param flowProject - The flow project to update with connections.
     * @param allConnections - Array of all connections in the Rete.js editor.
     */
    private static setConnectionsInProject(flowProject: FlowProject, allConnections: Schemes["Connection"][]) {
        if (!this.editor) {
            throw new Error("Editor is undefined!");
        }
        flowProject.flow.connections = [];
        for (const connection of allConnections) {
            let flowConnection: ConnectionModel = {
                from_node: "",
                from_output: "",
                to_node: "",
                to_input: "",
            };

            flowConnection.from_output = connection.sourceOutput;
            flowConnection.to_input = connection.targetInput;
            flowConnection.from_node = this.editor.getNode(connection.source).label;
            flowConnection.to_node = this.editor.getNode(connection.target).label;

            if (!flowConnection.from_node || !flowConnection.to_node) {
                throw new Error("Some node of this connection couldn't be resolved:\n" + flowConnection);
            }

            flowProject.flow.connections.push(flowConnection);
        }
    }


    /**
     * Adds Flowrs elements to the Rete.js editor and returns a context menu with the constructable nodes.
     * @param editor - The Rete.js editor to add elements to.
     * @returns A Promise of the context menu with the constructable nodes.
     */
    public static async addFlowrsElements(editor: NodeEditor<Schemes>) {
        const packagesStore = usePackagesStore();

        this.editor = editor;
        this.nodeNameCount = new Map<string, number>();
        const selectedProject = this.getCurrentlySelectedProject();

        // get (typename,typeDefinition) Map
        let typeDefinitionsMap = await useNuxtApp().$api.packages.getFlowrsTypeDefinitionsMap(packagesStore.currentActive);

        // construct scene out of selected project
        let projectNodesMap = await this.addProjectNodes(selectedProject, typeDefinitionsMap, editor);
        await this.connectProjectNodes(selectedProject, projectNodesMap, editor);

        this.preventTypeIncompatibleConnections(editor);
        return await this.createContextMenuWithConstructableNodes(typeDefinitionsMap);
    }

    /**
     * Updates the context menu based on the current active packages.
     */
    public static async updateContextMenu() {
        const packagesStore = usePackagesStore();

        let typeDefinitionsMap = await useNuxtApp().$api.packages.getFlowrsTypeDefinitionsMap(packagesStore.currentActive);
        this.constructableNodeList = await this.getConstructableNodeList(typeDefinitionsMap);
    }

    /**
     * Retrieves the currently selected project from the projects store.
     * If no project is selected we navigate to the frontpage.
     * @returns The currently selected FlowProject.
     */
    private static getCurrentlySelectedProject(): FlowProject  {
        const projectsStore = useProjectsStore();
        const selectedProjectUnwrapped = computed(() => projectsStore.selectedProject);
        const selectedProject: FlowProject | null = selectedProjectUnwrapped.value;
        if (!selectedProject || !projectsStore) {
            navigateTo("/");
        }
        return selectedProject!;
    }

    /**
     * Adds nodes to the Rete.js editor based on the nodes in the given project.
     * @param project - The project to extract nodes from.
     * @param typeDefinitionsMap - Map of type definitions for nodes.
     * @param editor - The Rete.js editor to add nodes to.
     * @returns A map of added nodes with node names as keys.
     */
    private static async addProjectNodes(project: FlowProject, typeDefinitionsMap: Map<string, Type>, editor: NodeEditor<Schemes>) {
        let allAddedNodes: Map<string, FlowrsNode> = new Map();

        for (let flowNodeName in project.flow.nodes) {
            let currentNode = project.flow.nodes[flowNodeName];
            let currentNodeType = currentNode.node_type;
            let typeDefinition = typeDefinitionsMap.get(currentNodeType);
            if (!typeDefinition) {
                console.error("TypeDefinition for", currentNodeType, "not found");
                continue;
            }

            let nodeNameWithoutBadChars = flowNodeName.replaceAll("::", "_");
            let countOfType = this.nodeNameCount.get(nodeNameWithoutBadChars);
            const node = new FlowrsNode(nodeNameWithoutBadChars + (countOfType || ""), currentNodeType, project.flow.data[flowNodeName]?.value, currentNode.constructor, currentNode.type_parameters, typeDefinitionsMap, editor);

            this.nodeNameCount.set(nodeNameWithoutBadChars, (countOfType || 0) + 1);

            await editor.addNode(node);
            allAddedNodes.set(flowNodeName, node);
            console.log("node added", flowNodeName, node);
        }

        return allAddedNodes;
    }

    /**
     * Connects nodes in the Rete.js editor based on the connections in the given project.
     * @param project - The project to extract connections from.
     * @param projectNodes - The map of added nodes in the editor.
     * @param editor - The Rete.js editor to add connections to.
     */
    private static async connectProjectNodes(project: FlowProject, projectNodes: Map<string, FlowrsNode>, editor: NodeEditor<Schemes>) {
        for (let flowConnection of project.flow.connections) {
            let source = projectNodes.get(flowConnection.from_node);
            let target = projectNodes.get(flowConnection.to_node);

            if (!source || !target) {
                console.error(source, target, "not found in", projectNodes);
                return;
            }

            let newConnection = new Connection(source, flowConnection.from_output, target, flowConnection.to_input);

            await editor.addConnection(newConnection);
            console.log("connection added", newConnection);
        }
    }

    /**
     * Retrieves a list of constructable nodes for the context menu based on type definitions.
     * @param typeDefinitionsMap - Map of type definitions for nodes.
     * @returns A Promise of an array of constructable nodes for the context menu.
     */
    private static async getConstructableNodeList(typeDefinitionsMap: Map<string, Type>): Promise<ItemDefinition<Schemes>[]> {
        let output: ItemDefinition<Schemes>[] = [];
        for (const fullTypeName of typeDefinitionsMap.keys()) {
            let typeDefinition = typeDefinitionsMap.get(fullTypeName);
            console.log("Package", fullTypeName, typeDefinition);
            if (!typeDefinition || (!typeDefinition.outputs && !typeDefinition.inputs)) {
                continue;
            }
            let constructorDefinition = typeDefinition.constructors;
            let constructableNodes: ItemDefinition<Schemes>[] = [];
            for (const constructorDefinitionKey in constructorDefinition) {
                let constructorDefinitionElement = constructorDefinition[constructorDefinitionKey];
                if (typeof constructorDefinitionElement == "string") {
                    console.error("Current constructor is a string constructor", constructorDefinitionElement, constructorDefinition.types);
                    continue;
                }

                constructableNodes.push([
                    constructorDefinitionKey,
                    () => {
                        let nodeName = fullTypeName.replaceAll("::", "_");
                        let countOfType = this.nodeNameCount.get(nodeName);
                        let node = new FlowrsNode(nodeName + (countOfType || ""), fullTypeName, null, constructorDefinitionKey, null, typeDefinitionsMap, this.editor!);
                        this.nodeNameCount.set(nodeName, (countOfType || 0) + 1);
                        return node;
                    },
                ]);
            }
            output.push([fullTypeName, constructableNodes]);
        }
        return output;
    }

    /**
     * Prevents type-incompatible connections in the Rete.js editor.
     * @param editor - The Rete.js editor to prevent type-incompatible connections in.
     */
    private static preventTypeIncompatibleConnections(editor: NodeEditor<Schemes>) {
        editor.addPipe((context) => {
            if (context.type == "connectioncreate") {
                let sourceNode = editor.getNode(context.data.source);
                let targetNode = editor.getNode(context.data.target);
                if (sourceNode.getTypeForKey(context.data.sourceOutput) != targetNode.getTypeForKey(context.data.targetInput)) {
                    console.warn("Keys dont match!", context.data, sourceNode, targetNode);
                    return;
                }
            }
            return context;
        });
    }

    /**
     * Creates a context menu with constructable nodes based on type definitions.
     * @param typeDefinitionsMap - Map of type definitions for nodes.
     * @returns A Promise of the context menu with constructable nodes.
     */
    private static async createContextMenuWithConstructableNodes(typeDefinitionsMap: Map<string, Type>) {
        this.constructableNodeList = await this.getConstructableNodeList(typeDefinitionsMap);
        return new ContextMenuPlugin<Schemes>({
            items: (context, plugin) => {
                return ContextMenuPresets.classic.setup(this.constructableNodeList)(context, plugin);
            }
        });
    }
}
