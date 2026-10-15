use crate::layout::{Constraints, Point, Size};
use slotmap::SlotMap;

slotmap::new_key_type! {
    /// Handle into the view-tree arena.
    pub struct NodeId;
}

/// Computed layout rectangle (position is relative to parent).
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Defines how a node measures itself and positions its children.
pub trait NodeLayout {
    /// Constraints to propose to the child at `index`.
    fn child_constraints(&self, _index: usize, constraints: &Constraints) -> Constraints {
        *constraints
    }

    /// Compute this node's size given its children's measured sizes.
    fn measure(&self, child_sizes: &[Size], constraints: &Constraints) -> Size;

    /// Compute each child's position relative to this node's origin.
    fn arrange(&self, child_sizes: &[Size], own_size: Size) -> Vec<Point>;
}

pub(crate) struct Node {
    pub(crate) children: Vec<NodeId>,
    pub(crate) layout: Box<dyn NodeLayout>,
    pub(crate) rect: LayoutRect,
}

/// Arena-based view tree that owns all nodes and runs the layout pass.
pub struct ViewTree {
    nodes: SlotMap<NodeId, Node>,
}

impl ViewTree {
    /// Create an empty tree.
    pub fn new() -> Self {
        Self {
            nodes: SlotMap::with_key(),
        }
    }

    /// Insert a node with the given layout behaviour and children.
    pub fn add_node(&mut self, layout: Box<dyn NodeLayout>, children: Vec<NodeId>) -> NodeId {
        self.nodes.insert(Node {
            children,
            layout,
            rect: LayoutRect::default(),
        })
    }

    /// Run the layout pass from `root` with the given constraints.
    pub fn layout(&mut self, root: NodeId, constraints: Constraints) {
        self.layout_node(root, constraints);
    }

    fn layout_node(&mut self, id: NodeId, constraints: Constraints) -> Size {
        let children: Vec<NodeId> = self.nodes[id].children.clone();

        let child_constraints: Vec<Constraints> = {
            let layout = &*self.nodes[id].layout;
            (0..children.len())
                .map(|i| layout.child_constraints(i, &constraints))
                .collect()
        };

        let child_sizes: Vec<Size> = children
            .iter()
            .zip(child_constraints.iter())
            .map(|(&child_id, &cc)| self.layout_node(child_id, cc))
            .collect();

        let (own_size, positions) = {
            let layout = &*self.nodes[id].layout;
            let size = layout.measure(&child_sizes, &constraints);
            let pos = layout.arrange(&child_sizes, size);
            (size, pos)
        };

        self.nodes[id].rect.width = own_size.width;
        self.nodes[id].rect.height = own_size.height;

        for (i, &child_id) in children.iter().enumerate() {
            if let Some(pos) = positions.get(i) {
                self.nodes[child_id].rect.x = pos.x;
                self.nodes[child_id].rect.y = pos.y;
            }
        }

        own_size
    }

    /// Computed layout for a node (position relative to parent).
    pub fn rect(&self, id: NodeId) -> LayoutRect {
        self.nodes[id].rect
    }

    /// Children of a node.
    pub fn children(&self, id: NodeId) -> &[NodeId] {
        &self.nodes[id].children
    }
}

impl Default for ViewTree {
    fn default() -> Self {
        Self::new()
    }
}
