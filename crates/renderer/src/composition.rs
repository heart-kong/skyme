//! DirectComposition visual tree integration.
//!
//! Manages the composition visual tree for the candidate window.

use skyme_common::Rect;

/// A node in the DirectComposition visual tree.
pub struct VisualNode {
    pub id: u64,
    pub bounds: Rect,
    pub opacity: f32,
    pub visible: bool,
}

impl VisualNode {
    pub fn new(id: u64) -> Self {
        Self { id, bounds: Rect::default(), opacity: 1.0, visible: true }
    }
    pub fn set_bounds(&mut self, rect: &Rect) { self.bounds = *rect; }
    pub fn set_opacity(&mut self, opacity: f32) { self.opacity = opacity.clamp(0.0, 1.0); }
    pub fn show(&mut self) { self.visible = true; }
    pub fn hide(&mut self) { self.visible = false; }
}

/// Manages the composition tree for the input method's UI.
pub struct CompositionTree {
    root: VisualNode,
    children: Vec<VisualNode>,
}

impl CompositionTree {
    pub fn new() -> Self { Self { root: VisualNode::new(0), children: Vec::new() } }
    pub fn root(&self) -> &VisualNode { &self.root }
    pub fn root_mut(&mut self) -> &mut VisualNode { &mut self.root }
    pub fn add_child(&mut self, node: VisualNode) { self.children.push(node); }
    pub fn children(&self) -> &[VisualNode] { &self.children }
}

impl Default for CompositionTree { fn default() -> Self { Self::new() } }
