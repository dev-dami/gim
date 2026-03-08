use std::collections::HashMap;
use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Sparkline, Wrap};
use ratatui::Terminal;

use crate::config::{parse_color, BorderStyle, Config, ModuleTheme, ThemeMode};
use crate::core::MetricValue;
use crate::engine::{Engine, MetricsSnapshot};
use crate::error::Result;

// ── Data types ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum ViewMode {
    Dashboard,
    ZoomIn(usize),
}

#[derive(Debug, Clone)]
struct ProcessEntry {
    pid: u32,
    name: String,
    cpu: f32,
    memory: u64,
    command: String,
}

pub struct App {
    engine: Engine,
    config: Config,
    snapshot: Option<MetricsSnapshot>,
    selected_tab: usize,
    should_quit: bool,
    history: HashMap<String, Vec<f64>>,
    view_mode: ViewMode,
    search_query: String,
    search_active: bool,
    filtered_processes: Vec<ProcessEntry>,
    process_selected: usize,
    all_processes: Vec<ProcessEntry>,
}

impl App {
    pub fn new(engine: Engine, config: Config) -> Self {
        Self {
            engine,
            config,
            snapshot: None,
            selected_tab: 0,
            should_quit: false,
            history: HashMap::new(),
            view_mode: ViewMode::Dashboard,
            search_query: String::new(),
            search_active: false,
            filtered_processes: Vec::new(),
            process_selected: 0,
            all_processes: Vec::new(),
        }
    }

    fn refresh(&mut self) {
        self.snapshot = Some(self.engine.collect_once());

        if let Some(snapshot) = &self.snapshot {
            for (name, data) in &snapshot.modules {
                if let Some(value) = extract_history_value(name, data) {
                    let entry = self.history.entry(name.clone()).or_default();
                    entry.push(value);
                    if entry.len() > 60 {
                        entry.remove(0);
                    }
                }
            }
        }
    }

    fn tab_count(&self) -> usize {
        self.snapshot.as_ref().map(|s| s.modules.len()).unwrap_or(0)
    }

    fn current_module_name(&self) -> Option<String> {
        self.snapshot
            .as_ref()
            .and_then(|s| s.modules.get(self.selected_tab))
            .map(|(name, _)| name.clone())
    }

    fn refresh_processes(&mut self) {
        use sysinfo::{ProcessRefreshKind, RefreshKind, System};
        let sys = System::new_with_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
        );
        self.all_processes = sys
            .processes()
            .values()
            .map(|p| ProcessEntry {
                pid: p.pid().as_u32(),
                name: p.name().to_string_lossy().to_string(),
                cpu: p.cpu_usage(),
                memory: p.memory(),
                command: p
                    .exe()
                    .map(|e| e.display().to_string())
                    .unwrap_or_default(),
            })
            .collect();
        self.all_processes.sort_by(|a, b| b.memory.cmp(&a.memory));
        self.filtered_processes = self.all_processes.clone();
        self.process_selected = 0;
    }

    fn apply_fuzzy_filter(&mut self) {
        use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
        use nucleo_matcher::{Config as NucleoConfig, Matcher, Utf32Str};

        if self.search_query.is_empty() {
            self.filtered_processes = self.all_processes.clone();
            self.process_selected = 0;
            return;
        }

        let mut matcher = Matcher::new(NucleoConfig::DEFAULT);
        let pattern =
            Pattern::parse(&self.search_query, CaseMatching::Smart, Normalization::Smart);
        let mut buf = Vec::new();

        let mut scored: Vec<(u32, ProcessEntry)> = self
            .all_processes
            .iter()
            .filter_map(|p| {
                let haystack = Utf32Str::new(&p.name, &mut buf);
                pattern
                    .score(haystack, &mut matcher)
                    .map(|score| (score, p.clone()))
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        self.filtered_processes = scored.into_iter().map(|(_, e)| e).collect();
        self.process_selected = 0;
    }
}

// ── Main loop ───────────────────────────────────────────────────────

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
            if let Event::Key(key) =
                event::read().map_err(|e| crate::error::GimError::Tui(e.to_string()))?
            {
                if key.kind == KeyEventKind::Press {
                    handle_key(&mut app, key.code);
                }
            }
        }

        if app.should_quit {
            break;
        }

        if last_refresh.elapsed() >= refresh_dur {
            app.refresh();
            // Also refresh processes if in process zoom view
            if let ViewMode::ZoomIn(idx) = &app.view_mode {
                if app
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.modules.get(*idx))
                    .map(|(n, _)| n == "process")
                    .unwrap_or(false)
                {
                    app.refresh_processes();
                    app.apply_fuzzy_filter();
                }
            }
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

// ── Input handling ──────────────────────────────────────────────────

fn handle_key(app: &mut App, code: KeyCode) {
    // If search is active, handle text input first
    if app.search_active {
        match code {
            KeyCode::Esc => {
                app.search_active = false;
                app.search_query.clear();
                app.filtered_processes = app.all_processes.clone();
                app.process_selected = 0;
            }
            KeyCode::Char(c) => {
                app.search_query.push(c);
                app.apply_fuzzy_filter();
            }
            KeyCode::Backspace => {
                app.search_query.pop();
                app.apply_fuzzy_filter();
            }
            KeyCode::Down | KeyCode::Tab => {
                let max = app.filtered_processes.len().saturating_sub(1);
                app.process_selected = (app.process_selected + 1).min(max);
            }
            KeyCode::Up | KeyCode::BackTab => {
                app.process_selected = app.process_selected.saturating_sub(1);
            }
            _ => {}
        }
        return;
    }

    match &app.view_mode {
        ViewMode::Dashboard => match code {
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
                    app.selected_tab = app.selected_tab.checked_sub(1).unwrap_or(count - 1);
                }
            }
            KeyCode::Enter => {
                let is_process = app.current_module_name().as_deref() == Some("process");
                app.view_mode = ViewMode::ZoomIn(app.selected_tab);
                if is_process {
                    app.refresh_processes();
                    app.search_active = true;
                }
            }
            _ => {}
        },
        ViewMode::ZoomIn(idx) => {
            let is_process = app
                .snapshot
                .as_ref()
                .and_then(|s| s.modules.get(*idx))
                .map(|(n, _)| n == "process")
                .unwrap_or(false);

            match code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.view_mode = ViewMode::Dashboard;
                    app.search_query.clear();
                    app.search_active = false;
                }
                KeyCode::Char('/') | KeyCode::Char('f') if is_process => {
                    app.search_active = true;
                }
                KeyCode::Down | KeyCode::Char('j') if is_process => {
                    let max = app.filtered_processes.len().saturating_sub(1);
                    app.process_selected = (app.process_selected + 1).min(max);
                }
                KeyCode::Up | KeyCode::Char('k') if is_process => {
                    app.process_selected = app.process_selected.saturating_sub(1);
                }
                _ => {}
            }
        }
    }
}

// ── Drawing ─────────────────────────────────────────────────────────

fn draw_ui(frame: &mut ratatui::Frame, app: &App) {
    match &app.view_mode {
        ViewMode::Dashboard => draw_dashboard(frame, app),
        ViewMode::ZoomIn(idx) => {
            if let Some(snapshot) = &app.snapshot {
                if let Some((name, _)) = snapshot.modules.get(*idx) {
                    if name == "process" {
                        draw_process_finder(frame, app);
                    } else {
                        draw_module_detail(frame, app, *idx);
                    }
                }
            }
        }
    }
}

// ── Dashboard view ──────────────────────────────────────────────────

fn draw_dashboard(frame: &mut ratatui::Frame, app: &App) {
    let area = frame.area();
    let border_type = resolve_border_type(&app.config);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Min(10),   // modules
            Constraint::Length(3), // footer
        ])
        .split(area);

    draw_header(frame, chunks[0], app, border_type);
    draw_modules(frame, chunks[1], app, border_type);
    draw_footer(frame, chunks[2], app, border_type);
}

fn draw_header(
    frame: &mut ratatui::Frame,
    area: Rect,
    app: &App,
    border_type: ratatui::widgets::BorderType,
) {
    let border_color = resolve_border(&app.config);
    let title_color = resolve_title(&app.config);

    let mut tabs: Vec<Span> = Vec::new();
    if let Some(snapshot) = &app.snapshot {
        for (i, (name, _)) in snapshot.modules.iter().enumerate() {
            let fg = resolve_module_fg(&app.config, name);
            let label = module_label(&app.config, name);
            if i == app.selected_tab {
                tabs.push(Span::styled(
                    format!(" [{}] ", label),
                    Style::default()
                        .fg(fg)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                ));
            } else {
                tabs.push(Span::styled(
                    format!("  {}  ", label),
                    Style::default().fg(fg),
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

    let num = snapshot.modules.len();
    let cols = if num <= 3 { num as u32 } else { 3 };
    let rows = (num as u32 + cols - 1) / cols;

    let row_constraints: Vec<Constraint> = (0..rows).map(|_| Constraint::Ratio(1, rows)).collect();
    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(area);

    let mut rects = Vec::new();
    for row_area in row_chunks.iter() {
        let col_constraints: Vec<Constraint> =
            (0..cols).map(|_| Constraint::Ratio(1, cols)).collect();
        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(*row_area);
        rects.extend(col_chunks.iter().copied());
    }

    let border_color = resolve_border(&app.config);

    for (i, (name, data)) in snapshot.modules.iter().enumerate() {
        let target = rects.get(i).copied().unwrap_or_default();
        if target.width == 0 || target.height == 0 {
            continue;
        }

        let fg = resolve_module_fg(&app.config, name);
        let accent = resolve_module_accent(&app.config, name);
        let panel_border = if i == app.selected_tab { fg } else { border_color };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(panel_border))
            .padding(ratatui::widgets::Padding::symmetric(1, 0))
            .title(Span::styled(
                format!(" {} ", module_label(&app.config, name)),
                Style::default().fg(fg).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(target);
        frame.render_widget(block, target);

        // Layout: gauge (if applicable) + sparkline + details
        let has_gauge = extract_gauge(name, data).is_some();
        let has_history = app
            .history
            .get(name)
            .map(|h| !h.is_empty())
            .unwrap_or(false);

        let mut constraints = Vec::new();
        if has_gauge {
            constraints.push(Constraint::Length(2));
        }
        if has_history {
            constraints.push(Constraint::Length(4));
        }
        constraints.push(Constraint::Min(1));

        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        let mut chunk_idx = 0;

        // Gauge
        if let Some(gauge_data) = extract_gauge(name, data) {
            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(accent).bg(Color::DarkGray))
                .use_unicode(true)
                .ratio(gauge_data.ratio.clamp(0.0, 1.0))
                .label(Span::styled(
                    format!("{:.1}%", gauge_data.ratio * 100.0),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            frame.render_widget(gauge, inner_chunks[chunk_idx]);
            chunk_idx += 1;
        }

        // Sparkline
        if has_history {
            if let Some(history) = app.history.get(name) {
                let data_vec: Vec<u64> = history.iter().map(|&v| v as u64).collect();
                let sparkline = Sparkline::default()
                    .data(data_vec)
                    .style(Style::default().fg(accent))
                    .max(100);
                frame.render_widget(sparkline, inner_chunks[chunk_idx]);
            }
            chunk_idx += 1;
        }

        // Details
        let detail_area = inner_chunks[chunk_idx];
        let mut lines: Vec<Line> = Vec::new();
        let mut entries: Vec<_> = data.metrics.iter().collect();
        entries.sort_by_key(|(k, _)| *k);

        for (key, value) in entries {
            lines.push(Line::from(vec![
                Span::styled(format!("{} ", key), Style::default().fg(fg)),
                Span::styled("→ ", Style::default().fg(Color::DarkGray)),
                Span::styled(metric_display(value), Style::default().fg(Color::White)),
            ]));
        }

        let detail = Paragraph::new(lines).wrap(Wrap { trim: true });
        frame.render_widget(detail, detail_area);
    }
}

fn draw_footer(
    frame: &mut ratatui::Frame,
    area: Rect,
    app: &App,
    border_type: ratatui::widgets::BorderType,
) {
    if !app.config.tui.show_help {
        return;
    }

    let border_color = resolve_border(&app.config);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(30), Constraint::Length(36)])
        .split(area);

    // Help keys
    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            "q",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ),
        Span::styled(" quit ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "←→",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ),
        Span::styled(" tab ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "⏎",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ),
        Span::styled(" zoom ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "esc",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ),
        Span::styled(" back", Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color)),
    );
    frame.render_widget(help, chunks[0]);

    // Color palette — terminal's ANSI 16 colors
    let palette_colors = [
        Color::Black,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::White,
        Color::DarkGray,
        Color::LightRed,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightCyan,
        Color::White,
    ];
    let spans: Vec<Span> = palette_colors
        .iter()
        .map(|&c| Span::styled("██", Style::default().fg(c)))
        .collect();

    let palette = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(border_type)
                .border_style(Style::default().fg(border_color)),
        );
    frame.render_widget(palette, chunks[1]);
}

// ── Process finder view ─────────────────────────────────────────────

fn draw_process_finder(frame: &mut ratatui::Frame, app: &App) {
    let area = frame.area();
    let border_color = resolve_border(&app.config);
    let accent = resolve_module_fg(&app.config, "process");
    let border_type = resolve_border_type(&app.config);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // search bar
            Constraint::Length(1), // column headers
            Constraint::Min(5),   // process list
            Constraint::Length(3), // footer
        ])
        .split(area);

    // Search bar
    let search_icon = if app.config.tui.icons { " " } else { ">" };
    let cursor = if app.search_active { "█" } else { "" };
    let search = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} {}", search_icon, app.search_query),
            Style::default().fg(Color::White),
        ),
        Span::styled(cursor, Style::default().fg(accent)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(if app.search_active {
                accent
            } else {
                border_color
            }))
            .title(Span::styled(
                if app.config.tui.icons {
                    "  Process Finder "
                } else {
                    " Process Finder "
                },
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(search, chunks[0]);

    // Column headers
    let headers = Paragraph::new(Line::from(vec![Span::styled(
        format!(
            " {:>7}  {:<20}  {:>6}  {:>10}  {}",
            "PID", "NAME", "CPU%", "MEM", "COMMAND"
        ),
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )]));
    frame.render_widget(headers, chunks[1]);

    // Process list
    let visible_height = chunks[2].height as usize;
    let scroll = if app.process_selected >= visible_height {
        app.process_selected - visible_height + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    for (i, proc_entry) in app
        .filtered_processes
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
    {
        let is_selected = i == app.process_selected;
        let mem_str = format_mem(proc_entry.memory);
        let line_str = format!(
            " {:>7}  {:<20}  {:>5.1}%  {:>10}  {}",
            proc_entry.pid,
            truncate_str(&proc_entry.name, 20),
            proc_entry.cpu,
            mem_str,
            truncate_str(&proc_entry.command, 40),
        );

        let style = if is_selected {
            Style::default()
                .bg(accent)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::styled(line_str, style));
    }

    let list_block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color));
    let process_list = Paragraph::new(lines).block(list_block);
    frame.render_widget(process_list, chunks[2]);

    // Footer
    let count_text = format!(
        " {}/{} processes",
        app.filtered_processes.len(),
        app.all_processes.len()
    );
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(count_text, Style::default().fg(accent)),
        Span::raw("  "),
        Span::styled(
            "/",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ),
        Span::styled(" search ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "j/k",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ),
        Span::styled(" navigate ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "esc",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ),
        Span::styled(" back", Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color)),
    );
    frame.render_widget(footer, chunks[3]);
}

// ── Module detail view ──────────────────────────────────────────────

fn draw_module_detail(frame: &mut ratatui::Frame, app: &App, idx: usize) {
    let area = frame.area();
    let snapshot = match &app.snapshot {
        Some(s) => s,
        None => return,
    };
    let (name, data) = match snapshot.modules.get(idx) {
        Some(m) => m,
        None => return,
    };

    let fg = resolve_module_fg(&app.config, name);
    let accent = resolve_module_accent(&app.config, name);
    let border_color = resolve_border(&app.config);
    let border_type = resolve_border_type(&app.config);

    let has_gauge = extract_gauge(name, data).is_some();
    let has_history = app
        .history
        .get(name)
        .map(|h| !h.is_empty())
        .unwrap_or(false);

    let mut constraints = Vec::new();
    if has_gauge {
        constraints.push(Constraint::Length(4));
    }
    if has_history {
        constraints.push(Constraint::Length(8));
    }
    constraints.push(Constraint::Min(5)); // details
    constraints.push(Constraint::Length(3)); // footer

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut chunk_idx = 0;

    // Gauge
    if let Some(gauge_data) = extract_gauge(name, data) {
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(border_type)
                    .border_style(Style::default().fg(border_color))
                    .title(Span::styled(
                        format!(" {} ", module_label(&app.config, name)),
                        Style::default().fg(fg).add_modifier(Modifier::BOLD),
                    )),
            )
            .gauge_style(Style::default().fg(accent).bg(Color::DarkGray))
            .use_unicode(true)
            .ratio(gauge_data.ratio.clamp(0.0, 1.0))
            .label(Span::styled(
                format!("{:.1}%", gauge_data.ratio * 100.0),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
        frame.render_widget(gauge, chunks[chunk_idx]);
        chunk_idx += 1;
    }

    // Sparkline
    if has_history {
        if let Some(history) = app.history.get(name) {
            let data_vec: Vec<u64> = history.iter().map(|&v| v as u64).collect();
            let sparkline = Sparkline::default()
                .block(
                    Block::default()
                        .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                        .border_type(border_type)
                        .border_style(Style::default().fg(border_color))
                        .title(Span::styled(
                            " History ",
                            Style::default().fg(Color::DarkGray),
                        )),
                )
                .data(data_vec)
                .style(Style::default().fg(accent))
                .max(100);
            frame.render_widget(sparkline, chunks[chunk_idx]);
        }
        chunk_idx += 1;
    }

    // Details
    let mut lines: Vec<Line> = Vec::new();
    let mut entries: Vec<_> = data.metrics.iter().collect();
    entries.sort_by_key(|(k, _)| *k);

    for (key, value) in entries {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} ", key),
                Style::default().fg(fg).add_modifier(Modifier::BOLD),
            ),
            Span::styled("→ ", Style::default().fg(Color::DarkGray)),
            Span::styled(metric_display(value), Style::default().fg(Color::White)),
        ]));
    }

    let detail_title = if !has_gauge {
        format!(" {} — Details ", module_label(&app.config, name))
    } else {
        " Details ".to_string()
    };

    let detail = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(border_type)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    detail_title,
                    Style::default().fg(if has_gauge { Color::DarkGray } else { fg }),
                )),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(detail, chunks[chunk_idx]);
    chunk_idx += 1;

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            "esc",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ),
        Span::styled(" back ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "q",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ),
        Span::styled(" quit", Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color)),
    );
    frame.render_widget(footer, chunks[chunk_idx]);
}

// ── Helpers ─────────────────────────────────────────────────────────

fn resolve_border_type(config: &Config) -> ratatui::widgets::BorderType {
    match config.tui.borders {
        BorderStyle::None | BorderStyle::Plain => ratatui::widgets::BorderType::Plain,
        BorderStyle::Rounded => ratatui::widgets::BorderType::Rounded,
    }
}

fn resolve_module_fg(config: &Config, name: &str) -> Color {
    match config.theme_mode {
        ThemeMode::Terminal => crate::config::ansi_module_color(name),
        ThemeMode::Custom => module_fg_color(config, name),
    }
}

fn resolve_module_accent(config: &Config, name: &str) -> Color {
    match config.theme_mode {
        ThemeMode::Terminal => crate::config::ansi_module_color(name),
        ThemeMode::Custom => module_accent_color(config, name),
    }
}

fn resolve_border(config: &Config) -> Color {
    match config.theme_mode {
        ThemeMode::Terminal => crate::config::ansi_chrome_border(),
        ThemeMode::Custom => parse_color(&config.theme.chrome.border),
    }
}

fn resolve_title(config: &Config) -> Color {
    match config.theme_mode {
        ThemeMode::Terminal => crate::config::ansi_chrome_title(),
        ThemeMode::Custom => parse_color(&config.theme.chrome.title),
    }
}

struct GaugeData {
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
        MetricValue::Float(f) => Some(GaugeData { ratio: *f / 100.0 }),
        _ => None,
    })
}

fn extract_history_value(module_name: &str, data: &crate::core::MetricData) -> Option<f64> {
    let key = match module_name {
        "cpu" => "cpu_usage_percent",
        "memory" => "memory_usage_percent",
        "disk" => "usage_percent",
        _ => return None,
    };

    data.metrics.get(key).and_then(|v| match v {
        MetricValue::Float(f) => Some(*f),
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
        format!("{:.1} GB", abs as f64 / 1_073_741_824.0)
    } else if abs >= 1_048_576 {
        format!("{:.1} MB", abs as f64 / 1_048_576.0)
    } else if abs >= 1024 {
        format!("{:.1} KB", abs as f64 / 1024.0)
    } else {
        value.to_string()
    }
}

fn format_mem(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let truncated: String = s.chars().take(max - 1).collect();
        format!("{}…", truncated)
    } else {
        s.to_string()
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

fn module_fg_color(config: &Config, name: &str) -> Color {
    parse_color(&module_theme(config, name).fg)
}

fn module_accent_color(config: &Config, name: &str) -> Color {
    parse_color(&module_theme(config, name).accent)
}

fn module_label(config: &Config, name: &str) -> String {
    if config.tui.icons {
        match name {
            "cpu" => " CPU".into(),
            "memory" => " MEM".into(),
            "disk" => " DSK".into(),
            "network" => "󰈀 NET".into(),
            "process" => " PRC".into(),
            "system" => "󰒋 SYS".into(),
            _ => name.to_uppercase(),
        }
    } else {
        let label = &module_theme(config, name).label;
        if label.is_empty() {
            name.to_uppercase()
        } else {
            label.clone()
        }
    }
}
