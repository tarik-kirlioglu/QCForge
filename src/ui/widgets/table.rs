use ratatui::style::{Color, Modifier, Style};

pub fn header_style() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

pub fn row_style() -> Style {
    Style::default().fg(Color::White)
}

pub fn highlight_style() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}
