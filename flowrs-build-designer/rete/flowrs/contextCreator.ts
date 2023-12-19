import { type FlowNode, type FlowConnection, type FlowProject, type ProjectIdentifier } from "~/repository/modules/projects";
import { FlowrsNode } from "~/rete/flowrs/flowrsNode";
import { NodeEditor } from "rete";
import { type ItemDefinition } from "rete-context-menu-plugin/_types/presets/classic/types";
import { useProjectsStore } from "~/store/projectStore";
import { ContextMenuPlugin, Presets as ContextMenuPresets } from "rete-context-menu-plugin";
import type { Schemes } from "~/rete/flowrs/editor";
import { Connection } from "~/rete/flowrs/editor";
import type { TypeDefinition } from "~/repository/modules/packages";
import { usePackagesStore } from "~/store/packageStore.js";
import { navigateTo } from "#app";

const packagesStore = usePackagesStore();
export class ContextCreator {
  private static editor: NodeEditor<Schemes> | undefined;

  private static nodeNameCount: Map<string, number> = new Map<string, number>();

  public static async saveBuilderStateAsProject() {
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
    let typeDefinitionsMap = await useNuxtApp().$api.packages.getFlowrsTypeDefinitionsMap();

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
      await useNuxtApp().$api.projects.deleteProject({ project_name: flowProject.name });
    } catch (e) {
      console.log("Delete failed", e);
    }

    // delete the old & create the new setup, knowing that it will succeed
    flowProject.name = original_name;
    try {
      await useNuxtApp().$api.projects.deleteProject({ project_name: flowProject.name });
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
  private static async setPackagesInProject(flowProject: FlowProject) {
    const packages: string[]=[];
    const toRemovePackages: string[]=[];
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
        if(!packagesStore.currentActive.includes(element)){
            toRemovePackages.push(element);
        }
    });
    toRemovePackages.forEach(element => {
        flowProject.packages = flowProject.packages.filter(obj => obj.name !== element);
    });
  }
  private static setNodesAndDataInProject(flowProject: FlowProject, allNodes: Schemes["Node"][], typeDefinitionsMap: Map<string, TypeDefinition>) {
    flowProject.flow.nodes = {};
    flowProject.flow.data = {};
    for (const node of allNodes) {
      this.addNodeToProject(typeDefinitionsMap, node, flowProject);

      this.addDataToProject(node, flowProject);
    }
  }

  private static addNodeToProject(typeDefinitionsMap: Map<string, TypeDefinition>, node: FlowrsNode, flowProject: FlowProject) {
    let typeDefinitionOfCurrentNode = typeDefinitionsMap.get(node.fullTypeName);
    if (!typeDefinitionOfCurrentNode) {
      throw new Error(`Node ${node.label} is currently not in the package list`);
    }

    let flowNode: FlowNode = {
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

  private static addDataToProject(node: FlowrsNode, flowProject: FlowProject) {
    if (node.node_data) {
      try {
        let parsedJson = JSON.parse(node.node_data);
        flowProject.flow.data[node.label] = { value: parsedJson };
      } catch (e) {
        throw new Error(`Check the input field of node ${node.label}. The input field could not be parsed as JSON`);
      }
    }
  }

  private static setConnectionsInProject(flowProject: FlowProject, allConnections: Schemes["Connection"][]) {
    if (!this.editor) {
      throw new Error("Editor is undefined!");
    }
    flowProject.flow.connections = [];
    for (const connection of allConnections) {
      let flowConnection: FlowConnection = {
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

  public static async addFlowrsElements(editor: NodeEditor<Schemes>) {
    this.editor = editor;
    this.nodeNameCount = new Map<string, number>();
    const selectedProject = this.getCurrentlySelectedProject();

    // get (typename,typeDefinition) Map
    let typeDefinitionsMap = await useNuxtApp().$api.packages.getFlowrsTypeDefinitionsMap();

    // construct scene out of selected project
    if (selectedProject) {
      let projectNodesMap = await this.addProjectNodes(selectedProject, typeDefinitionsMap, editor);
      await this.connectProjectNodes(selectedProject, projectNodesMap, editor);
    }

    this.preventTypeIncompatibleConnections(editor);
    typeDefinitionsMap = this.filterInActive(typeDefinitionsMap);
    return await this.createContextMenuWithConstructableNodes(typeDefinitionsMap);
  }
  public static async updateContextMenu() {
    let typeDefinitionsMap = await useNuxtApp().$api.packages.getFlowrsTypeDefinitionsMap();
    typeDefinitionsMap = this.filterInActive(typeDefinitionsMap);
    return await this.createContextMenuWithConstructableNodes(typeDefinitionsMap);
  }
  private static filterInActive(map: Map<string, TypeDefinition>) {
    const keysToRemove: string[] = [];
    console.log(toRaw(packagesStore.currentActive));
    map.forEach((value, key) => {
      let keyToSearch = key.substring(0, key.indexOf("::")).replace("_", "-");
      if (!toRaw(packagesStore.currentActive).includes(keyToSearch)) {
        console.log(key);
        keysToRemove.push(key);
      }
    });
    keysToRemove.forEach((key) => {
      map.delete(key);
    });
    console.log(map);
    return map;
  }
  private static getCurrentlySelectedProject() {
    const projectsStore = useProjectsStore();
    const selectedProjectUnwrapped = computed(() => projectsStore.selectedProject);
    const selectedProject: FlowProject | null = selectedProjectUnwrapped.value;
    if (!selectedProject) {
      navigateTo("/");
    }
    return selectedProject;
  }

  private static async addProjectNodes(project: FlowProject, typeDefinitionsMap: Map<string, TypeDefinition>, editor: NodeEditor<Schemes>) {
    let allAddedNodes: Map<string, FlowrsNode> = new Map();

    for (let flowNode in project.flow.nodes) {
      let currentNode = project.flow.nodes[flowNode];
      let currentNodeType = currentNode.node_type;
      let typeDefinition = typeDefinitionsMap.get(currentNodeType);
      if (!typeDefinition) {
        console.error("TypeDefinition for", currentNodeType, "not found");
        continue;
      }

      let nodeName = flowNode.replaceAll("::", "_");
      let countOfType = this.nodeNameCount.get(nodeName);
      const node = new FlowrsNode(nodeName + (countOfType || ""), currentNodeType, project.flow.data[flowNode]?.value, currentNode.constructor, currentNode.type_parameters, typeDefinitionsMap, editor);

      this.nodeNameCount.set(nodeName, (countOfType || 0) + 1);

      await editor.addNode(node);
      allAddedNodes.set(flowNode, node);
      console.log("node added", flowNode, node);
    }

    return allAddedNodes;
  }

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

  private static async getConstructableNodeList(typeDefinitionsMap: Map<string, TypeDefinition>): Promise<ItemDefinition<Schemes>[]> {
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

  private static async createContextMenuWithConstructableNodes(typeDefinitionsMap: Map<string, TypeDefinition>) {
    let constructableNodeList = await this.getConstructableNodeList(typeDefinitionsMap);
    return new ContextMenuPlugin<Schemes>({
      items: ContextMenuPresets.classic.setup(constructableNodeList),
    });
  }
}
