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
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Wrap};
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

fn draw_ui(frame: &mut ratatui::Frame, app: &App) {
    let area = frame.area();
    let theme = &app.config.theme;

    let chrome_border = parse_color(&theme.chrome.border);
    let chrome_title = parse_color(&theme.chrome.title);

    let border_type = match app.config.tui.borders {
        BorderStyle::None => ratatui::widgets::BorderType::Plain,
        BorderStyle::Plain => ratatui::widgets::BorderType::Plain,
        BorderStyle::Rounded => ratatui::widgets::BorderType::Rounded,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    draw_header(frame, chunks[0], app, chrome_border, chrome_title, border_type);
    draw_modules(frame, chunks[1], app, border_type);
    draw_footer(frame, chunks[2], app, chrome_border, border_type);
}

fn draw_header(
    frame: &mut ratatui::Frame,
    area: Rect,
    app: &App,
    border_color: ratatui::style::Color,
    title_color: ratatui::style::Color,
    border_type: ratatui::widgets::BorderType,
) {
    let mut tabs: Vec<Span> = Vec::new();
    if let Some(snapshot) = &app.snapshot {
        for (i, (name, _)) in snapshot.modules.iter().enumerate() {
            let module_color = module_fg_color(&app.config, name);
            if i == app.selected_tab {
                tabs.push(Span::styled(
                    format!(" [{}] ", name.to_uppercase()),
                    Style::default()
                        .fg(module_color)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                ));
            } else {
                tabs.push(Span::styled(
                    format!("  {}  ", name.to_uppercase()),
                    Style::default().fg(module_color),
                ));
            }
        }
    }

    let header = Paragraph::new(Line::from(tabs)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                " gim ",
                Style::default()
                    .fg(title_color)
                    .add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(header, area);
}

fn draw_footer(
    frame: &mut ratatui::Frame,
    area: Rect,
    app: &App,
    border_color: ratatui::style::Color,
    border_type: ratatui::widgets::BorderType,
) {
    if !app.config.tui.show_help {
        return;
    }

    let help = Paragraph::new(Line::from(vec![
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" quit  "),
        Span::styled("←/→", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" switch tab  "),
        Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" next  "),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color)),
    );
    frame.render_widget(help, area);
}

fn draw_modules(
    frame: &mut ratatui::Frame,
    area: Rect,
    app: &App,
    border_type: ratatui::widgets::BorderType,
) {
    let snapshot = match &app.snapshot {
        Some(s) => s,
        None => return,
    };

    if snapshot.modules.is_empty() {
        return;
    }

    let module_constraints: Vec<Constraint> = snapshot
        .modules
        .iter()
        .map(|_| Constraint::Ratio(1, snapshot.modules.len() as u32))
        .collect();

    let module_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(module_constraints)
        .split(area);

    for (i, (name, data)) in snapshot.modules.iter().enumerate() {
        let fg = module_fg_color(&app.config, name);
        let accent = module_accent_color(&app.config, name);
        let border_color = if i == app.selected_tab {
            fg
        } else {
            parse_color(&app.config.theme.chrome.border)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                format!(" {} ", module_label(&app.config, name)),
                Style::default().fg(fg).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(module_chunks[i]);
        frame.render_widget(block, module_chunks[i]);

        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(inner);

        if let Some(gauge_data) = extract_gauge(name, data) {
            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(accent))
                .ratio(gauge_data.ratio.clamp(0.0, 1.0))
                .label(format!(
                    "{}: {:.1}%",
                    gauge_data.label,
                    gauge_data.ratio * 100.0
                ));
            frame.render_widget(gauge, inner_chunks[0]);
        }

        let mut lines: Vec<Line> = Vec::new();
        let mut entries: Vec<_> = data.metrics.iter().collect();
        entries.sort_by_key(|(k, _)| k.clone());

        for (key, value) in entries {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{}: ", key),
                    Style::default().fg(fg).add_modifier(Modifier::BOLD),
                ),
                Span::raw(metric_display(value)),
            ]));
        }

        let detail = Paragraph::new(lines).wrap(Wrap { trim: true });
        frame.render_widget(detail, inner_chunks[1]);
    }
}

struct GaugeData {
    label: String,
    ratio: f64,
}

fn extract_gauge(module_name: &str, data: &crate::core::MetricData) -> Option<GaugeData> {
    let key = match module_name {
        "cpu" => "cpu_usage_percent",
        "memory" => "memory_usage_percent",
        "disk" => "usage_percent",
        _ => return None,
    };

    data.metrics.get(key).and_then(|v| match v {
        MetricValue::Float(f) => Some(GaugeData {
            label: module_name.to_uppercase(),
            ratio: *f / 100.0,
        }),
        _ => None,
    })
}

fn metric_display(value: &MetricValue) -> String {
    match value {
        MetricValue::Integer(i) => format_bytes_smart(*i),
        MetricValue::Float(f) => format!("{:.2}", f),
        MetricValue::String(s) => s.clone(),
        MetricValue::Boolean(b) => b.to_string(),
        MetricValue::List(items) => items
            .iter()
            .map(metric_display)
            .collect::<Vec<_>>()
            .join(", "),
    }
}

fn format_bytes_smart(value: i64) -> String {
    let abs = value.unsigned_abs();
    if abs >= 1_073_741_824 {
        format!("{:.2} GB", abs as f64 / 1_073_741_824.0)
    } else if abs >= 1_048_576 {
        format!("{:.2} MB", abs as f64 / 1_048_576.0)
    } else if abs >= 1024 {
        format!("{:.2} KB", abs as f64 / 1024.0)
    } else {
        value.to_string()
    }
}

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
