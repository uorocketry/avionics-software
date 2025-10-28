use messages::argus::envelope::{Node, NodeType};

// The identifier for this board (node) with some params set at compile time.
pub const CURRENT_NODE: Node = Node {
	// SHOULD DO: inject from environment variable once we have a way to convert option_env to i32 in no_std environments.
	id: None,

	// Set node type based on enabled feature.
	#[cfg(feature = "temperature")]
	r#type: NodeType::ArgusTemperature as i32,
	#[cfg(feature = "pressure")]
	r#type: NodeType::ArgusPressure as i32,
	#[cfg(feature = "strain")]
	r#type: NodeType::ArgusStrain as i32,
};
