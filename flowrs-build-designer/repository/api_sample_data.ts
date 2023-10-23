import {FlowNode, FlowProject, TimerConfigNode, TimerTokenNode} from "~/repository/modules/projects";


const timerConfigNodeData: TimerConfigNode = {value: {duration: {secs: 1, nanos: 0}}};
const timerTokenNodeData: TimerTokenNode = {value: 42}
const debugNode: FlowNode<any> = {node_type: "flowrs_std::nodes::debug::DebugNode", type_parameters: {"I": "i32"}, constructor: "New"}
const timerConfigNode: FlowNode<any> = {node_type: "flowrs_std::nodes::value::ValueNode", type_parameters: {"I": "flowrs_std::nodes::timer::TimerNodeConfig"}, constructor: "New"}
const timerTokenNode: FlowNode<any> = {node_type: "flowrs_std::nodes::value::ValueNode", type_parameters: {"I": "i32"}, constructor: "New"}
const timerNode: FlowNode<any> = {node_type: "flowrs_std::nodes::timer::TimerNode", type_parameters: {"T": "flowrs_std::nodes::timer::SelectedTimer", "U": "i32"}, constructor: "New"}

export const newFlowProject: FlowProject = {
    name: 'flow_project_100',
    version: '1.0.0',
    packages: [
        {
            name: 'flowrs',
            version: '1.0.0',
            path: '../../../flowrs',
        },
        {
            name: 'flowrs-std',
            version: '1.0.0',
            path: '../../../flowrs-std',
        },
    ],
    flow: {
        nodes: {
            debug_node: debugNode,
            timer_config_node: timerConfigNode,
            timer_token_node: timerTokenNode,
            timer_node: timerNode
            // Define other nodes here
        },
        connections: [
            {
                from_node: 'timer_config_node',
                to_node: 'timer_node',
                to_input: 'config_input',
                from_output: 'output',
            },
            // Define other connections here
        ],
        data: {
            timer_config_node: timerConfigNodeData,
            timer_token_node: timerTokenNodeData,
        },
    },
};
