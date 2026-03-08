# TUI Redesign — Ricing-First UX with fzf Process Finder

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Redesign gim's TUI into a ricing-friendly dashboard with zoom-in detail views, fzf-style process fuzzy finder, terminal-native ANSI colors, Nerd Font icons, and a color palette bar.

**Architecture:** Hybrid layout — compact dashboard overview as default, Enter zooms into any module for full-screen detail. Process zoom-in becomes a full-screen fuzzy finder with nucleo matching. Colors default to ANSI 16 palette (inherits terminal theme) with optional hex override via `theme_mode: "custom"` in config.

**Tech Stack:** ratatui 0.29, crossterm 0.28, nucleo-matcher (fuzzy search), sysinfo 0.37

---

### Task 1: Add nucleo-matcher dependency

**Files:**
- Modify: `Cargo.toml:9-18`

**Step 1: Add nucleo-matcher to dependencies**

In `Cargo.toml`, add after the `crossterm` line:

```toml
nucleo-matcher = "0.3"
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "deps: add nucleo-matcher for fuzzy process search"
```

---

### Task 2: Config — Add theme_mode, icons toggle, and ANSI color defaults

**Files:**
- Modify: `src/config/mod.rs`

**Step 1: Add `theme_mode` and `icons` fields to config structs**

Add to `TuiConfig` struct (after `show_help`):

```rust
pub icons: bool,
```

Add a new field to `Config` struct (after `theme`):

```rust
pub theme_mode: ThemeMode,
```

Add enum before `Config`:

```rust
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    Terminal,
    Custom,
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::Terminal
    }
}
```

**Step 2: Update defaults**

In `Default for TuiConfig`, add:
```rust
icons: true,
```

In `Default for Config`, add:
```rust
theme_mode: ThemeMode::default(),
```

**Step 3: Update `parse_color` to support ANSI mapping**

Add a new public function that resolves colors based on theme mode:

```rust
/// Maps module names to ANSI terminal colors (adapts to user's terminal theme)
pub fn ansi_module_color(name: &str) -> Color {
    match name {
        "cpu" => Color::Cyan,
        "memory" => Color::Green,
        "disk" => Color::Yellow,
        "network" => Color::Magenta,
        "process" => Color::Red,
        "system" => Color::Blue,
        _ => Color::White,
    }
}

pub fn ansi_chrome_border() -> Color {
    Color::DarkGray
}

pub fn ansi_chrome_title() -> Color {
    Color::White
}
```

**Step 4: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add src/config/mod.rs
git commit -m "feat(config): add theme_mode (terminal/custom), icons toggle"
```

---

### Task 3: Refactor App state for view modes (Dashboard vs ZoomIn)

**Files:**
- Modify: `src/tui/mod.rs:22-65`

**Step 1: Add ViewMode enum and update App struct**

Add before `App` struct:

```rust
#[derive(Debug, Clone, PartialEq)]
enum ViewMode {
    Dashboard,
    ZoomIn(usize), // index of zoomed module
}
```

Add to `App` struct:

```rust
view_mode: ViewMode,
search_query: String,
search_active: bool,
filtered_processes: Vec<ProcessEntry>,
process_scroll: usize,
process_selected: usize,
all_processes: Vec<ProcessEntry>,
```

Add the `ProcessEntry` struct:

```rust
#[derive(Debug, Clone)]
struct ProcessEntry {
    pid: u32,
    name: String,
    cpu: f32,
    memory: u64,
    command: String,
    score: Option<u32>,  // fuzzy match score
}
```

Update `App::new()` to initialize these fields.

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/tui/mod.rs
git commit -m "feat(tui): add ViewMode enum and process fuzzy finder state"
```

---

### Task 4: Update keybindings for zoom-in/out navigation

**Files:**
- Modify: `src/tui/mod.rs` (event loop, around line 83-123)

**Step 1: Add Enter/Esc keybindings**

In the key event match block, add:

```rust
KeyCode::Enter => {
    match &app.view_mode {
        ViewMode::Dashboard => {
            app.view_mode = ViewMode::ZoomIn(app.selected_tab);
            // If zooming into process module, refresh process list
            if let Some(snapshot) = &app.snapshot {
                if let Some((name, _)) = snapshot.modules.get(app.selected_tab) {
                    if name == "process" {
                        app.refresh_processes();
                        app.search_active = true;
                    }
                }
            }
        }
                            ViewMode::ZoomIn(_) => {
                                // In process view with search active, selecting does nothing yet
                            }
                        }
                    }
                    KeyCode::Esc => {
                        if app.search_active {
                            app.search_active = false;
                            app.search_query.clear();
                            app.filtered_processes = app.all_processes.clone();
                        } else {
                            match &app.view_mode {
                                ViewMode::ZoomIn(_) => {
                                    app.view_mode = ViewMode::Dashboard;
                                }
                                ViewMode::Dashboard => {
                                    app.should_quit = true;
                                }
                            }
                        }
                    }
```

Add Char handler for search input when in process zoom-in:

```rust
KeyCode::Char(c) => {
    if app.search_active {
        app.search_query.push(c);
        app.apply_fuzzy_filter();
    } else if c == 'q' {
        match &app.view_mode {
            ViewMode::Dashboard => app.should_quit = true,
            ViewMode::ZoomIn(_) => app.view_mode = ViewMode::Dashboard,
        }
    } else if c == '/' || c == 'f' {
        // Activate search in process zoom
        if let ViewMode::ZoomIn(idx) = &app.view_mode {
            if let Some(snapshot) = &app.snapshot {
                if let Some((name, _)) = snapshot.modules.get(*idx) {
                    if name == "process" {
                        app.search_active = true;
                    }
                }
            }
        }
    } else if c == 'j' {
        app.navigate_down();
    } else if c == 'k' {
        app.navigate_up();
    } else if c == 'l' {
        // existing tab next
    } else if c == 'h' {
        // existing tab prev
    }
}
KeyCode::Backspace => {
    if app.search_active {
        app.search_query.pop();
        app.apply_fuzzy_filter();
    }
}
KeyCode::Down => app.navigate_down(),
KeyCode::Up => app.navigate_up(),
```

**Step 2: Add navigation methods to App**

```rust
impl App {
    fn navigate_down(&mut self) {
        if let ViewMode::ZoomIn(_) = &self.view_mode {
            let max = self.filtered_processes.len().saturating_sub(1);
            self.process_selected = (self.process_selected + 1).min(max);
        }
    }

    fn navigate_up(&mut self) {
        if let ViewMode::ZoomIn(_) = &self.view_mode {
            self.process_selected = self.process_selected.saturating_sub(1);
        }
    }

    fn refresh_processes(&mut self) {
        use sysinfo::{System, RefreshKind, ProcessRefreshKind};
        let sys = System::new_with_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
        );
        self.all_processes = sys.processes().values().map(|p| {
            ProcessEntry {
                pid: p.pid().as_u32(),
                name: p.name().to_string_lossy().to_string(),
                cpu: p.cpu_usage(),
                memory: p.memory(),
                command: p.exe().map(|e| e.display().to_string()).unwrap_or_default(),
                score: None,
            }
        }).collect();
        self.all_processes.sort_by(|a, b| b.memory.cmp(&a.memory));
        self.filtered_processes = self.all_processes.clone();
        self.process_selected = 0;
        self.process_scroll = 0;
    }

    fn apply_fuzzy_filter(&mut self) {
        use nucleo_matcher::pattern::{Pattern, CaseMatching, Normalization};
        use nucleo_matcher::{Matcher, Config as NucleoConfig};
        use nucleo_matcher::Utf32Str;

        if self.search_query.is_empty() {
            self.filtered_processes = self.all_processes.clone();
            self.process_selected = 0;
            return;
        }

        let mut matcher = Matcher::new(NucleoConfig::DEFAULT);
        let pattern = Pattern::parse(&self.search_query, CaseMatching::Smart, Normalization::Smart);
        let mut buf = Vec::new();

        let mut scored: Vec<(u32, ProcessEntry)> = self.all_processes.iter().filter_map(|p| {
            let haystack = Utf32Str::new(&p.name, &mut buf);
            pattern.score(haystack, &mut matcher).map(|score| {
                let mut entry = p.clone();
                entry.score = Some(score);
                (score, entry)
            })
        }).collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        self.filtered_processes = scored.into_iter().map(|(_, e)| e).collect();
        self.process_selected = 0;
    }
}
```

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/tui/mod.rs
git commit -m "feat(tui): add zoom-in keybindings and process fuzzy search logic"
```

---

### Task 5: Build the dashboard renderer with ANSI colors and Nerd Font icons

**Files:**
- Modify: `src/tui/mod.rs` (draw_ui, draw_header, draw_modules, draw_footer)

**Step 1: Update `draw_ui` to route based on ViewMode**

```rust
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
```

**Step 2: Refactor color resolution to respect theme_mode**

Add helper function:

```rust
fn resolve_module_fg(config: &Config, name: &str) -> Color {
    match config.theme_mode {
        crate::config::ThemeMode::Terminal => crate::config::ansi_module_color(name),
        crate::config::ThemeMode::Custom => module_fg_color(config, name),
    }
}

fn resolve_module_accent(config: &Config, name: &str) -> Color {
    match config.theme_mode {
        crate::config::ThemeMode::Terminal => crate::config::ansi_module_color(name),
        crate::config::ThemeMode::Custom => module_accent_color(config, name),
    }
}

fn resolve_border(config: &Config) -> Color {
    match config.theme_mode {
        crate::config::ThemeMode::Terminal => crate::config::ansi_chrome_border(),
        crate::config::ThemeMode::Custom => parse_color(&config.theme.chrome.border),
    }
}

fn resolve_title(config: &Config) -> Color {
    match config.theme_mode {
        crate::config::ThemeMode::Terminal => crate::config::ansi_chrome_title(),
        crate::config::ThemeMode::Custom => parse_color(&config.theme.chrome.title),
    }
}
```

**Step 3: Update Nerd Font icon mapping**

Update `module_label` to use icons when enabled:

```rust
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
```

**Step 4: Update draw_header to use resolve functions and show Nerd Font labels**

Replace all `module_fg_color` calls with `resolve_module_fg`, `parse_color(&theme.chrome.border)` with `resolve_border`, etc.

**Step 5: Add color palette bar to footer**

Update `draw_footer`:

```rust
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

    // Split footer: help keys on left, color palette on right
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(30), Constraint::Length(38)])
        .split(area);

    // Help keys
    let help = Paragraph::new(Line::from(vec![
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD).fg(Color::White)),
        Span::styled(" quit ", Style::default().fg(Color::DarkGray)),
        Span::styled("←→", Style::default().add_modifier(Modifier::BOLD).fg(Color::White)),
        Span::styled(" tab ", Style::default().fg(Color::DarkGray)),
        Span::styled("⏎", Style::default().add_modifier(Modifier::BOLD).fg(Color::White)),
        Span::styled(" zoom ", Style::default().fg(Color::DarkGray)),
        Span::styled("esc", Style::default().add_modifier(Modifier::BOLD).fg(Color::White)),
        Span::styled(" back", Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color)),
    );
    frame.render_widget(help, chunks[0]);

    // Color palette — shows terminal's ANSI 16 colors
    let palette_colors = [
        Color::Black, Color::Red, Color::Green, Color::Yellow,
        Color::Blue, Color::Magenta, Color::Cyan, Color::White,
        Color::DarkGray, Color::LightRed, Color::LightGreen, Color::LightYellow,
        Color::LightBlue, Color::LightMagenta, Color::LightCyan, Color::White,
    ];
    let palette_spans: Vec<Span> = palette_colors
        .iter()
        .map(|&c| Span::styled("██", Style::default().fg(c)))
        .collect();

    let palette = Paragraph::new(Line::from(palette_spans))
        .alignment(ratatui::layout::Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(border_type)
                .border_style(Style::default().fg(border_color)),
        );
    frame.render_widget(palette, chunks[1]);
}
```

**Step 6: Update draw_modules to use resolve functions**

Replace all hardcoded color calls with `resolve_module_fg` / `resolve_module_accent` / `resolve_border`.

**Step 7: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully

**Step 8: Commit**

```bash
git add src/tui/mod.rs
git commit -m "feat(tui): ricing-first dashboard with ANSI colors, Nerd Font icons, color palette"
```

---

### Task 6: Build the process fuzzy finder full-screen view

**Files:**
- Modify: `src/tui/mod.rs`

**Step 1: Implement `draw_process_finder`**

```rust
fn draw_process_finder(frame: &mut ratatui::Frame, app: &App) {
    let area = frame.area();
    let border_color = resolve_border(&app.config);
    let accent = resolve_module_fg(&app.config, "process");

    let border_type = match app.config.tui.borders {
        BorderStyle::None => ratatui::widgets::BorderType::Plain,
        BorderStyle::Plain => ratatui::widgets::BorderType::Plain,
        BorderStyle::Rounded => ratatui::widgets::BorderType::Rounded,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // search bar
            Constraint::Length(2),  // column headers
            Constraint::Min(5),    // process list
            Constraint::Length(3), // footer help
        ])
        .split(area);

    // Search bar
    let search_icon = if app.config.tui.icons { " " } else { ">" };
    let search_text = format!("{} {}", search_icon, app.search_query);
    let cursor = if app.search_active { "█" } else { "" };
    let search = Paragraph::new(Line::from(vec![
        Span::styled(&search_text, Style::default().fg(Color::White)),
        Span::styled(cursor, Style::default().fg(accent)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(if app.search_active { accent } else { border_color }))
            .title(Span::styled(
                if app.config.tui.icons { "  Process Finder " } else { " Process Finder " },
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(search, chunks[0]);

    // Column headers
    let headers = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {:>7}  {:<20}  {:>6}  {:>10}  {}",
                "PID", "NAME", "CPU%", "MEM", "COMMAND"),
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(headers, chunks[1]);

    // Process list
    let visible_height = chunks[2].height as usize;
    // Adjust scroll to keep selected visible
    let scroll = if app.process_selected >= visible_height {
        app.process_selected - visible_height + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    for (i, proc) in app.filtered_processes.iter().enumerate().skip(scroll).take(visible_height) {
        let is_selected = i == app.process_selected;
        let mem_str = format_mem(proc.memory);
        let line_str = format!(
            " {:>7}  {:<20}  {:>5.1}%  {:>10}  {}",
            proc.pid,
            truncate_str(&proc.name, 20),
            proc.cpu,
            mem_str,
            truncate_str(&proc.command, 40),
        );

        let style = if is_selected {
            Style::default().bg(accent).fg(Color::Black).add_modifier(Modifier::BOLD)
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

    // Footer with count and help
    let count_text = format!(
        " {}/{} processes",
        app.filtered_processes.len(),
        app.all_processes.len()
    );
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(&count_text, Style::default().fg(accent)),
        Span::raw("  "),
        Span::styled("/", Style::default().add_modifier(Modifier::BOLD).fg(Color::White)),
        Span::styled(" search ", Style::default().fg(Color::DarkGray)),
        Span::styled("j/k", Style::default().add_modifier(Modifier::BOLD).fg(Color::White)),
        Span::styled(" navigate ", Style::default().fg(Color::DarkGray)),
        Span::styled("esc", Style::default().add_modifier(Modifier::BOLD).fg(Color::White)),
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

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}…", &s[..max - 1])
    } else {
        s.to_string()
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
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/tui/mod.rs
git commit -m "feat(tui): full-screen process fuzzy finder with nucleo matching"
```

---

### Task 7: Build generic module zoom-in detail view

**Files:**
- Modify: `src/tui/mod.rs`

**Step 1: Implement `draw_module_detail` for non-process modules**

```rust
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

    let border_type = match app.config.tui.borders {
        BorderStyle::None => ratatui::widgets::BorderType::Plain,
        BorderStyle::Plain => ratatui::widgets::BorderType::Plain,
        BorderStyle::Rounded => ratatui::widgets::BorderType::Rounded,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // gauge (if applicable)
            Constraint::Length(8),  // sparkline
            Constraint::Min(5),    // details
            Constraint::Length(3), // footer
        ])
        .split(area);

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
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ));
        frame.render_widget(gauge, chunks[0]);
    } else {
        // Title block for modules without gauge
        let title_block = Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                format!(" {} ", module_label(&app.config, name)),
                Style::default().fg(fg).add_modifier(Modifier::BOLD),
            ));
        frame.render_widget(title_block, chunks[0]);
    }

    // Sparkline
    if let Some(history) = app.history.get(name) {
        if !history.is_empty() {
            let sparkline_data: Vec<u64> = history.iter().map(|&v| v as u64).collect();
            let sparkline = Sparkline::default()
                .block(
                    Block::default()
                        .borders(Borders::LEFT | Borders::RIGHT)
                        .border_type(border_type)
                        .border_style(Style::default().fg(border_color))
                        .title(Span::styled(" History ", Style::default().fg(Color::DarkGray))),
                )
                .data(sparkline_data)
                .style(Style::default().fg(accent))
                .max(100);
            frame.render_widget(sparkline, chunks[1]);
        }
    }

    // Detail key-value pairs
    let mut lines: Vec<Line> = Vec::new();
    let mut entries: Vec<_> = data.metrics.iter().collect();
    entries.sort_by_key(|(k, _)| *k);

    for (key, value) in entries {
        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", key), Style::default().fg(fg).add_modifier(Modifier::BOLD)),
            Span::styled("→ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", metric_display(value)),
                Style::default().fg(Color::White),
            ),
        ]));
    }

    let detail = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(border_type)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(" Details ", Style::default().fg(Color::DarkGray))),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(detail, chunks[2]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("esc", Style::default().add_modifier(Modifier::BOLD).fg(Color::White)),
        Span::styled(" back ", Style::default().fg(Color::DarkGray)),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD).fg(Color::White)),
        Span::styled(" quit", Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color)),
    );
    frame.render_widget(footer, chunks[3]);
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/tui/mod.rs
git commit -m "feat(tui): full-screen module detail view with gauge, sparkline, and details"
```

---

### Task 8: Polish dashboard layout — improved grid, separators, alignment

**Files:**
- Modify: `src/tui/mod.rs` (draw_modules function)

**Step 1: Rename `draw_modules` to `draw_dashboard` and refactor**

Create `draw_dashboard` that wraps: draw_header + draw_modules + draw_footer, using the resolved color functions throughout. Update the module grid to use 3 columns for 6 modules (2 rows x 3 cols).

Improve detail rendering inside each module panel:
- Right-align keys with fixed width
- Use `→` separator instead of `›`
- Add padding for cleaner look

**Step 2: Verify and commit**

Run: `cargo check && cargo run -- tui`
Expected: Dashboard renders with improved layout

```bash
git add src/tui/mod.rs
git commit -m "feat(tui): polish dashboard grid layout and metric alignment"
```

---

### Task 9: Update output/mod.rs to use Nerd Font icons in print mode

**Files:**
- Modify: `src/output/mod.rs`

**Step 1: Add icon support to format_snapshot**

Update the module header/footer borders to optionally include Nerd Font icons:

```rust
fn module_icon(name: &str) -> &'static str {
    match name {
        "cpu" => "",
        "memory" => "",
        "disk" => "",
        "network" => "󰈀",
        "process" => "",
        "system" => "󰒋",
        _ => "●",
    }
}
```

Update `format_snapshot` to use: `format!("{} {}", module_icon(name), name.to_uppercase())`

**Step 2: Verify and commit**

Run: `cargo run -- print -m cpu`
Expected: Shows icon in output

```bash
git add src/output/mod.rs
git commit -m "feat(output): add Nerd Font icons to print mode module headers"
```

---

### Task 10: Integration test — full flow verification

**Step 1: Build release and test**

Run: `cargo build --release`
Expected: Builds successfully

**Step 2: Test dashboard mode**

Run: `cargo run -- tui`
Expected:
- Dashboard shows all 6 modules in 3x2 grid
- Nerd Font icons visible in tab bar and panel titles
- ANSI colors adapt to terminal theme
- Color palette bar visible in footer
- Tab/arrow keys navigate between modules

**Step 3: Test zoom-in**

- Press Enter on CPU tab → full-screen CPU detail with gauge + sparkline + details
- Press Esc → back to dashboard
- Navigate to Process tab, press Enter → full-screen fuzzy finder
- Type to filter processes, j/k to navigate
- Esc clears search, Esc again returns to dashboard

**Step 4: Test print mode**

Run: `cargo run -- print -m cpu,memory`
Expected: Shows Nerd Font icons in module headers

**Step 5: Final commit**

```bash
git add -A
git commit -m "feat(tui): complete ricing-first TUI redesign with fzf process finder"
```

---

## Summary of Changes

| File | Change |
|------|--------|
| `Cargo.toml` | Add `nucleo-matcher` dependency |
| `src/config/mod.rs` | Add `ThemeMode`, `icons` toggle, ANSI color helpers |
| `src/tui/mod.rs` | Complete rewrite: ViewMode, dashboard, zoom-in, process finder, color palette |
| `src/output/mod.rs` | Add Nerd Font icons to print output |
