use crate::core::tree::{NodeId, NodeLayout, ViewTree};
use crate::layout::{Alignment, Constraints, EdgeInsets, Point, Size};

/// A declarative description of UI that builds nodes into the view tree.
pub trait View {
    /// Materialise this view into the tree, returning its root node.
    fn build(&self, tree: &mut ViewTree) -> NodeId;
}

/// Extension methods that wrap a view in modifier nodes.
pub trait ViewExt: View + Sized + 'static {
    /// Wrap in uniform padding on all edges.
    fn padding(self, amount: f32) -> Padded {
        Padded {
            inner: Box::new(self),
            insets: EdgeInsets::all(amount),
        }
    }

    /// Wrap in edge-specific padding.
    fn padding_edges(self, insets: EdgeInsets) -> Padded {
        Padded {
            inner: Box::new(self),
            insets,
        }
    }

    /// Wrap in a fixed-size frame (centered by default).
    fn frame_size(self, width: f32, height: f32) -> Framed {
        Framed {
            inner: Box::new(self),
            width: Some(width),
            height: Some(height),
            alignment: Alignment::CENTER,
        }
    }

    /// Wrap in a frame with optional dimensions and explicit alignment.
    fn frame(self, width: Option<f32>, height: Option<f32>, alignment: Alignment) -> Framed {
        Framed {
            inner: Box::new(self),
            width,
            height,
            alignment,
        }
    }
}

impl<V: View + Sized + 'static> ViewExt for V {}

// ---------------------------------------------------------------------------
// Padding modifier
// ---------------------------------------------------------------------------

/// A view wrapped in padding.
pub struct Padded {
    inner: Box<dyn View>,
    insets: EdgeInsets,
}

impl View for Padded {
    fn build(&self, tree: &mut ViewTree) -> NodeId {
        let child = self.inner.build(tree);
        tree.add_node(Box::new(PaddingLayout(self.insets)), vec![child])
    }
}

struct PaddingLayout(EdgeInsets);

impl NodeLayout for PaddingLayout {
    fn child_constraints(&self, _index: usize, constraints: &Constraints) -> Constraints {
        constraints.shrink(self.0)
    }

    fn measure(&self, child_sizes: &[Size], constraints: &Constraints) -> Size {
        let child = child_sizes.first().copied().unwrap_or(Size::ZERO);
        constraints.constrain(Size::new(
            child.width + self.0.horizontal(),
            child.height + self.0.vertical(),
        ))
    }

    fn arrange(&self, _child_sizes: &[Size], _own_size: Size) -> Vec<Point> {
        vec![Point::new(self.0.left, self.0.top)]
    }
}

// ---------------------------------------------------------------------------
// Frame modifier
// ---------------------------------------------------------------------------

/// A view wrapped in a frame with optional fixed dimensions.
pub struct Framed {
    inner: Box<dyn View>,
    width: Option<f32>,
    height: Option<f32>,
    alignment: Alignment,
}

impl View for Framed {
    fn build(&self, tree: &mut ViewTree) -> NodeId {
        let child = self.inner.build(tree);
        tree.add_node(
            Box::new(FrameLayout {
                width: self.width,
                height: self.height,
                alignment: self.alignment,
            }),
            vec![child],
        )
    }
}

struct FrameLayout {
    width: Option<f32>,
    height: Option<f32>,
    alignment: Alignment,
}

impl NodeLayout for FrameLayout {
    fn child_constraints(&self, _index: usize, constraints: &Constraints) -> Constraints {
        Constraints {
            min_width: 0.0,
            max_width: self.width.unwrap_or(constraints.max_width),
            min_height: 0.0,
            max_height: self.height.unwrap_or(constraints.max_height),
        }
    }

    fn measure(&self, child_sizes: &[Size], constraints: &Constraints) -> Size {
        let child = child_sizes.first().copied().unwrap_or(Size::ZERO);
        constraints.constrain(Size::new(
            self.width.unwrap_or(child.width),
            self.height.unwrap_or(child.height),
        ))
    }

    fn arrange(&self, child_sizes: &[Size], own_size: Size) -> Vec<Point> {
        let child = child_sizes.first().copied().unwrap_or(Size::ZERO);
        vec![Point::new(
            (own_size.width - child.width) * self.alignment.h.factor(),
            (own_size.height - child.height) * self.alignment.v.factor(),
        )]
    }
}
