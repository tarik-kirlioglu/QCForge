pub mod layout;
pub mod tabs;
pub mod widgets;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::state::{ActiveTab, AppState};

pub fn draw(frame: &mut Frame, state: &AppState) {
    // Splash / loading state
    if state.loading {
        render_splash(frame, state.splash_tick);
        return;
    }

    // Error state
    if let Some(ref err) = state.error_message {
        render_error(frame, err);
        return;
    }

    // Main chrome (header, tabs, footer)
    let app_layout = layout::render_chrome(frame, state);

    // Tab content
    match state.active_tab {
        ActiveTab::Summary => tabs::summary::render(frame, app_layout.content, state),
        ActiveTab::Overview => tabs::overview::render(frame, app_layout.content, state),
        ActiveTab::Samtools => tabs::samtools::render(frame, app_layout.content, state),
        ActiveTab::Bcftools => tabs::bcftools::render(frame, app_layout.content, state),
        ActiveTab::Fastqc => tabs::fastqc::render(frame, app_layout.content, state),
    }

    // Help overlay
    if state.show_help {
        render_help_overlay(frame);
    }
}

fn render_splash(frame: &mut Frame, tick: u16) {
    let area = frame.area();
    let w = area.width as usize;
    let h = area.height as usize;
    if w < 10 || h < 10 {
        return;
    }

    let logo = [
        r"   ___   ____  _____                        ",
        r"  / _ \ / ___||  ___|___  _ __ __ _  ___    ",
        r" | | | | |    | |_ / _ \| '__/ _` |/ _ \   ",
        r" | |_| | |___ |  _| (_) | | | (_| |  __/   ",
        r"  \__\_\\____||_|  \___/|_|  \__, |\___|   ",
        r"                             |___/          ",
    ];

    let subtitle = "Terminal QC Dashboard for Bioinformatics";
    let loading_text = "Loading QC data...";

    // Spark particles
    let spark_chars = ['✦', '·', '°', '*', '∘', '⁕', '✧'];
    let spark_colors = [
        Color::Rgb(255, 69, 0),   // red-orange
        Color::Rgb(255, 140, 0),  // dark orange
        Color::Rgb(255, 165, 0),  // orange
        Color::Rgb(255, 200, 50), // golden
        Color::Rgb(255, 220, 100),// light gold
        Color::Rgb(255, 255, 150),// pale yellow
        Color::Rgb(200, 200, 200),// fading gray
    ];

    let mut lines: Vec<Line> = Vec::new();

    // Calculate logo position (centered vertically)
    let logo_start_row = h / 2 - 5;

    // Simple deterministic "random" based on tick and position
    let seed = tick as usize;

    for row in 0..h {
        if row >= logo_start_row && row < logo_start_row + logo.len() {
            // Logo line
            let logo_line = logo[row - logo_start_row];
            let pad = if w > logo_line.len() {
                (w - logo_line.len()) / 2
            } else {
                0
            };

            // Fade-in effect: reveal chars based on tick
            let reveal = ((tick as usize) * 3).min(logo_line.len());
            let visible = &logo_line[..reveal];
            let hidden = &logo_line[reveal..];

            let mut spans = vec![Span::raw(" ".repeat(pad))];
            spans.push(Span::styled(
                visible.to_string(),
                Style::default()
                    .fg(Color::Rgb(255, 180, 50))
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                hidden.to_string(),
                Style::default().fg(Color::Rgb(40, 40, 40)),
            ));
            lines.push(Line::from(spans));
        } else if row == logo_start_row + logo.len() + 1 {
            // Subtitle
            let pad = if w > subtitle.len() {
                (w - subtitle.len()) / 2
            } else {
                0
            };
            lines.push(Line::from(vec![
                Span::raw(" ".repeat(pad)),
                Span::styled(
                    subtitle,
                    Style::default().fg(Color::Rgb(180, 180, 180)),
                ),
            ]));
        } else if row == logo_start_row + logo.len() + 3 {
            // Loading text with animated dots
            let dots = ".".repeat(((tick / 3) % 4) as usize);
            let lt = format!("{}{}", loading_text.trim_end_matches('.'), dots);
            let pad = if w > lt.len() {
                (w - lt.len()) / 2
            } else {
                0
            };
            lines.push(Line::from(vec![
                Span::raw(" ".repeat(pad)),
                Span::styled(
                    lt,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        } else {
            // Spark particle rows
            let mut spans: Vec<Span> = Vec::new();
            let mut col = 0;
            while col < w {
                // Deterministic pseudo-random particle placement
                let hash = (row.wrapping_mul(31).wrapping_add(col.wrapping_mul(17)).wrapping_add(seed.wrapping_mul(7))) % 47;
                if hash < 3 {
                    // Place a spark
                    let char_idx = (hash + seed + row) % spark_chars.len();
                    let color_idx = if row < h / 2 {
                        // Upper sparks: fading (higher = more faded)
                        let dist = (h / 2).saturating_sub(row);
                        (spark_colors.len() - 1).min(dist / 2)
                    } else {
                        // Lower sparks: hot (closer to bottom = hotter)
                        let dist = row.saturating_sub(h / 2);
                        spark_colors.len().saturating_sub(1).min(dist / 3)
                    };
                    // Animate: some sparks blink based on tick
                    let visible = (seed + row + col) % 3 != 0;
                    if visible {
                        spans.push(Span::styled(
                            spark_chars[char_idx].to_string(),
                            Style::default().fg(spark_colors[color_idx]),
                        ));
                    } else {
                        spans.push(Span::raw(" "));
                    }
                } else {
                    spans.push(Span::raw(" "));
                }
                col += 1;
            }
            lines.push(Line::from(spans));
        }
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_error(frame: &mut Frame, error: &str) {
    let area = frame.area();
    let text = Paragraph::new(vec![
        Line::from(Span::styled(
            "Error",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(error.to_string()),
        Line::from(""),
        Line::from(Span::styled(
            "Press q to quit",
            Style::default().fg(Color::Gray),
        )),
    ])
    .block(
        Block::default()
            .title(" QCForge ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
    )
    .wrap(Wrap { trim: false });

    frame.render_widget(text, area);
}

fn render_help_overlay(frame: &mut Frame) {
    let area = frame.area();
    let popup = centered_rect(50, 60, area);

    let help_text = vec![
        Line::from(Span::styled(
            "Keybindings",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  q / Esc       Quit"),
        Line::from("  ← / →/ Tab   Switch tabs"),
        Line::from("  j / k / ↑↓   Scroll"),
        Line::from("  n / p         Next/Prev file"),
        Line::from("  s             Cycle sort column"),
        Line::from("  S             Toggle sort direction"),
        Line::from("  /             Search files"),
        Line::from("  h / l         Scroll columns (Summary)"),
        Line::from("  ?             Toggle this help"),
        Line::from(""),
        Line::from(Span::styled(
            "Press ? to close",
            Style::default().fg(Color::Gray),
        )),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(Clear, popup);
    frame.render_widget(help, popup);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
