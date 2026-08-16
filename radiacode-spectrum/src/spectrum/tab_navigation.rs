use crate::view_tab::ViewTab;

pub struct TabNavigation {
    pub active: ViewTab,
    pub previous: ViewTab,
    pub pending: Option<ViewTab>,
    pub pending_after_save: Option<ViewTab>,
    pub monitor_leave_open: bool,
}

impl TabNavigation {
    pub fn new() -> Self {
        Self {
            active: ViewTab::Device,
            previous: ViewTab::Device,
            pending: None,
            pending_after_save: None,
            monitor_leave_open: false,
        }
    }

    pub fn try_switch(&mut self, tab: ViewTab, draft_dirty: bool) -> bool {
        if self.active == ViewTab::Monitor && tab != ViewTab::Monitor && draft_dirty {
            self.pending = Some(tab);
            self.monitor_leave_open = true;
            return false;
        }
        self.active = tab;
        true
    }
}
