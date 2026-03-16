use ratatui::style::{Color, Style};

/// Returns a color based on a percentage value and quality thresholds
pub fn quality_color(percent: f64, good_threshold: f64, warn_threshold: f64) -> Color {
    if percent >= good_threshold {
        Color::Green
    } else if percent >= warn_threshold {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// Returns a gauge style for mapping percentage (good > 90%, warn > 80%)
pub fn mapping_style(percent: f64) -> Style {
    Style::default().fg(quality_color(percent, 90.0, 80.0))
}

/// Returns a gauge style for duplication percentage (inverted: lower is better)
pub fn duplication_style(percent: f64) -> Style {
    let color = if percent <= 10.0 {
        Color::Green
    } else if percent <= 20.0 {
        Color::Yellow
    } else {
        Color::Red
    };
    Style::default().fg(color)
}
