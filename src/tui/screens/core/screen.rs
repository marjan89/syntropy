use crate::{
    app::App,
    tui::{events::InputEvent, navigation::Intent, screens::core::status::Status, views::Styles},
};
use ratatui::{Frame, layout::Rect};

/// Screen trait providing a unified interface for all screen implementations.
///
/// Each screen combines Model + View + Controller in one self-contained unit, managing
/// its own state, rendering logic, and event handling. Screens follow a complete lifecycle:
///
/// **Lifecycle Flow**: `on_enter()` → `on_update()` → `handle_event()` → `render()` → `on_exit()`
///
/// # Type Parameters
///
/// * `T` - The payload type specific to this screen, enabling type-safe data passing between screens
///
/// # Examples
///
/// ```
/// // Example screen implementation pattern:
/// //
/// // struct PluginListScreen { /* ... */ }
/// //
/// // impl Screen<PluginPayload> for PluginListScreen {
/// //     fn on_enter(&mut self, app: &App, payload: &PluginPayload) {
/// //         // Initialize screen state from app and payload
/// //     }
/// //
/// //     fn on_exit(&mut self) {
/// //         // Cleanup when leaving screen
/// //     }
/// //
/// //     fn get_status(&mut self) -> Status {
/// //         Status::ready("Plugin List")
/// //     }
/// //
/// //     fn handle_event(&mut self, event: InputEvent, app: &App, payload: &PluginPayload) -> Intent {
/// //         // Process user input and return navigation intent
/// //         Intent::None
/// //     }
/// //
/// //     fn render(&mut self, frame: &mut Frame, area: Rect, app: &App, payload: &PluginPayload) {
/// //         // Render screen content
/// //     }
/// // }
/// ```
pub trait Screen<T> {
    /// Called when the screen becomes active in the navigation stack.
    ///
    /// This lifecycle hook is invoked when the screen is navigated to, allowing
    /// initialization of screen state, caching data from the app context, and
    /// preparing UI elements for rendering.
    ///
    /// # Parameters
    ///
    /// * `app` - Immutable reference to the application context (read-only access)
    /// * `payload` - Type-safe payload containing screen-specific data
    fn on_enter(&mut self, app: &App, payload: &T);

    /// Called after rendering but before event polling.
    ///
    /// This hook allows screens to perform their own polling actions and state updates
    /// between render cycles. Optional with default no-op implementation.
    ///
    /// # Parameters
    ///
    /// * `app` - Immutable reference to the application context
    /// * `payload` - Type-safe payload containing screen-specific data
    fn on_update(&mut self, _app: &App, _payload: &T) -> Intent {
        Intent::None
    }

    /// Returns a mutable reference to the screen's current status.
    ///
    /// Polled before drawing to fetch the status line that will be displayed
    /// in the UI. Screens manage their own status with caching to avoid
    /// unnecessary regeneration.
    ///
    /// # Returns
    ///
    /// Mutable reference to the screen's `Status` for the status bar
    fn get_status(&mut self) -> &mut Status;

    /// Called when the screen becomes inactive in the navigation stack.
    ///
    /// This lifecycle hook is invoked when navigating away from the screen,
    /// allowing cleanup of resources and state reset if needed.
    fn on_exit(&mut self);

    /// Handles input events and returns navigation intent.
    ///
    /// Processes user input (keyboard events) and returns an `Intent` describing
    /// the desired navigation action. The navigator resolves intents to concrete routes.
    ///
    /// # Parameters
    ///
    /// * `event` - The input event to handle (e.g., key press, navigation action)
    /// * `app` - Immutable reference to the application context
    /// * `payload` - Type-safe payload containing screen-specific data
    ///
    /// # Returns
    ///
    /// `Intent` enum indicating navigation action (e.g., `Intent::None`, `Intent::Back`, etc.)
    fn handle_event(&mut self, event: InputEvent, app: &App, payload: &T) -> Intent;

    /// Renders the screen to the terminal frame.
    ///
    /// Each screen owns its layout logic, including how it divides the available area
    /// for lists, previews, and other UI elements. Rendering occurs at 30 FPS in the
    /// main event loop.
    ///
    /// # Parameters
    ///
    /// * `frame` - Mutable reference to the ratatui frame for rendering
    /// * `area` - The rectangular area allocated for this screen
    /// * `styles` - Shared style configuration for consistent UI theming
    fn render(&mut self, frame: &mut Frame, area: Rect, styles: &Styles);

    /// Called when an updated search query is available
    ///
    /// This hook allows the screen to react to a search query and perform any necessary filtering.
    ///
    /// #Parameters
    ///
    /// * `query` - Search query to perform filtering with
    fn on_search(&mut self, query: &str);

    /// Called before app processes input events.
    ///
    /// App can return `true` if it wants to consume the event exclusively. Otherwise it can return
    /// `false` so that App will handle it as well.
    ///
    /// #Parameters
    ///
    /// * `event` - InputEvent pending to be handled
    fn consumed_event(&mut self, _event: &InputEvent) -> bool {
        false
    }
}
