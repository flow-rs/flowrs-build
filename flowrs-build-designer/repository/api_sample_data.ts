import {
    type FlowNode,
    type  FlowProject,
    type TimerConfigNode,
    type TimerTokenNode
} from "~/repository/modules/projects";


const timerConfigNodeData: TimerConfigNode = {value: {duration: {secs: 1, nanos: 0}}};
const timerTokenNodeData: TimerTokenNode = {value: 42}
const debugNode: FlowNode = {
    node_type: "flowrs_std::nodes::debug::DebugNode",
    type_parameters: {"I": "i32"},
    constructor: "New"
}
const timerConfigNode: FlowNode = {
    node_type: "flowrs_std::nodes::value::ValueNode",
    type_parameters: {"I": "flowrs_std::nodes::timer::TimerNodeConfig"},
    constructor: "New"
}
const timerTokenNode: FlowNode = {
    node_type: "flowrs_std::nodes::value::ValueNode",
    type_parameters: {"I": "i32"},
    constructor: "New"
}
const timerNode: FlowNode = {
    node_type: "flowrs_std::nodes::timer::TimerNode",
    type_parameters: {"T": "flowrs_std::nodes::timer::SelectedTimer", "U": "i32"},
    constructor: "New"
}

export const newFlowProject: FlowProject = {
    name: 'flow_project_301',
    version: '1.0.0',
    packages: [
        {
            name: 'flowrs',
            version: '0.2.0',
            //path: "../../../flowrs",
            git: "https://github.com/flow-rs/flowrs",
            branch: "feature-project7"
        }
    ],
    flow: {
        nodes: {
        },
        connections: [
        ],
        data: {
        },
    },
};
