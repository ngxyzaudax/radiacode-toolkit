use radiacode_nuclides::{ChainFilters, NuclideId, search_chains};

pub struct ChainBrowseState {
    pub filters: ChainFilters,
    pub results: Vec<usize>,
    pub selected: Option<usize>,
    pub hovered_line: Option<usize>,
    pub hovered_member: Option<NuclideId>,
    pub pending_scroll: bool,
}

impl ChainBrowseState {
    pub fn new() -> Self {
        let mut state = Self {
            filters: ChainFilters::default(),
            results: Vec::new(),
            selected: None,
            hovered_line: None,
            hovered_member: None,
            pending_scroll: false,
        };
        state.refresh_results();
        state
    }

    pub fn refresh_results(&mut self) {
        self.results = search_chains(&self.filters);
        if let Some(selected) = self.selected {
            if !self.results.contains(&selected) {
                self.selected = None;
            }
        }
    }

    pub fn select(&mut self, index: usize) {
        self.selected = Some(index);
        self.hovered_line = None;
        self.hovered_member = None;
    }

    pub fn reveal(&mut self, index: usize) {
        self.select(index);
        self.pending_scroll = true;
    }

    pub fn clear_selection(&mut self) {
        self.selected = None;
        self.hovered_line = None;
        self.hovered_member = None;
        self.pending_scroll = false;
    }
}

impl Default for ChainBrowseState {
    fn default() -> Self {
        Self::new()
    }
}
