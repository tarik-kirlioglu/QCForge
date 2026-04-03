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
        render_splash(frame, state.splash_tick, &state.splash_status);
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

fn render_splash(frame: &mut Frame, tick: u16, status: &str) {
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

    let strand_chars = ['A', 'T', 'G', 'C'];

    // DNA base colors (bioinformatics standard)
    let base_colors: [Color; 4] = [
        Color::Rgb(80, 220, 100),  // A = green
        Color::Rgb(220, 60, 60),   // T = red
        Color::Rgb(60, 140, 255),  // G = blue
        Color::Rgb(255, 180, 40),  // C = yellow/amber
    ];

    // Mid-brightness tints
    let mid_colors: [Color; 4] = [
        Color::Rgb(25, 65, 30),   // A
        Color::Rgb(65, 18, 18),   // T
        Color::Rgb(18, 40, 75),   // G
        Color::Rgb(75, 55, 12),   // C
    ];

    // Dim near-black tints
    let dim_colors: [Color; 4] = [
        Color::Rgb(10, 25, 12),   // A
        Color::Rgb(25, 8, 8),     // T
        Color::Rgb(8, 15, 30),    // G
        Color::Rgb(30, 22, 5),    // C
    ];

    // Cross-link color (hydrogen bonds between strands)
    let bond_color = Color::Rgb(200, 170, 80);

    // Deterministic base assignment per cell (position-only, never changes)
    let base_at = |row: usize, col: usize| -> usize {
        let h = row.wrapping_mul(137).wrapping_add(col.wrapping_mul(251));
        let h = h ^ (h >> 4);
        let h = h.wrapping_mul(0x9E37_79B9);
        h & 3
    };

    let seed = tick as usize;
    let hf = h as f32;

    let mut lines: Vec<Line> = Vec::new();

    // Calculate logo position (centered vertically)
    let logo_start_row = h / 2 - 5;

    // Build a row of base-letter spans with double-helix coloring
    // Letters are fixed; only color changes based on distance to helix waves
    let build_base_spans = |row: usize, col_start: usize, col_end: usize| -> Vec<Span<'static>> {
        let mut spans: Vec<Span<'static>> = Vec::new();
        for col in col_start..col_end {
            let bi = base_at(row, col);

            // Two sine waves offset by pi — same as the original double helix
            let phase1 = ((col as f32 + seed as f32 * 1.5) * 0.15).sin();
            let phase2 = ((col as f32 + seed as f32 * 1.5 + 3.14) * 0.15).sin();
            let wave1_row = (hf / 2.0 + phase1 * (hf * 0.3)) as usize;
            let wave2_row = (hf / 2.0 + phase2 * (hf * 0.3)) as usize;

            let dist1 = (row as isize - wave1_row as isize).unsigned_abs();
            let dist2 = (row as isize - wave2_row as isize).unsigned_abs();
            let min_dist = dist1.min(dist2);

            let (color, bold) = if min_dist == 0 {
                // On the helix strand — full bright base color
                (base_colors[bi], true)
            } else if min_dist <= 1 && dist1 <= 1 && dist2 <= 1 {
                // Between strands — hydrogen bond color
                (bond_color, false)
            } else if min_dist <= 2 {
                // Near helix — mid tint
                (mid_colors[bi], false)
            } else {
                // Background — dim
                (dim_colors[bi], false)
            };

            let mut style = Style::default().fg(color);
            if bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            spans.push(Span::styled(
                strand_chars[bi].to_string(),
                style,
            ));
        }
        spans
    };

    for row in 0..h {
        if row >= logo_start_row && row < logo_start_row + logo.len() {
            // Logo line overlaid on base background
            let logo_line = logo[row - logo_start_row];
            let pad = if w > logo_line.len() {
                (w - logo_line.len()) / 2
            } else {
                0
            };
            let text_end = (pad + logo_line.len()).min(w);

            let mut spans = build_base_spans(row, 0, pad);

            // Fade-in effect: reveal chars based on tick
            let reveal = ((tick as usize) * 3).min(logo_line.len());
            let visible = &logo_line[..reveal];
            let hidden = &logo_line[reveal..];

            spans.push(Span::styled(
                visible.to_string(),
                Style::default()
                    .fg(Color::Rgb(180, 220, 255))
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                hidden.to_string(),
                Style::default().fg(Color::Rgb(25, 30, 40)),
            ));

            spans.extend(build_base_spans(row, text_end, w));
            lines.push(Line::from(spans));
        } else if row == logo_start_row + logo.len() + 1 {
            // Subtitle overlaid on base background
            let pad = if w > subtitle.len() {
                (w - subtitle.len()) / 2
            } else {
                0
            };
            let text_end = (pad + subtitle.len()).min(w);

            let mut spans = build_base_spans(row, 0, pad);
            spans.push(Span::styled(
                subtitle,
                Style::default().fg(Color::Rgb(180, 180, 180)),
            ));
            spans.extend(build_base_spans(row, text_end, w));
            lines.push(Line::from(spans));
        } else if row == logo_start_row + logo.len() + 3 {
            // Loading text overlaid on base background
            let dots = ".".repeat(((tick / 3) % 4) as usize);
            let lt = format!("{}{}", status.trim_end_matches('.'), dots);
            let pad = if w > lt.len() {
                (w - lt.len()) / 2
            } else {
                0
            };
            let text_end = (pad + lt.len()).min(w);

            let mut spans = build_base_spans(row, 0, pad);
            spans.push(Span::styled(
                lt,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.extend(build_base_spans(row, text_end, w));
            lines.push(Line::from(spans));
        } else {
            // Full row of base letters with spiral coloring
            let spans = build_base_spans(row, 0, w);
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
