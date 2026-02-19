use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ScreenScaffold {
    pub left_split: u16,
    pub right_split: u16,
}

impl Default for ScreenScaffold {
    fn default() -> Self {
        Self {
            left_split: 50,
            right_split: 50,
        }
    }
}
