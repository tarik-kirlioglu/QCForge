use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::parser::types::BcftoolsStats;
use crate::ui::widgets::table as table_style;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let results = match &state.qc_results {
        Some(r) => r,
        None => return,
    };

    if results.bcftools_reports.is_empty() {
        let msg = Paragraph::new("No bcftools stats files found.")
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(msg, area);
        return;
    }

    let report = &results.bcftools_reports[state.bcftools_selected];
    let file_count = results.bcftools_reports.len();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(area);

    render_file_header(frame, chunks[0], report, state.bcftools_selected, file_count);
    render_content(frame, chunks[1], report);
}

fn render_file_header(
    frame: &mut Frame,
    area: Rect,
    report: &BcftoolsStats,
    selected: usize,
    total: usize,
) {
    let filename = report
        .source_file
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let header = Line::from(vec![
        Span::styled("bcftools: ", Style::default().fg(Color::Cyan)),
        Span::styled(
            &filename,
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

fn render_content(frame: &mut Frame, area: Rect, report: &BcftoolsStats) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    // Left: summary + Ts/Tv
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(14),
            Constraint::Min(0),
        ])
        .split(chunks[0]);

    render_summary_table(frame, left_chunks[0], report);
    render_tstv(frame, left_chunks[1], report);

    // Right: substitution types + indel distribution
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(16),
            Constraint::Min(0),
        ])
        .split(chunks[1]);

    render_substitution_types(frame, right_chunks[0], report);
    render_indel_distribution(frame, right_chunks[1], report);
}

fn render_summary_table(frame: &mut Frame, area: Rect, report: &BcftoolsStats) {
    let s = &report.summary;

    let data: Vec<(&str, String)> = vec![
        ("Samples", format_number(s.num_samples)),
        ("Records", format_number(s.num_records)),
        ("SNPs", format_number(s.num_snps)),
        ("MNPs", format_number(s.num_mnps)),
        ("Indels", format_number(s.num_indels)),
        ("Others", format_number(s.num_others)),
        ("No-ALTs", format_number(s.num_no_alts)),
        ("Multi-allelic", format_number(s.num_multiallelic_sites)),
        ("Multi-allelic SNPs", format_number(s.num_multiallelic_snp_sites)),
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

    let widths = [Constraint::Percentage(55), Constraint::Percentage(45)];

    let table = Table::new(rows, widths)
        .header(Row::new(vec![
            Cell::from("Metric").style(table_style::header_style()),
            Cell::from("Value").style(table_style::header_style()),
        ]))
        .block(
            Block::default()
                .title(" Summary Numbers ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(table, area);
}

fn render_tstv(frame: &mut Frame, area: Rect, report: &BcftoolsStats) {
    let t = &report.tstv;

    let ratio_color = if t.ts_tv_ratio >= 2.0 {
        Color::Green
    } else if t.ts_tv_ratio >= 1.5 {
        Color::Yellow
    } else {
        Color::Red
    };

    let data: Vec<(&str, String, Color)> = vec![
        ("Transitions (Ts)", format_number(t.ts), Color::White),
        ("Transversions (Tv)", format_number(t.tv), Color::White),
        ("Ts/Tv Ratio", format!("{:.2}", t.ts_tv_ratio), ratio_color),
        (
            "Ts (1st ALT)",
            format_number(t.ts_first_alt),
            Color::Gray,
        ),
        (
            "Tv (1st ALT)",
            format_number(t.tv_first_alt),
            Color::Gray,
        ),
        (
            "Ts/Tv (1st ALT)",
            format!("{:.2}", t.ts_tv_ratio_first_alt),
            ratio_color,
        ),
    ];

    let rows: Vec<Row> = data
        .iter()
        .map(|(label, value, color)| {
            Row::new(vec![
                Cell::from(*label).style(Style::default().fg(Color::White)),
                Cell::from(value.clone()).style(Style::default().fg(*color)),
            ])
        })
        .collect();

    let widths = [Constraint::Percentage(55), Constraint::Percentage(45)];

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .title(" Ts/Tv ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(table, area);
}

fn render_substitution_types(frame: &mut Frame, area: Rect, report: &BcftoolsStats) {
    if report.substitution_types.is_empty() {
        return;
    }

    let max_count = report
        .substitution_types
        .iter()
        .map(|s| s.count)
        .max()
        .unwrap_or(1);

    let rows: Vec<Row> = report
        .substitution_types
        .iter()
        .map(|st| {
            let bar_width = ((st.count as f64 / max_count as f64) * 20.0) as usize;
            let bar = "█".repeat(bar_width);

            let color = if st.sub_type.contains("A>G")
                || st.sub_type.contains("G>A")
                || st.sub_type.contains("C>T")
                || st.sub_type.contains("T>C")
            {
                Color::Cyan // transitions
            } else {
                Color::Magenta // transversions
            };

            Row::new(vec![
                Cell::from(st.sub_type.clone()).style(Style::default().fg(Color::White)),
                Cell::from(format_number(st.count)).style(Style::default().fg(Color::Green)),
                Cell::from(bar).style(Style::default().fg(color)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(5),
        Constraint::Length(8),
        Constraint::Min(10),
    ];

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .title(" Substitution Types (Ts=cyan Tv=magenta) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(table, area);
}

fn render_indel_distribution(frame: &mut Frame, area: Rect, report: &BcftoolsStats) {
    if report.indel_dist.is_empty() {
        return;
    }

    let max_count = report
        .indel_dist
        .iter()
        .map(|i| i.count)
        .max()
        .unwrap_or(1);

    let rows: Vec<Row> = report
        .indel_dist
        .iter()
        .map(|idd| {
            let bar_width = ((idd.count as f64 / max_count as f64) * 20.0) as usize;
            let bar = "█".repeat(bar_width);

            let color = if idd.length < 0 {
                Color::Red // deletions
            } else {
                Color::Green // insertions
            };

            Row::new(vec![
                Cell::from(format!("{:+}", idd.length))
                    .style(Style::default().fg(Color::White)),
                Cell::from(format_number(idd.count))
                    .style(Style::default().fg(Color::Green)),
                Cell::from(bar).style(Style::default().fg(color)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(5),
        Constraint::Length(8),
        Constraint::Min(10),
    ];

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .title(" InDel Distribution (del=red ins=green) ")
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
