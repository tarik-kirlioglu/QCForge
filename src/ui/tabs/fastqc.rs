use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::parser::types::{FastqcReport, ModuleStatus};
use crate::ui::widgets::table as table_style;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let results = match &state.qc_results {
        Some(r) => r,
        None => return,
    };

    if results.fastqc_reports.is_empty() {
        let msg = Paragraph::new("No FastQC files found.")
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(msg, area);
        return;
    }

    let report = &results.fastqc_reports[state.fastqc_selected];
    let file_count = results.fastqc_reports.len();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(area);

    render_file_header(frame, chunks[0], report, state.fastqc_selected, file_count);
    render_content(frame, chunks[1], report);
}

fn render_file_header(
    frame: &mut Frame,
    area: Rect,
    report: &FastqcReport,
    selected: usize,
    total: usize,
) {
    let name = if report.sample_name.is_empty() {
        report
            .source_file
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        report.sample_name.clone()
    };

    let header = Line::from(vec![
        Span::styled("FastQC: ", Style::default().fg(Color::Cyan)),
        Span::styled(
            &name,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  [{}/{}]", selected + 1, total),
            Style::default().fg(Color::Gray),
        ),
        Span::styled("  n:Next p:Prev", Style::default().fg(Color::DarkGray)),
    ]);

    frame.render_widget(Paragraph::new(header), area);
}

fn render_content(frame: &mut Frame, area: Rect, report: &FastqcReport) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Percentage(55),
        ])
        .split(area);

    // Left: basic stats + module statuses
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11),
            Constraint::Min(0),
        ])
        .split(chunks[0]);

    render_basic_stats(frame, left_chunks[0], report);
    render_module_statuses(frame, left_chunks[1], report);

    // Right: per base quality chart
    render_per_base_quality(frame, chunks[1], report);
}

fn render_basic_stats(frame: &mut Frame, area: Rect, report: &FastqcReport) {
    let s = &report.basic_statistics;

    let data: Vec<(&str, String)> = vec![
        ("Filename", s.filename.clone()),
        ("File Type", s.file_type.clone()),
        ("Encoding", s.encoding.clone()),
        ("Total Sequences", format_number(s.total_sequences)),
        ("Poor Quality", format_number(s.sequences_flagged_poor_quality)),
        ("Sequence Length", s.sequence_length.clone()),
        ("GC %", format!("{}%", s.percent_gc)),
    ];

    let rows: Vec<Row> = data
        .iter()
        .map(|(label, value)| {
            Row::new(vec![
                Cell::from(*label).style(Style::default().fg(Color::White)),
                Cell::from(value.clone()).style(Style::default().fg(Color::Green)),
            ])
        })
        .collect();

    let widths = [Constraint::Percentage(45), Constraint::Percentage(55)];

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .title(" Basic Statistics ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(table, area);
}

fn render_module_statuses(frame: &mut Frame, area: Rect, report: &FastqcReport) {
    let rows: Vec<Row> = report
        .module_statuses
        .iter()
        .map(|(name, status)| {
            let (status_str, color) = match status {
                ModuleStatus::Pass => ("PASS", Color::Green),
                ModuleStatus::Warn => ("WARN", Color::Yellow),
                ModuleStatus::Fail => ("FAIL", Color::Red),
            };

            Row::new(vec![
                Cell::from(name.clone()).style(Style::default().fg(Color::White)),
                Cell::from(status_str).style(
                    Style::default()
                        .fg(color)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        })
        .collect();

    let widths = [Constraint::Min(30), Constraint::Length(6)];

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .title(" Module Status ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(table, area);
}

fn render_per_base_quality(frame: &mut Frame, area: Rect, report: &FastqcReport) {
    if report.per_base_quality.is_empty() {
        let msg = Paragraph::new("No per-base quality data available.")
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .title(" Per Base Quality ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        frame.render_widget(msg, area);
        return;
    }

    // ASCII quality plot using bar representation
    let rows: Vec<Row> = report
        .per_base_quality
        .iter()
        .map(|pbq| {
            let mean_color = if pbq.mean >= 28.0 {
                Color::Green
            } else if pbq.mean >= 20.0 {
                Color::Yellow
            } else {
                Color::Red
            };

            let bar_width = ((pbq.mean / 40.0) * 20.0).min(20.0) as usize;
            let bar = "█".repeat(bar_width);

            Row::new(vec![
                Cell::from(pbq.base.clone()).style(Style::default().fg(Color::White)),
                Cell::from(format!("{:.1}", pbq.mean)).style(Style::default().fg(mean_color)),
                Cell::from(format!("{:.0}", pbq.median)).style(Style::default().fg(Color::Gray)),
                Cell::from(format!("{:.0}-{:.0}", pbq.lower_quartile, pbq.upper_quartile))
                    .style(Style::default().fg(Color::DarkGray)),
                Cell::from(bar).style(Style::default().fg(mean_color)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Length(6),
        Constraint::Length(4),
        Constraint::Length(7),
        Constraint::Min(10),
    ];

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec![
                Cell::from("Base").style(table_style::header_style()),
                Cell::from("Mean").style(table_style::header_style()),
                Cell::from("Med").style(table_style::header_style()),
                Cell::from("IQR").style(table_style::header_style()),
                Cell::from("Quality").style(table_style::header_style()),
            ])
        )
        .block(
            Block::default()
                .title(" Per Base Quality ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(table, area);
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
