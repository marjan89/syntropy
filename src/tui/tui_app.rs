use crate::{
    app::App,
    execution::clamp_exit_code,
    tui::{
        ExternalTuiRequest, TuiRequestReceiver, create_tui_channel,
        dispatcher::ScreenDispatcher,
        events::{InputEvent, handle_key},
        key_bindings::ParsedKeyBindings,
        navigation::{Intent, ItemPayload, Navigator, PluginPayload, Route, TaskPayload},
        run_tui_command_blocking,
        screens::{ItemListScreen, PluginListScreen, TaskListScreen},
        set_tui_sender,
        views::{SearchBar, StatusBar, Styles},
    },
};
use anyhow::{Context, Result, ensure};
use crossterm::{
    cursor::Show,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::Block,
};
use std::{
    io,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::runtime::Handle as RuntimeHandle;

const SECOND_IN_MILLIS: u64 = 1000;
const RENDER_FPS: u64 = 30;
const ANIMATION_KEY_FRAMES_PER_SECOND: u64 = 10;
const MILLIS_PER_KEYFRAME: u64 = SECOND_IN_MILLIS / ANIMATION_KEY_FRAMES_PER_SECOND;
const BAR_HEIGHT: u16 = 3;

pub struct TuiApp {
    app: App,
    navigator: Navigator,
    should_quit: bool,
    keybindings: ParsedKeyBindings,
    styles: Styles,
    screen_dispatcher: ScreenDispatcher,
    status_bar: StatusBar,
    search_bar: SearchBar,
    tui_rx: TuiRequestReceiver,
}

impl TuiApp {
    pub fn new(app: App, runtime_handle: RuntimeHandle) -> Result<Self> {
        let keybindings = ParsedKeyBindings::from(&app.config.keybindings)?;

        let initial_route = Self::resolve_initial_route(&app)?;
        let route_name = Self::get_route_name(&initial_route, &app);

        let navigator = Navigator::new(
            initial_route,
            route_name,
            app.config.styles.status.breadcrumbs_separator.clone(),
        );
        let styles = Styles::try_from(&app.config.styles)?;
        let screen_dispatcher = ScreenDispatcher {
            plugin_screen: PluginListScreen::new(app.config.show_preview_pane),
            task_screen: TaskListScreen::new(
                runtime_handle.clone(),
                &app.lua_runtime,
                app.config.show_preview_pane,
            ),
            item_screen: ItemListScreen::new(
                runtime_handle.clone(),
                &app.lua_runtime,
                app.config.show_preview_pane,
            ),
        };

        let status_bar = StatusBar::default();
        let search_bar = SearchBar::default();

        // Create TUI command channel for external TUI applications (editors, file managers, etc.)
        let (tui_tx, tui_rx) = create_tui_channel();

        // Set global sender so Lua functions can request TUI suspension
        set_tui_sender(tui_tx)?;

        Ok(Self {
            app,
            navigator,
            should_quit: false,
            keybindings,
            styles,
            screen_dispatcher,
            status_bar,
            search_bar,
            tui_rx,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        self.screen_dispatcher
            .on_enter(self.navigator.current(), &self.app);

        loop {
            let breadcrumbs = self.navigator.get_breadcrumbs();
            let mut constraints: Vec<Constraint> = Vec::new();
            if self.app.config.search_bar {
                constraints.push(Constraint::Length(BAR_HEIGHT));
            }
            constraints.push(Constraint::Min(0));
            if self.app.config.status_bar {
                constraints.push(Constraint::Length(BAR_HEIGHT));
            }
            let screen_chunk = if self.app.config.search_bar { 1 } else { 0 };
            let status_bar_chunk = if self.app.config.search_bar { 2 } else { 1 };

            terminal.draw(|frame| {
                let background_block =
                    Block::default().style(Style::default().bg(self.styles.colors.background));
                frame.render_widget(background_block, frame.area());

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(constraints)
                    .split(frame.area());
                if self.app.config.search_bar {
                    self.search_bar.render(
                        frame,
                        chunks[0],
                        &self.styles.search_bar_style,
                        &self.styles.colors,
                    );
                }
                self.screen_dispatcher.render(
                    self.navigator.current(),
                    chunks[screen_chunk],
                    frame,
                    &self.styles,
                );
                let status = self.screen_dispatcher.get_status(self.navigator.current());
                if self.app.config.status_bar {
                    self.status_bar.render(
                        frame,
                        status,
                        breadcrumbs,
                        get_key_frame(),
                        chunks[status_bar_chunk],
                        &self.styles.status,
                        &self.styles.colors,
                    );
                }
            })?;
            self.update_screens();

            // Check for external TUI requests (imperative: handle immediately)
            if let Ok(request) = self.tui_rx.try_recv() {
                self.suspend_and_run_tui(request, &mut terminal)?;
                continue; // Skip poll_events, go straight to next render
            }

            self.poll_events()?;

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn poll_events(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(
            SECOND_IN_MILLIS.div_euclid(RENDER_FPS),
        ))? {
            let event = event::read()?;
            if self.app.config.search_bar && self.search_bar.handle_event(&event) {
                self.screen_dispatcher
                    .on_search(self.navigator.current(), self.search_bar.value());
                return Ok(());
            }

            if let Event::Key(key) = event {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.should_quit = true;
                    return Ok(());
                }

                if let Some(input_event) = handle_key(&key, &self.keybindings) {
                    self.handle_event(input_event);
                }
            }
        }
        Ok(())
    }

    fn update_screens(&mut self) {
        let intent = self
            .screen_dispatcher
            .update(self.navigator.current(), &self.app);

        if matches!(intent, Intent::Quit) {
            self.should_quit = true;
        }
    }

    fn handle_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::Back => {
                if self
                    .screen_dispatcher
                    .consumed_event(self.navigator.current(), &event)
                {
                    let _ = self.screen_dispatcher.handle_event(
                        self.navigator.current(),
                        event,
                        &self.app,
                    );
                } else if let Some(popped_stack_entry) = self.navigator.pop() {
                    self.search_bar.clear();
                    self.screen_dispatcher.on_exit(&popped_stack_entry.route);
                    self.screen_dispatcher
                        .on_enter(self.navigator.current(), &self.app);
                } else {
                    self.should_quit = true;
                }
            }
            _ => {
                let intent =
                    self.screen_dispatcher
                        .handle_event(self.navigator.current(), event, &self.app);

                if let Some(new_route) = self.navigator.resolve_intent(intent) {
                    self.search_bar.clear();
                    self.screen_dispatcher.on_exit(self.navigator.current());
                    let route_name = Self::get_route_name(&new_route, &self.app);
                    self.navigator.push(new_route, route_name);
                    self.screen_dispatcher
                        .on_enter(self.navigator.current(), &self.app);
                }
            }
        }
    }

    fn resolve_initial_route(app: &App) -> Result<Route> {
        if let Some(default_plugin_name) = &app.config.default_plugin {
            let plugin_idx = app
                .plugins
                .iter()
                .position(|p| p.metadata.name == *default_plugin_name)
                .with_context(|| {
                    format!(
                        "default_plugin '{}' not found in loaded plugins",
                        default_plugin_name
                    )
                })?;

            if let Some(default_task_key) = &app.config.default_task {
                let plugin = app.get_plugin(plugin_idx).with_context(|| {
                    format!(
                        "default_plugin '{}' not found in loaded plugins",
                        default_plugin_name
                    )
                })?;

                ensure!(
                    plugin.tasks.contains_key(default_task_key),
                    "default_task '{}' not found in plugin '{}' (available tasks: {})",
                    default_task_key,
                    default_plugin_name,
                    plugin
                        .tasks
                        .keys()
                        .map(|k| k.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                Ok(Route::Item {
                    payload: ItemPayload {
                        plugin_idx,
                        task_key: default_task_key.clone(),
                    },
                })
            } else {
                Ok(Route::Task {
                    payload: TaskPayload { plugin_idx },
                })
            }
        } else {
            Ok(Route::Plugin {
                payload: PluginPayload {},
            })
        }
    }

    fn get_route_name(route: &Route, app: &App) -> String {
        match route {
            Route::Plugin { .. } => route.to_string(),
            Route::Task { payload } => app
                .plugins
                .get(payload.plugin_idx)
                .map(|p| p.metadata.name.clone())
                .unwrap_or_else(|| route.to_string()),
            Route::Item { payload } => app
                .plugins
                .get(payload.plugin_idx)
                .and_then(|p| p.tasks.get(&payload.task_key))
                .map(|t| t.name.clone())
                .unwrap_or_else(|| route.to_string()),
        }
    }

    fn suspend_and_run_tui(
        &mut self,
        request: ExternalTuiRequest,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<()> {
        // Suspend TUI: disable raw mode and leave alternate screen
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

        // Run external TUI command in blocking mode (gives it full terminal control)
        let exit_code = run_tui_command_blocking(&request.command, &request.args)
            .unwrap_or_else(|_| clamp_exit_code(-1));

        // Restore TUI: re-enter alternate screen and enable raw mode
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        enable_raw_mode()?;

        // Clear terminal immediately (imperative, not deferred)
        terminal.clear()?;

        // Send response back to waiting Lua function
        let _ = request.response.send(exit_code);

        Ok(())
    }
}

impl Drop for TuiApp {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            Show
        );
    }
}

fn get_key_frame() -> u64 {
    let system_time_in_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_millis() as u64;
    system_time_in_millis / MILLIS_PER_KEYFRAME
}
