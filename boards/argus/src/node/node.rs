use uor_utils::messages::argus::envelope::{Node, NodeType};

#[cfg(feature = "temperature")]
pub static CURRENT_NODE: Node = Node {
	r#type: NodeType::ArgusTemperature as i32,
	id: Some(0),
};

#[cfg(feature = "pressure")]
pub static CURRENT_NODE: Node = Node {
	r#type: NodeType::ArgusPressure as i32,
	id: Some(0),
};

#[cfg(feature = "strain")]
pub static CURRENT_NODE: Node = Node {
	r#type: NodeType::ArgusStrain as i32,
	id: Some(0),
};
