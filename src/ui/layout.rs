use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::Frame;

use crate::app::state::{ActiveTab, AppState};

/// Main layout structure returned to the caller
pub struct AppLayout {
    pub content: Rect,
}

pub fn render_chrome(frame: &mut Frame, state: &AppState) -> AppLayout {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Length(2), // tabs
            Constraint::Min(0),   // content
            Constraint::Length(1), // footer
        ])
        .split(size);

    render_header(frame, chunks[0], state);
    render_tabs(frame, chunks[1], state);
    render_footer(frame, chunks[3]);

    AppLayout {
        content: chunks[2],
    }
}

fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let scan_path = state
        .qc_results
        .as_ref()
        .map(|r| r.scan_path.display().to_string())
        .unwrap_or_else(|| "...".to_string());

    let header = Line::from(vec![
        Span::styled(
            " QCForge v0.1.0 ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("── ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("[{}]", scan_path),
            Style::default().fg(Color::White),
        ),
    ]);

    frame.render_widget(Paragraph::new(header), area);
}

fn render_tabs(frame: &mut Frame, area: Rect, state: &AppState) {
    let titles: Vec<Line> = ActiveTab::all()
        .iter()
        .map(|t| Line::from(t.title()))
        .collect();

    let selected = ActiveTab::all()
        .iter()
        .position(|t| *t == state.active_tab)
        .unwrap_or(0);

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .select(selected)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );

    frame.render_widget(tabs, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Line::from(vec![
        Span::styled(" ←/→", Style::default().fg(Color::Cyan)),
        Span::styled(" Tabs  ", Style::default().fg(Color::Gray)),
        Span::styled("j/k", Style::default().fg(Color::Cyan)),
        Span::styled(" Scroll  ", Style::default().fg(Color::Gray)),
        Span::styled("n/p", Style::default().fg(Color::Cyan)),
        Span::styled(" File  ", Style::default().fg(Color::Gray)),
        Span::styled("?", Style::default().fg(Color::Cyan)),
        Span::styled(" Help  ", Style::default().fg(Color::Gray)),
        Span::styled("q", Style::default().fg(Color::Cyan)),
        Span::styled(" Quit", Style::default().fg(Color::Gray)),
    ]);

    frame.render_widget(Paragraph::new(footer), area);
}
