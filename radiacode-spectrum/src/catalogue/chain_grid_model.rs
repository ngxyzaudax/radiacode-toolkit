use egui::{Pos2, Rect, Vec2};

use radiacode_nuclides::{DecayMode, NuclideId};

pub const NODE_HEIGHT: f32 = 52.0;
pub const NODE_GAP: f32 = 16.0;
pub const MIN_COL_GAP: f32 = 108.0;
pub const PADDING: f32 = 24.0;
pub const MIN_NODE_WIDTH: f32 = 96.0;
pub const MAX_NODE_WIDTH: f32 = 148.0;
pub const NODE_PAD_X: f32 = 14.0;
pub const NAME_CHAR_WIDTH: f32 = 9.2;
pub const SUBTITLE_CHAR_WIDTH: f32 = 7.4;
pub const LABEL_CHAR_WIDTH: f32 = 10.0;
pub const LABEL_PAD: f32 = 36.0;
pub const LABEL_LANE_STEP: f32 = 22.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Focus,
    Parent,
    Stable,
    Absent,
    Normal,
}

#[derive(Debug, Clone)]
pub struct GridNode {
    pub graph_index: usize,
    pub nuclide_id: NuclideId,
    pub display_name: String,
    pub half_life_secs: Option<f64>,
    pub in_catalogue: bool,
    pub role: NodeRole,
    pub rect: Rect,
}

#[derive(Debug, Clone)]
pub struct GridEdge {
    pub mode: DecayMode,
    pub points: Vec<Pos2>,
    pub label_pos: Option<Pos2>,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChainGrid {
    pub nodes: Vec<GridNode>,
    pub edges: Vec<GridEdge>,
    pub size: Vec2,
    pub focus_rect: Rect,
}

impl ChainGrid {
    pub fn node_at(&self, point: Pos2, origin: Pos2) -> Option<usize> {
        let local = point - origin.to_vec2();
        self.nodes.iter().position(|node| node.rect.contains(local))
    }

    pub fn content_rect(&self) -> Rect {
        Rect::from_min_size(Pos2::ZERO, self.size)
    }
}
