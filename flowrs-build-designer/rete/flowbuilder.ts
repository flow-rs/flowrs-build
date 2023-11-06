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
    type ContextMenuExtra,

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
import type {TypeDefinition} from "~/repository/modules/packages";
import {createAllPackagesToTypeDefintionMap} from "~/repository/modules/packages";

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
    width = 400;
    height = 120;

    constructor(name: string, types: TypeDefinition, data: any) {
        super(name);
        let hasNoInputs: boolean = false;
        console.log("types", types)
        if (types.inputs) {
            for (let i = 0; i < types.inputs.length; i++) {
                this.addInput(types.inputs[i], new Classic.Input(socket, types.inputs[i]));
            }
        } else {
            hasNoInputs = true;
        }
        if (types.outputs) {
            if (hasNoInputs) {
                this.addControl(
                    'value', // TODO determine type
                    new Classic.InputControl('text', data)
                );
            }

            for (let i = 0; i < types.outputs.length; i++) {
                this.addOutput(types.outputs[i], new Classic.Output(socket, types.outputs[i]));
            }
        }
    }

    data() {
        const value = (this.controls['value'] as Classic.InputControl<'text'>)
            .value;

        return {
            value,
        };
    }
}

const socket = new Classic.Socket('socket');

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
    const selectedProject = computed(() => projectsStore.selectedProject);

    let allPackagesToTypeDefintionMap = await createAllPackagesToTypeDefintionMap();

    let project = selectedProject.value;
    for (let flowNode in project?.flow.nodes) {
        let currentNodetype = project.flow.nodes[flowNode].node_type;
        let typeDefinition = allPackagesToTypeDefintionMap.get(currentNodetype);
        if (!typeDefinition) {
            console.error("TypeDefinition for", currentNodetype, "not found")
            continue
        }
        console.log(allPackagesToTypeDefintionMap, currentNodetype);
        const node = new GeneralFlowNode(flowNode, typeDefinition, `${project?.flow.data[flowNode]?.value}`);
        console.log("add node", project, node)
        await editor.addNode(node);
    }

    const arrange = new AutoArrangePlugin<Schemes>();

    arrange.addPreset(ArrangePresets.classic.setup());

    area.use(arrange);

    await arrange.layout();

    AreaExtensions.zoomAt(area, editor.getNodes());

    AreaExtensions.simpleNodesOrder(area);

    const selector = AreaExtensions.selector();
    const accumulating = AreaExtensions.accumulateOnCtrl();

    AreaExtensions.selectableNodes(area, selector, {accumulating});
    RerouteExtensions.selectablePins(reroutePlugin, selector, accumulating);

    return {
        destroy: () => area.destroy(),
    };
}
