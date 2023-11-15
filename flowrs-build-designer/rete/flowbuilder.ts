import {ClassicPreset as Classic, type GetSchemes, NodeEditor} from 'rete';

import {type Area2D, AreaExtensions, AreaPlugin} from 'rete-area-plugin';
import {
    ConnectionPlugin,
    Presets as ConnectionPresets,
} from 'rete-connection-plugin';

import {VuePlugin, type VueArea2D, Presets as VuePresets} from 'rete-vue-plugin';

import {DataflowEngine, type DataflowNode} from 'rete-engine';
import {
    AutoArrangePlugin,
    Presets as ArrangePresets,
} from 'rete-auto-arrange-plugin';

import {
    type ContextMenuExtra, ContextMenuPlugin, Presets as ContextMenuPresets,

} from 'rete-context-menu-plugin';
import {type MinimapExtra, MinimapPlugin} from 'rete-minimap-plugin';
import {
    ReroutePlugin,
    type RerouteExtra,
    RerouteExtensions,
} from 'rete-connection-reroute-plugin';
import {
    HistoryPlugin,
    type HistoryActions,
    HistoryExtensions,
    Presets
} from "rete-history-plugin";
import {useProjectsStore} from "~/store/projectStore";
import type {
    ConstructorDefinition,
    ConstructorDescription,
    TypeDefinition,
    TypeDescription
} from "~/repository/modules/packages";
import {createAllPackagesToTypeDefintionMap} from "~/repository/modules/packages";
import type {FlowProject} from "~/repository/modules/projects";
import type {ItemDefinition} from "rete-context-menu-plugin/_types/presets/classic/types";

type Node = GeneralFlowNode;
type Conn =
    | Connection<GeneralFlowNode, GeneralFlowNode>
type Schemes = GetSchemes<Node, Conn>;

type AreaExtra =
    | Area2D<Schemes>
    | VueArea2D<Schemes>
    | ContextMenuExtra
    | MinimapExtra
    | RerouteExtra;

class Connection<A extends Node, B extends Node> extends Classic.Connection<A, B> {
}

class GeneralFlowNode extends Classic.Node implements DataflowNode {
    width = 500;
    height = 140;
    node_data: string | undefined;
    constructor_type: string = "New";

    constructor(name: string, typeDefinition: TypeDefinition, data: { [key: string]: any } | null, constructor_type: string) {
        super(name);

        this.constructor_type = constructor_type;
        this.setNodeData(data);
        this.addChooseConstructorAndControlInputs(typeDefinition.constructors)
        this.addInputs(typeDefinition);
        this.addOutputs(typeDefinition);
    }

    private addChooseConstructorAndControlInputs(constructors: ConstructorDefinition) {
        let currentConstructor;
        if (this.constructor_type == "New") {
            currentConstructor = constructors.New;
        } else if (this.constructor_type == "NewWithToken") {
            currentConstructor = constructors.NewWithToken;
        }
        if (!currentConstructor) {
            return;
        }
        let recordKeys = Object.keys(currentConstructor);
        if (recordKeys.length > 1) {
            console.error("This isn't supposed to be like that!", currentConstructor);
        }
        this.addControlInputs(currentConstructor[recordKeys[0]]);
    }

    private addControlInputs(constructor: ConstructorDescription) {
        if (!constructor.arguments) {
            return
        }
        for (const argument of constructor.arguments) {
            if (argument.construction.Constructor == "Json") {
                this.addControl(
                    'data',
                    new Classic.InputControl('text', {
                        initial: this.node_data, readonly: false, change: value => {
                            this.node_data = value;
                        }
                    })
                );
            }
        }

    }

    private setNodeData(data: { [key: string]: any } | null) {
        if (!data) {
            return
        }

        this.node_data = JSON.stringify(data, null, 2);
    }

    private addOutputs(types: TypeDefinition) {
        for (let name in types.outputs) {
            let typeDescription = types.outputs[name].type;
            let type_name = this.constructLabel(name, typeDescription);
            this.addOutput(name, new Classic.Output(socket, type_name));
        }
    }

    private addInputs(types: TypeDefinition) {
        for (let name in types.inputs) {
            let typeDescription = types.inputs[name].type;
            let type_name = this.constructLabel(name, typeDescription);
            this.addInput(name, new Classic.Input(socket, type_name));
            this.height += 20;
        }
    }

    private constructLabel(inputPairKey: string, typeDescription: TypeDescription) {
        let label = inputPairKey;
        if (typeDescription.Generic) {
            label += ' : ' + typeDescription.Generic.name;
        } else if (typeDescription.Type) {
            label += ' : ' + typeDescription.Type.name;
        } else {
            console.error("Type not supported", typeDescription)
        }
        return label;
    }

    data() {
        const value = this.node_data;
        return {value,};
    }
}

const socket = new Classic.Socket('socket');

async function addProjectNodes(project: FlowProject, allPackagesToTypeDefinitionMap: Map<string, TypeDefinition>, editor: NodeEditor<Schemes>) {
    let allAddedNodes: Map<string, GeneralFlowNode> = new Map();

    for (let flowNode in project.flow.nodes) {
        let currentNode = project.flow.nodes[flowNode];
        let currentNodeType = currentNode.node_type;
        let typeDefinition = allPackagesToTypeDefinitionMap.get(currentNodeType);
        if (!typeDefinition) {
            console.error("TypeDefinition for", currentNodeType, "not found")
            continue
        }
        const node = new GeneralFlowNode(flowNode, typeDefinition, project.flow.data[flowNode]?.value, currentNode.constructor);
        await editor.addNode(node);
        allAddedNodes.set(flowNode, node);
        console.log('node added', flowNode, node);
    }

    return allAddedNodes;
}

async function connectProjectNodes(project: FlowProject, projectNodes: Map<string, GeneralFlowNode>, editor: NodeEditor<Schemes>) {
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

async function getConstructableNodeList(allPackagesToTypeDefinitionMap: Map<string, TypeDefinition>): Promise<ItemDefinition<Schemes>[]> {
    let output: ItemDefinition<Schemes>[] = [];
    for (const fullTypeName of allPackagesToTypeDefinitionMap.keys()) {
        let typeDefinition = allPackagesToTypeDefinitionMap.get(fullTypeName);
        console.log('Package', fullTypeName, typeDefinition);
        if (!typeDefinition || (!typeDefinition.outputs && !typeDefinition.inputs)) {
            continue
        }
        let constructorDefinition = typeDefinition.constructors;
        let constructableNodes: ItemDefinition<Schemes>[] = [];
        if (constructorDefinition.New) {
            constructableNodes.push(["New",() => new GeneralFlowNode(fullTypeName, typeDefinition!, null, "New")]);
        }
        if (constructorDefinition.NewWithToken) {
            constructableNodes.push(["NewWithToken",() => new GeneralFlowNode(fullTypeName, typeDefinition!, null, "NewWithToken")]);
        }
        output.push([fullTypeName, constructableNodes]);
    }
    return output;
}

export async function createEditor(container: HTMLElement) {
    const editor = new NodeEditor<Schemes>();
    const area = new AreaPlugin<Schemes, AreaExtra>(container);
    const connection = new ConnectionPlugin<Schemes, AreaExtra>();

    const vueRender = new VuePlugin<Schemes, AreaExtra>();

    const minimap = new MinimapPlugin<Schemes>();
    const reroutePlugin = new ReroutePlugin<Schemes>();
    const history = new HistoryPlugin<Schemes, HistoryActions<Schemes>>();
    history.addPreset(Presets.classic.setup())
    HistoryExtensions.keyboard(history);

    editor.use(area);

    area.use(vueRender);

    area.use(connection);
    area.use(minimap);
    area.use(history);

    vueRender.use(reroutePlugin);

    connection.addPreset(ConnectionPresets.classic.setup());

    vueRender.addPreset(VuePresets.classic.setup());
    vueRender.addPreset(VuePresets.contextMenu.setup());
    vueRender.addPreset(VuePresets.minimap.setup());
    vueRender.addPreset(
        VuePresets.reroute.setup({
            contextMenu(id) {
                reroutePlugin.remove(id);
            },
            translate(id, dx, dy) {
                reroutePlugin.translate(id, dx, dy);
            },
            pointerdown(id) {
                reroutePlugin.unselect(id);
                reroutePlugin.select(id);
            },
        })
    );

    const dataflow = new DataflowEngine<Schemes>();

    editor.use(dataflow);

    const projectsStore = useProjectsStore();
    const selectedProjectUnwrapped = computed(() => projectsStore.selectedProject);
    const selectedProject: FlowProject | null = selectedProjectUnwrapped.value;

    let allPackagesToTypeDefinitionMap = await createAllPackagesToTypeDefintionMap();

    if (selectedProject) {
        let projectNodesMap = await addProjectNodes(selectedProject, allPackagesToTypeDefinitionMap, editor);
        await connectProjectNodes(selectedProject, projectNodesMap, editor);
    }

    let constructableNodeList = await getConstructableNodeList(allPackagesToTypeDefinitionMap);
    const contextMenu = new ContextMenuPlugin<Schemes>({
        items: ContextMenuPresets.classic.setup(constructableNodeList),
    });
    area.use(contextMenu);

    const arrange = new AutoArrangePlugin<Schemes>();

    arrange.addPreset(ArrangePresets.classic.setup());

    area.use(arrange);

    await arrange.layout();

    await AreaExtensions.zoomAt(area, editor.getNodes());

    AreaExtensions.simpleNodesOrder(area);

    const selector = AreaExtensions.selector();
    const accumulating = AreaExtensions.accumulateOnCtrl();

    AreaExtensions.selectableNodes(area, selector, {accumulating});
    RerouteExtensions.selectablePins(reroutePlugin, selector, accumulating);

    return {
        destroy: () => area.destroy(),
    };
}
