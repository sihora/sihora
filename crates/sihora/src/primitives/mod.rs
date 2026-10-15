//! Headless primitive views: stacks, sized boxes, spacers, and interaction helpers.

use crate::core::{NodeId, NodeLayout, View, ViewTree};
use crate::layout::{Alignment, Constraints, HAlign, Point, Size, VAlign};

/// Test whether point `(px, py)` lies inside axis-aligned rect `(x, y, w, h)`.
pub fn hit_test(px: f32, py: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
    px >= x && px <= x + w && py >= y && py <= y + h
}

// ---------------------------------------------------------------------------
// SizedBox — leaf with fixed intrinsic size
// ---------------------------------------------------------------------------

/// A leaf view with a fixed intrinsic size (useful for testing and spacing).
pub struct SizedBox {
    width: f32,
    height: f32,
}

impl SizedBox {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl View for SizedBox {
    fn build(&self, tree: &mut ViewTree) -> NodeId {
        tree.add_node(
            Box::new(SizedBoxLayout {
                width: self.width,
                height: self.height,
            }),
            vec![],
        )
    }
}

struct SizedBoxLayout {
    width: f32,
    height: f32,
}

impl NodeLayout for SizedBoxLayout {
    fn measure(&self, _child_sizes: &[Size], constraints: &Constraints) -> Size {
        constraints.constrain(Size::new(self.width, self.height))
    }

    fn arrange(&self, _child_sizes: &[Size], _own_size: Size) -> Vec<Point> {
        Vec::new()
    }
}

// ---------------------------------------------------------------------------
// VStack
// ---------------------------------------------------------------------------

/// Arranges children in a vertical column.
pub struct VStack {
    spacing: f32,
    alignment: HAlign,
    children: Vec<Box<dyn View>>,
}

impl VStack {
    pub fn new() -> Self {
        Self {
            spacing: 0.0,
            alignment: HAlign::Center,
            children: Vec::new(),
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn alignment(mut self, alignment: HAlign) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn child(mut self, view: impl View + 'static) -> Self {
        self.children.push(Box::new(view));
        self
    }
}

impl Default for VStack {
    fn default() -> Self {
        Self::new()
    }
}

impl View for VStack {
    fn build(&self, tree: &mut ViewTree) -> NodeId {
        let child_ids: Vec<NodeId> = self.children.iter().map(|c| c.build(tree)).collect();
        tree.add_node(
            Box::new(VStackLayout {
                spacing: self.spacing,
                alignment: self.alignment,
            }),
            child_ids,
        )
    }
}

struct VStackLayout {
    spacing: f32,
    alignment: HAlign,
}

impl NodeLayout for VStackLayout {
    fn child_constraints(&self, _index: usize, constraints: &Constraints) -> Constraints {
        Constraints {
            min_width: constraints.min_width,
            max_width: constraints.max_width,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }

    fn measure(&self, child_sizes: &[Size], constraints: &Constraints) -> Size {
        let gap = if child_sizes.len() > 1 {
            self.spacing * (child_sizes.len() - 1) as f32
        } else {
            0.0
        };
        let width = child_sizes.iter().map(|s| s.width).fold(0.0_f32, f32::max);
        let height: f32 = child_sizes.iter().map(|s| s.height).sum::<f32>() + gap;
        constraints.constrain(Size::new(width, height))
    }

    fn arrange(&self, child_sizes: &[Size], own_size: Size) -> Vec<Point> {
        let mut y = 0.0;
        child_sizes
            .iter()
            .map(|size| {
                let x = (own_size.width - size.width) * self.alignment.factor();
                let pt = Point::new(x, y);
                y += size.height + self.spacing;
                pt
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// HStack
// ---------------------------------------------------------------------------

/// Arranges children in a horizontal row.
pub struct HStack {
    spacing: f32,
    alignment: VAlign,
    children: Vec<Box<dyn View>>,
}

impl HStack {
    pub fn new() -> Self {
        Self {
            spacing: 0.0,
            alignment: VAlign::Center,
            children: Vec::new(),
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn alignment(mut self, alignment: VAlign) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn child(mut self, view: impl View + 'static) -> Self {
        self.children.push(Box::new(view));
        self
    }
}

impl Default for HStack {
    fn default() -> Self {
        Self::new()
    }
}

impl View for HStack {
    fn build(&self, tree: &mut ViewTree) -> NodeId {
        let child_ids: Vec<NodeId> = self.children.iter().map(|c| c.build(tree)).collect();
        tree.add_node(
            Box::new(HStackLayout {
                spacing: self.spacing,
                alignment: self.alignment,
            }),
            child_ids,
        )
    }
}

struct HStackLayout {
    spacing: f32,
    alignment: VAlign,
}

impl NodeLayout for HStackLayout {
    fn child_constraints(&self, _index: usize, constraints: &Constraints) -> Constraints {
        Constraints {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: constraints.min_height,
            max_height: constraints.max_height,
        }
    }

    fn measure(&self, child_sizes: &[Size], constraints: &Constraints) -> Size {
        let gap = if child_sizes.len() > 1 {
            self.spacing * (child_sizes.len() - 1) as f32
        } else {
            0.0
        };
        let width: f32 = child_sizes.iter().map(|s| s.width).sum::<f32>() + gap;
        let height = child_sizes.iter().map(|s| s.height).fold(0.0_f32, f32::max);
        constraints.constrain(Size::new(width, height))
    }

    fn arrange(&self, child_sizes: &[Size], own_size: Size) -> Vec<Point> {
        let mut x = 0.0;
        child_sizes
            .iter()
            .map(|size| {
                let y = (own_size.height - size.height) * self.alignment.factor();
                let pt = Point::new(x, y);
                x += size.width + self.spacing;
                pt
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// ZStack
// ---------------------------------------------------------------------------

/// Overlays children on top of each other.
pub struct ZStack {
    alignment: Alignment,
    children: Vec<Box<dyn View>>,
}

impl ZStack {
    pub fn new() -> Self {
        Self {
            alignment: Alignment::CENTER,
            children: Vec::new(),
        }
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn child(mut self, view: impl View + 'static) -> Self {
        self.children.push(Box::new(view));
        self
    }
}

impl Default for ZStack {
    fn default() -> Self {
        Self::new()
    }
}

impl View for ZStack {
    fn build(&self, tree: &mut ViewTree) -> NodeId {
        let child_ids: Vec<NodeId> = self.children.iter().map(|c| c.build(tree)).collect();
        tree.add_node(Box::new(ZStackLayout(self.alignment)), child_ids)
    }
}

struct ZStackLayout(Alignment);

impl NodeLayout for ZStackLayout {
    fn measure(&self, child_sizes: &[Size], constraints: &Constraints) -> Size {
        let width = child_sizes.iter().map(|s| s.width).fold(0.0_f32, f32::max);
        let height = child_sizes.iter().map(|s| s.height).fold(0.0_f32, f32::max);
        constraints.constrain(Size::new(width, height))
    }

    fn arrange(&self, child_sizes: &[Size], own_size: Size) -> Vec<Point> {
        child_sizes
            .iter()
            .map(|size| {
                Point::new(
                    (own_size.width - size.width) * self.0.h.factor(),
                    (own_size.height - size.height) * self.0.v.factor(),
                )
            })
            .collect()
    }
}
