use egui::Rect;
use radiacode_nuclides::NuclideId;

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
        Self::focused_on(None)
    }

    fn focused_on(focus: Option<NuclideId>) -> Self {
        Self {
            scene_rect: Rect::ZERO,
            last_focus: focus,
            needs_fit: true,
            fit_mode: ChainFitMode::Adaptive,
        }
    }

    pub fn refocus(&mut self, focus: NuclideId) {
        *self = Self::focused_on(Some(focus));
    }

    pub fn reset(&mut self) {
        *self = Self::focused_on(None);
    }

    pub fn sync_focus(&mut self, focus: NuclideId) {
        if self.last_focus != Some(focus) {
            self.refocus(focus);
        }
    }
}
