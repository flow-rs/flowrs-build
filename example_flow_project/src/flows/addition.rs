use flowrs_core::{Error, Flow};
use flowrs_macros::connect_nodes;
use flowrs_std::add::SimpleAddNode;
use flowrs_std::debug::DebugNode;
use flowrs_std::value::ValueNode;

pub async fn return_flow() -> Result<Flow, Error> {
    let mut flow = Flow::new_empty();

    let number_node_1 = ValueNode::<u32>::new(3);
    let number_node_2 = ValueNode::<u32>::new(2);
    let add_node = SimpleAddNode::<u32>::new();
    let debug_node = DebugNode::<u32>::new(true);

    flow.add_node_with_id(Box::new(number_node_1), 1);
    flow.add_node_with_id(Box::new(number_node_2), 2);
    flow.add_node_with_id(Box::new(add_node), 3);
    flow.add_node_with_id(Box::new(debug_node), 4);

    connect_nodes!(u32, flow, 1, 3, 0, 0)?;
    connect_nodes!(u32, flow, 2, 3, 0, 1)?;
    connect_nodes!(u32, flow, 3, 4, 0, 0)?;

    Ok(flow)
}
