use std::fmt::Display;

use crate::tui::strings::StatusStrings;

#[derive(Default)]
pub enum Status {
    #[default]
    Idle,
    Error,
    Running,
    Complete,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Idle => write!(f, "{}", StatusStrings::IDLE),
            Status::Error => write!(f, "{}", StatusStrings::ERROR),
            Status::Running => write!(f, "{}", StatusStrings::RUNNING),
            Status::Complete => write!(f, "{}", StatusStrings::COMPLETE),
        }
    }
}
