use radiacode_nuclides::{GammaSource, filter_sources, gamma_sources, source_by_key};

pub struct SourcesState {
    pub query: String,
    pub results: Vec<&'static GammaSource>,
    pub selected: Option<&'static GammaSource>,
    pub hovered_line: Option<usize>,
    pub log_scale: bool,
    pub overlay_measured: bool,
    pub pane_open: bool,
    pub pending_list_scroll: bool,
}

impl SourcesState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: filter_sources(""),
            selected: gamma_sources().next(),
            hovered_line: None,
            log_scale: false,
            overlay_measured: true,
            pane_open: true,
            pending_list_scroll: false,
        }
    }

    pub fn refresh_results(&mut self) {
        self.results = filter_sources(&self.query);
    }

    pub fn select(&mut self, source: &'static GammaSource) {
        if self.selected.map(|current| current.key) != Some(source.key) {
            self.hovered_line = None;
        }
        self.selected = Some(source);
    }

    pub fn reveal(&mut self, key: &str) {
        let Some(source) = source_by_key(key) else {
            return;
        };
        if !self.lists(source) {
            self.query.clear();
            self.refresh_results();
        }
        self.select(source);
        self.pending_list_scroll = true;
    }

    pub fn is_selected(&self, source: &GammaSource) -> bool {
        self.selected
            .is_some_and(|current| current.key == source.key)
    }

    fn lists(&self, source: &GammaSource) -> bool {
        self.results.iter().any(|entry| entry.key == source.key)
    }
}

impl Default for SourcesState {
    fn default() -> Self {
        Self::new()
    }
}
