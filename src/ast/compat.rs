use crate::ast::core::Tag;

// TODO: Refactor to not use this intermediate form

#[derive(Clone, Debug)]
/// Net representation used only as an intermediate for converting to hvm-core format
pub struct INet {
  pub nodes: Vec<NodeVal>,
}

pub type NodeVal = u64;
pub type NodeKind = NodeVal;
pub type Port = NodeVal;
pub type NodeId = NodeVal;
pub type SlotId = NodeVal;

/// The ROOT port is on the deadlocked root node at address 0.
pub const ROOT: Port = 1;
pub const TAG_WIDTH: u32 = Tag::BITS;
pub const TAG: u32 = 64 - TAG_WIDTH;
pub const ERA: NodeKind = 0 << TAG;
pub const CON: NodeKind = 1 << TAG;
pub const DUP: NodeKind = 2 << TAG;
pub const REF: NodeKind = 3 << TAG;
pub const NUM: NodeKind = 4 << TAG;
pub const NUMOP: NodeKind = 5 << TAG;
pub const LABEL_MASK: NodeKind = (1 << TAG) - 1;
pub const TAG_MASK: NodeKind = !LABEL_MASK;

/// Create a new net, with a deadlocked root node.
pub fn new_inet() -> INet {
  INet {
    nodes: vec![2, 1, 0, ERA], // p2 points to p0, p1 points to net
  }
}

/// Allocates a new node, reclaiming a freed space if possible.
pub fn new_node(inet: &mut INet, kind: NodeKind) -> NodeId {
  let node = addr(inet.nodes.len() as Port);
  inet.nodes.extend([port(node, 0), port(node, 1), port(node, 2), kind]);
  node
}

/// Builds a port (an address / slot pair).
pub fn port(node: NodeId, slot: SlotId) -> Port {
  (node << 2) | slot
}

/// Returns the address of a port (TODO: rename).
pub fn addr(port: Port) -> NodeId {
  port >> 2
}

/// Returns the slot of a port.
pub fn slot(port: Port) -> SlotId {
  port & 3
}

/// Enters a port, returning the port on the other side.
pub fn enter(inet: &INet, port: Port) -> Port {
  inet.nodes[port as usize]
}

/// Kind of the node.
pub fn kind(inet: &INet, node: NodeId) -> NodeKind {
  inet.nodes[port(node, 3) as usize]
}

/// Links two ports.
pub fn link(inet: &mut INet, ptr_a: Port, ptr_b: Port) {
  inet.nodes[ptr_a as usize] = ptr_b;
  inet.nodes[ptr_b as usize] = ptr_a;
}

#[derive(Debug)]
pub struct INode {
  pub kind: NodeKind,
  pub ports: [String; 3],
}

pub type INodes = Vec<INode>;