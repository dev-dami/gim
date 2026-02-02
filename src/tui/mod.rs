use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

use crate::config::{parse_color, BorderStyle, Config, ModuleTheme};
use crate::core::MetricValue;
use crate::engine::{Engine, MetricsSnapshot};
use crate::error::Result;

pub struct App {
    engine: Engine,
    config: Config,
    snapshot: Option<MetricsSnapshot>,
    selected_tab: usize,
    should_quit: bool,
}

impl App {
    pub fn new(engine: Engine, config: Config) -> Self {
        Self {
            engine,
            config,
            snapshot: None,
            selected_tab: 0,
            should_quit: false,
        }
    }

    fn refresh(&mut self) {
        self.snapshot = Some(self.engine.collect_once());
    }

    fn tab_count(&self) -> usize {
        self.snapshot
            .as_ref()
            .map(|s| s.modules.len())
            .unwrap_or(0)
    }
}

pub fn run_tui(engine: Engine, config: Config) -> Result<()> {
    enable_raw_mode().map_err(|e| crate::error::GimError::Tui(e.to_string()))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| crate::error::GimError::Tui(e.to_string()))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal =
        Terminal::new(backend).map_err(|e| crate::error::GimError::Tui(e.to_string()))?;

    let mut app = App::new(engine, config.clone());
    app.refresh();

    let refresh_dur = Duration::from_millis(config.tui_refresh_ms());
    let mut last_refresh = Instant::now();

    loop {
        terminal
            .draw(|frame| draw_ui(frame, &app))
            .map_err(|e| crate::error::GimError::Tui(e.to_string()))?;

        let timeout = refresh_dur.saturating_sub(last_refresh.elapsed());
        if event::poll(timeout).map_err(|e| crate::error::GimError::Tui(e.to_string()))? {
            if let Event::Key(key) = event::read().map_err(|e| crate::error::GimError::Tui(e.to_string()))? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                        KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                            let count = app.tab_count();
                            if count > 0 {
                                app.selected_tab = (app.selected_tab + 1) % count;
                            }
                        }
                        KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                            let count = app.tab_count();
                            if count > 0 {
                                app.selected_tab =
                                    app.selected_tab.checked_sub(1).unwrap_or(count - 1);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }

        if last_refresh.elapsed() >= refresh_dur {
            app.refresh();
            last_refresh = Instant::now();
        }
    }

    disable_raw_mode().map_err(|e| crate::error::GimError::Tui(e.to_string()))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .map_err(|e| crate::error::GimError::Tui(e.to_string()))?;
    terminal
        .show_cursor()
        .map_err(|e| crate::error::GimError::Tui(e.to_string()))?;

    Ok(())
}

fn draw_ui(_frame: &mut ratatui::Frame, _app: &App) {}

fn module_theme<'a>(config: &'a Config, name: &str) -> &'a ModuleTheme {
    match name {
        "cpu" => &config.theme.cpu,
        "memory" => &config.theme.memory,
        "disk" => &config.theme.disk,
        "network" => &config.theme.network,
        "process" => &config.theme.process,
        "system" => &config.theme.system,
        _ => &config.theme.cpu,
    }
}

fn module_fg_color(config: &Config, name: &str) -> ratatui::style::Color {
    parse_color(&module_theme(config, name).fg)
}

fn module_accent_color(config: &Config, name: &str) -> ratatui::style::Color {
    parse_color(&module_theme(config, name).accent)
}

fn module_label(config: &Config, name: &str) -> String {
    let label = &module_theme(config, name).label;
    if label.is_empty() {
        name.to_uppercase()
    } else {
        label.clone()
    }
}
