use egui::Rect;
use radiacode_nuclides::{NuclideId, SearchFilters, catalog, nuclide_index, search_nuclides};

use crate::catalogue::browse_mode::CatalogueMode;
use crate::catalogue::chain_state::ChainBrowseState;
use crate::catalogue::chain_view_cache::ChainViewCache;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChainFitMode {
    #[default]
    Adaptive,
    All,
    Focus,
}

pub struct ChainSceneState {
    pub scene_rect: Rect,
    pub last_focus: Option<NuclideId>,
    pub needs_fit: bool,
    pub fit_mode: ChainFitMode,
}

impl ChainSceneState {
    pub fn new() -> Self {
        Self {
            scene_rect: Rect::ZERO,
            last_focus: None,
            needs_fit: true,
            fit_mode: ChainFitMode::Adaptive,
        }
    }
}

pub struct CatalogueState {
    pub mode: CatalogueMode,
    pub filters: SearchFilters,
    pub results: Vec<usize>,
    pub selected: Option<NuclideId>,
    pub chains: ChainBrowseState,
    pub hovered_gamma: Option<usize>,
    pub hovered_chain_node: Option<NuclideId>,
    pub preview_log_scale: bool,
    pub chain_view: Option<ChainViewCache>,
    pub chain_scene: ChainSceneState,
    pub pane_open: bool,
    pub chain_collapsed: bool,
    pub pending_list_scroll: bool,
}

impl CatalogueState {
    pub fn new() -> Self {
        let mut state = Self {
            mode: CatalogueMode::Nuclides,
            filters: SearchFilters::default(),
            results: Vec::new(),
            selected: None,
            chains: ChainBrowseState::new(),
            hovered_gamma: None,
            hovered_chain_node: None,
            preview_log_scale: false,
            chain_view: None,
            chain_scene: ChainSceneState::new(),
            pane_open: true,
            chain_collapsed: false,
            pending_list_scroll: false,
        };
        state.refresh_results();
        state
    }

    pub fn refresh_results(&mut self) {
        self.results = search_nuclides(&self.filters);
        if let Some(selected) = self.selected {
            let still_visible = self
                .results
                .iter()
                .any(|&index| catalog().nuclides[index].id == selected);
            if !still_visible {
                self.selected = None;
            }
        }
    }

    pub fn sync_chain_scene_focus(&mut self, focus: NuclideId) {
        if self.chain_scene.last_focus != Some(focus) {
            self.chain_scene.scene_rect = Rect::ZERO;
            self.chain_scene.last_focus = Some(focus);
            self.chain_scene.needs_fit = true;
            self.chain_scene.fit_mode = ChainFitMode::Adaptive;
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected = None;
        self.hovered_gamma = None;
        self.hovered_chain_node = None;
        self.chain_view = None;
        self.chain_scene.scene_rect = Rect::ZERO;
        self.chain_scene.last_focus = None;
        self.chain_scene.needs_fit = true;
        self.chain_scene.fit_mode = ChainFitMode::Adaptive;
        self.pending_list_scroll = false;
    }

    pub fn select(&mut self, id: NuclideId) {
        self.apply_selection(id);
    }

    pub fn reveal(&mut self, id: NuclideId) {
        self.apply_selection(id);
        self.ensure_visible(id);
        self.pending_list_scroll = true;
    }

    fn apply_selection(&mut self, id: NuclideId) {
        if self.selected != Some(id) {
            self.chain_view = None;
            self.chain_scene.scene_rect = Rect::ZERO;
            self.chain_scene.last_focus = Some(id);
            self.chain_scene.needs_fit = true;
            self.chain_scene.fit_mode = ChainFitMode::Adaptive;
        }
        self.selected = Some(id);
        self.hovered_gamma = None;
    }

    fn ensure_visible(&mut self, id: NuclideId) {
        let Some(index) = nuclide_index(id) else {
            return;
        };
        if self.results.contains(&index) {
            return;
        }
        let insert_at = self
            .results
            .iter()
            .position(|&existing| existing > index)
            .unwrap_or(self.results.len());
        self.results.insert(insert_at, index);
    }

    pub fn on_tab_enter(&mut self) {
        self.refresh_results();
        self.chains.refresh_results();
    }

    pub fn select_chain_by_head(&mut self, head: NuclideId) {
        let Some(index) = radiacode_nuclides::chain_series()
            .iter()
            .position(|series| series.head == head)
        else {
            return;
        };
        self.mode = CatalogueMode::Chains;
        self.chains.reveal(index);
    }
}

impl Default for CatalogueState {
    fn default() -> Self {
        Self::new()
    }
}
