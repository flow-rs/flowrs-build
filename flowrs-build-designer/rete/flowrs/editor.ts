import {ClassicPreset as Classic, type GetSchemes, NodeEditor} from 'rete';

import {type Area2D, AreaExtensions, AreaPlugin} from 'rete-area-plugin';
import {ConnectionPlugin, Presets as ConnectionPresets,} from 'rete-connection-plugin';

import {Presets as VuePresets, type VueArea2D, VuePlugin} from 'rete-vue-plugin';

import {AutoArrangePlugin, Presets as ArrangePresets,} from 'rete-auto-arrange-plugin';

import {type ContextMenuExtra,} from 'rete-context-menu-plugin';
import {type MinimapExtra, MinimapPlugin} from 'rete-minimap-plugin';
import {type RerouteExtra, ReroutePlugin,} from 'rete-connection-reroute-plugin';
import {type HistoryActions, HistoryExtensions, HistoryPlugin, Presets} from "rete-history-plugin";
import {FlowrsNode} from "~/rete/flowrs/flowrsNode";
import {ContextCreator} from "~/rete/flowrs/contextCreator";

import CustomDropdownControl from "../../components/CustomDropdownControl.vue";

import type {TypeDefinition} from "~/repository/modules/packages";

type Node = FlowrsNode;
type Conn =
    | Connection<FlowrsNode, FlowrsNode>;
export type Schemes = GetSchemes<Node, Conn>;

type AreaExtra =
    | Area2D<Schemes>
    | VueArea2D<Schemes>
    | ContextMenuExtra
    | MinimapExtra
    | RerouteExtra;

export class Connection<A extends Node, B extends Node> extends Classic.Connection<A, B> {
}

export class DropdownControl extends Classic.Control {
    typeName: string;
    possibleValues: [string, TypeDefinition][];

    constructor(typeName: string, possibleValues: [string, TypeDefinition][], public onSelection: (selectedTypeName: string) => void) {
        super()
        this.typeName = typeName;
        this.possibleValues = possibleValues;
    }
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

    vueRender.addPreset(VuePresets.classic.setup({
        customize: {
            control(data) {
                if (data.payload instanceof DropdownControl) {
                    return CustomDropdownControl;
                }
                if (data.payload instanceof Classic.InputControl) {
                    return VuePresets.classic.Control;
                }
            }
        }
    }));
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

    // inject all flowrs specific things into rete editor
    const contextMenu = await ContextCreator.addFlowrsElements(editor);
    area.use(contextMenu);

    const arrange = new AutoArrangePlugin<Schemes>();

    arrange.addPreset(ArrangePresets.classic.setup());

    area.use(arrange);

    await arrange.layout();

    await AreaExtensions.zoomAt(area, editor.getNodes());

    AreaExtensions.simpleNodesOrder(area);

    return {
        destroy: () => area.destroy(),
    };
}
