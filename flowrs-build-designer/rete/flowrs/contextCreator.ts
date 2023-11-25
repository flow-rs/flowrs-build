import {type FlowProject} from "~/repository/modules/projects";
import {FlowrsNode} from "~/rete/flowrs/flowrsNode";
import {NodeEditor} from "rete";
import {type ItemDefinition} from "rete-context-menu-plugin/_types/presets/classic/types";
import {useProjectsStore} from "~/store/projectStore";
import {ContextMenuPlugin, Presets as ContextMenuPresets} from "rete-context-menu-plugin";
import type {Schemes} from "~/rete/flowrs/editor";
import {Connection} from "~/rete/flowrs/editor";
import type {TypeDefinition} from "~/repository/modules/packages";


export class ContextCreator {

    public static async addFlowrsElements(editor: NodeEditor<Schemes>) {
        const selectedProject = this.getCurrentlySelectedProject();

        // get (typename,typeDefinition) Map
        let typeDefinitionsMap = await useNuxtApp().$api.packages.getFlowrsTypeDefinitionsMap();

        // construct scene out of selected project
        if (selectedProject) {
            let projectNodesMap = await this.addProjectNodes(selectedProject, typeDefinitionsMap, editor);
            await this.connectProjectNodes(selectedProject, projectNodesMap, editor);
        }

        this.preventTypeIncompatibleConnections(editor);

        return await this.createContextMenuWithConstructableNodes(typeDefinitionsMap);
    }

    private static getCurrentlySelectedProject() {
        const projectsStore = useProjectsStore();
        const selectedProjectUnwrapped = computed(() => projectsStore.selectedProject);
        const selectedProject: FlowProject | null = selectedProjectUnwrapped.value;
        return selectedProject;
    }

    private static async addProjectNodes(project: FlowProject, typeDefinitionsMap: Map<string, TypeDefinition>, editor: NodeEditor<Schemes>) {
        let allAddedNodes: Map<string, FlowrsNode> = new Map();

        for (let flowNode in project.flow.nodes) {
            let currentNode = project.flow.nodes[flowNode];
            let currentNodeType = currentNode.node_type;
            let typeDefinition = typeDefinitionsMap.get(currentNodeType);
            if (!typeDefinition) {
                console.error("TypeDefinition for", currentNodeType, "not found")
                continue
            }
            const node = new FlowrsNode(
                flowNode,
                typeDefinition,
                project.flow.data[flowNode]?.value,
                currentNode.constructor,
                currentNode.type_parameters,
                typeDefinitionsMap);

            await editor.addNode(node);
            allAddedNodes.set(flowNode, node);
            console.log('node added', flowNode, node);
        }

        return allAddedNodes;
    }

    private static async connectProjectNodes(project: FlowProject, projectNodes: Map<string, FlowrsNode>, editor: NodeEditor<Schemes>) {
        for (let flowConnection of project.flow.connections) {
            let source = projectNodes.get(flowConnection.from_node);
            let target = projectNodes.get(flowConnection.to_node);

            if (!source || !target) {
                console.error(source, target, 'not found in', projectNodes)
                return
            }

            let newConnection = new Connection(
                source,
                flowConnection.from_output,
                target,
                flowConnection.to_input,
            );

            await editor.addConnection(newConnection);
            console.log('connection added', newConnection);
        }
    }

    private static async getConstructableNodeList(typeDefinitionsMap: Map<string, TypeDefinition>): Promise<ItemDefinition<Schemes>[]> {
        let output: ItemDefinition<Schemes>[] = [];
        for (const fullTypeName of typeDefinitionsMap.keys()) {
            let typeDefinition = typeDefinitionsMap.get(fullTypeName);
            console.log('Package', fullTypeName, typeDefinition);
            if (!typeDefinition || (!typeDefinition.outputs && !typeDefinition.inputs)) {
                continue
            }
            let constructorDefinition = typeDefinition.constructors;
            let constructableNodes: ItemDefinition<Schemes>[] = [];
            if (constructorDefinition.New) {
                constructableNodes.push(["New",
                    () => new FlowrsNode(
                        fullTypeName,
                        typeDefinition!,
                        null,
                        "New",
                        null,
                        typeDefinitionsMap)]);
            }
            if (constructorDefinition.NewWithToken) {
                constructableNodes.push(["NewWithToken",
                    () => new FlowrsNode(
                        fullTypeName,
                        typeDefinition!,
                        null,
                        "NewWithToken",
                        null,
                        typeDefinitionsMap)]);
            }
            output.push([fullTypeName, constructableNodes]);
        }
        return output;
    }

    // TODO would be nicer if it would prevent the dropping of the connection onto the wrong socket --> smth like https://retejs.org/docs/guides/connections --> How do i get from socket to node ?
    private static preventTypeIncompatibleConnections(editor: NodeEditor<Schemes>) {
        editor.addPipe(context => {
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

    private static async createContextMenuWithConstructableNodes(typeDefinitionsMap: Map<string, TypeDefinition>) {
        let constructableNodeList = await this.getConstructableNodeList(typeDefinitionsMap);
        return new ContextMenuPlugin<Schemes>({
            items: ContextMenuPresets.classic.setup(constructableNodeList),
        });
    }
}
