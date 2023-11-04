import {ClassicPreset as Classic, GetSchemes, NodeEditor} from 'rete';

import {Area2D, AreaExtensions, AreaPlugin} from 'rete-area-plugin';
import {
    ConnectionPlugin,
    Presets as ConnectionPresets,
} from 'rete-connection-plugin';

import {VuePlugin, VueArea2D, Presets as VuePresets} from 'rete-vue-plugin';

import {DataflowEngine, DataflowNode} from 'rete-engine';
import {
    AutoArrangePlugin,
    Presets as ArrangePresets,
} from 'rete-auto-arrange-plugin';

import {
    ContextMenuPlugin,
    ContextMenuExtra,
    Presets as ContextMenuPresets,
} from 'rete-context-menu-plugin';
import {MinimapExtra, MinimapPlugin} from 'rete-minimap-plugin';
import {
    ReroutePlugin,
    RerouteExtra,
    RerouteExtensions,
} from 'rete-connection-reroute-plugin';
import {
    HistoryPlugin,
    HistoryActions,
    HistoryExtensions,
    Presets
} from "rete-history-plugin";
import {FlowConnection, FlowNode} from "~/repository/modules/projects";
import {A} from "vite-node/types-516036fa";

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

class Connection<A extends Node, B extends Node> extends Classic.Connection<A, B>
{}

class GeneralFlowNode extends Classic.Node implements DataflowNode {
    constructor(initial: number, change?: (value: number) => void) {
        super('Number');

        this.addOutput('value', new Classic.Output(socket, 'Number'));
        this.addControl(
            'value',
            new Classic.InputControl('number', {initial, change})
        );
    }

    data() {
        const value = (this.controls['value'] as Classic.InputControl<'number'>)
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

    const contextMenu = new ContextMenuPlugin<Schemes>({
        items: ContextMenuPresets.classic.setup([
            ['Node', () => new NumberNode(1, process)],
            ['Add', () => new AddNode()],
        ]),
    });
    const minimap = new MinimapPlugin<Schemes>();
    const reroutePlugin = new ReroutePlugin<Schemes>();
    const history = new HistoryPlugin<Schemes, HistoryActions<Schemes>>();
    history.addPreset(Presets.classic.setup())
    HistoryExtensions.keyboard(history);

    editor.use(area);

    area.use(vueRender);

    area.use(connection);
    area.use(contextMenu);
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

    const a = new NumberNode(1, process);
    const b = new NumberNode(1, process);
    const add = new AddNode();

    await editor.addNode(a);
    await editor.addNode(b);
    await editor.addNode(add);

    await editor.addConnection(new Connection(a, 'value', add, 'a'));
    await editor.addConnection(new Connection(b, 'value', add, 'b'));

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

    async function process() {
        dataflow.reset();

        editor
            .getNodes()
            .filter((node) => node instanceof AddNode)
            .forEach(async (node) => {
                const sum = await dataflow.fetch(node.id);

                console.log(node.id, 'produces', sum);

                area.update(
                    'control',
                    (node.controls['result'] as Classic.InputControl<'number'>).id
                );
            });
    }

    editor.addPipe((context) => {
        if (
            context.type === 'connectioncreated' ||
            context.type === 'connectionremoved'
        ) {
            process();
        }
        return context;
    });

    process();

    return {
        destroy: () => area.destroy(),
    };
}
