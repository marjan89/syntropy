mod dispatcher;
pub mod events;
pub mod external_tui;
pub mod fuzzy_searcher;
pub mod key_bindings;
pub mod navigation;
mod screens;
mod strings;
mod tui_app;
pub mod views;

pub use external_tui::{
    ExternalTuiRequest, TuiRequestReceiver, TuiRequestSender, create_tui_channel, get_tui_sender,
    run_tui_command_blocking, set_tui_sender,
};
pub use tui_app::TuiApp;
