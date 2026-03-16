use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::parser::types::SamtoolsStats;
use crate::ui::widgets::gauge::{duplication_style, mapping_style};
use crate::ui::widgets::table as table_style;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let results = match &state.qc_results {
        Some(r) => r,
        None => return,
    };

    if results.samtools_reports.is_empty() {
        let msg = Paragraph::new("No samtools stats files found.")
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(msg, area);
        return;
    }

    let report = &results.samtools_reports[state.samtools_selected];
    let file_count = results.samtools_reports.len();

    // Main layout: header + content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // file selector
            Constraint::Min(0),   // content
        ])
        .split(area);

    render_file_header(frame, chunks[0], report, state.samtools_selected, file_count);
    render_content(frame, chunks[1], report);
}

fn render_file_header(
    frame: &mut Frame,
    area: Rect,
    report: &SamtoolsStats,
    selected: usize,
    total: usize,
) {
    let filename = report
        .source_file
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let header = Line::from(vec![
        Span::styled("samtools: ", Style::default().fg(Color::Cyan)),
        Span::styled(&filename, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("  [{}/{}]", selected + 1, total),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            "  n:Next p:Prev",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    frame.render_widget(Paragraph::new(header), area);
}

fn render_content(frame: &mut Frame, area: Rect, report: &SamtoolsStats) {
    // Split into left (summary table) and right (gauges)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(area);

    render_summary_table(frame, chunks[0], report);
    render_gauges(frame, chunks[1], report);
}

fn render_summary_table(frame: &mut Frame, area: Rect, report: &SamtoolsStats) {
    let s = &report.summary;

    let data: Vec<(&str, String)> = vec![
        ("Total Reads", format_number(s.raw_total_sequences)),
        ("Reads Mapped", format_number(s.reads_mapped)),
        ("Reads Unmapped", format_number(s.reads_unmapped)),
        ("Reads Duplicated", format_number(s.reads_duplicated)),
        ("Reads Properly Paired", format_number(s.reads_properly_paired)),
        ("Reads QC Failed", format_number(s.reads_qc_failed)),
        ("Reads MQ0", format_number(s.reads_mq0)),
        ("Total Length", format_number(s.total_length)),
        ("Bases Mapped", format_number(s.bases_mapped)),
        ("Bases Mapped (CIGAR)", format_number(s.bases_mapped_cigar)),
        ("Error Rate", format!("{:.4}", s.error_rate)),
        ("Average Length", format!("{:.1}", s.average_length)),
        ("Average Quality", format!("{:.1}", s.average_quality)),
        ("Insert Size Avg", format!("{:.1}", s.insert_size_average)),
        ("Insert Size Std", format!("{:.1}", s.insert_size_std_deviation)),
        ("Diff Chr Pairs", format_number(s.pairs_on_different_chromosomes)),
    ];

    let rows: Vec<Row> = data
        .iter()
        .map(|(label, value)| make_row(label, value))
        .collect();

    let widths = [Constraint::Percentage(55), Constraint::Percentage(45)];

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec![
                Cell::from("Metric").style(table_style::header_style()),
                Cell::from("Value").style(table_style::header_style()),
            ])
        )
        .block(
            Block::default()
                .title(" Summary Numbers ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .row_highlight_style(table_style::highlight_style());

    frame.render_widget(table, area);
}

fn render_gauges(frame: &mut Frame, area: Rect, report: &SamtoolsStats) {
    let s = &report.summary;
    let mapping_pct = s.mapping_percent();
    let dup_pct = s.duplication_percent();
    let paired_pct = s.properly_paired_percent();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    // Mapping rate gauge
    let mapping_gauge = Gauge::default()
        .block(
            Block::default()
                .title(" Mapping Rate ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .gauge_style(mapping_style(mapping_pct))
        .percent(mapping_pct.min(100.0) as u16)
        .label(format!("{:.1}%", mapping_pct));
    frame.render_widget(mapping_gauge, chunks[0]);

    // Duplication rate gauge
    let dup_gauge = Gauge::default()
        .block(
            Block::default()
                .title(" Duplication Rate ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .gauge_style(duplication_style(dup_pct))
        .percent(dup_pct.min(100.0) as u16)
        .label(format!("{:.1}%", dup_pct));
    frame.render_widget(dup_gauge, chunks[2]);

    // Properly paired gauge
    let paired_gauge = Gauge::default()
        .block(
            Block::default()
                .title(" Properly Paired ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .gauge_style(mapping_style(paired_pct))
        .percent(paired_pct.min(100.0) as u16)
        .label(format!("{:.1}%", paired_pct));
    frame.render_widget(paired_gauge, chunks[4]);
}

fn make_row<'a>(label: &'a str, value: &'a str) -> Row<'a> {
    Row::new(vec![
        Cell::from(label).style(Style::default().fg(Color::White)),
        Cell::from(value.to_string()).style(Style::default().fg(Color::Green)),
    ])
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
