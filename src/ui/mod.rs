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
    // Loading state
    if state.loading {
        render_loading(frame);
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
        ActiveTab::Samtools => tabs::samtools::render(frame, app_layout.content, state),
        ActiveTab::Bcftools => tabs::bcftools::render(frame, app_layout.content, state),
        ActiveTab::Fastqc => tabs::fastqc::render(frame, app_layout.content, state),
    }

    // Help overlay
    if state.show_help {
        render_help_overlay(frame);
    }
}

fn render_loading(frame: &mut Frame) {
    let area = frame.area();
    let text = Paragraph::new(Line::from(vec![
        Span::styled(
            "  Loading QC data...",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(
        Block::default()
            .title(" QCForge ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(text, area);
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
