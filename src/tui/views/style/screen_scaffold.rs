use crate::configs::style::ScreenScaffold;

pub struct ScreenScaffoldStyle {
    pub left_split: u16,
    pub right_split: u16,
}

impl From<&ScreenScaffold> for ScreenScaffoldStyle {
    fn from(status_style: &ScreenScaffold) -> Self {
        Self {
            left_split: status_style.left_split,
            right_split: status_style.right_split,
        }
    }
}
