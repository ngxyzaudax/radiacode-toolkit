use std::cell::RefCell;
use std::rc::Rc;

pub type SnapLabel = Rc<RefCell<Option<String>>>;

pub fn snap_label() -> SnapLabel {
    Rc::new(RefCell::new(None))
}

pub fn set_snap_label(label: &SnapLabel, text: String) {
    *label.borrow_mut() = Some(text);
}

pub fn override_hover(label: &SnapLabel, fallback: Option<String>) -> Option<String> {
    label.borrow_mut().take().or(fallback)
}
