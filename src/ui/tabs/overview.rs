use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table};
use ratatui::Frame;

use crate::app::state::{AppState, SortColumn};
use crate::parser::types::{ModuleStatus, QcResults};
use crate::ui::widgets::gauge::mapping_style;
use crate::ui::widgets::table as table_style;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let results = match &state.qc_results {
        Some(r) => r,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // discovery summary + quick stats
            Constraint::Min(0),    // file list
        ])
        .split(area);

    render_top_panel(frame, chunks[0], results);
    render_file_list(frame, chunks[1], state, results);
}

fn render_top_panel(frame: &mut Frame, area: Rect, results: &QcResults) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Percentage(35),
            Constraint::Percentage(30),
        ])
        .split(area);

    render_discovery(frame, chunks[0], results);
    render_quick_stats(frame, chunks[1], results);
    render_gauges(frame, chunks[2], results);
}

fn render_discovery(frame: &mut Frame, area: Rect, results: &QcResults) {
    let sam_count = results.samtools_reports.len();
    let bcf_count = results.bcftools_reports.len();
    let fqc_count = results.fastqc_reports.len();
    let total = sam_count + bcf_count + fqc_count;

    let lines = vec![
        Line::from(vec![
            Span::styled("  Total files: ", Style::default().fg(Color::White)),
            Span::styled(
                total.to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  samtools:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} file(s)", sam_count),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled("  bcftools:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} file(s)", bcf_count),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled("  FastQC:    ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} file(s)", fqc_count),
                Style::default().fg(Color::Green),
            ),
        ]),
    ];

    let block = Block::default()
        .title(" Files Discovered ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_quick_stats(frame: &mut Frame, area: Rect, results: &QcResults) {
    let mut lines = Vec::new();

    // Aggregate samtools stats
    if !results.samtools_reports.is_empty() {
        let total_reads: u64 = results
            .samtools_reports
            .iter()
            .map(|r| r.summary.raw_total_sequences)
            .sum();
        let total_mapped: u64 = results
            .samtools_reports
            .iter()
            .map(|r| r.summary.reads_mapped)
            .sum();
        let avg_error: f64 = results
            .samtools_reports
            .iter()
            .map(|r| r.summary.error_rate)
            .sum::<f64>()
            / results.samtools_reports.len() as f64;

        lines.push(Line::from(vec![
            Span::styled("  Total Reads: ", Style::default().fg(Color::Gray)),
            Span::styled(format_large_number(total_reads), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Mapped:      ", Style::default().fg(Color::Gray)),
            Span::styled(format_large_number(total_mapped), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Avg Error:   ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.4}", avg_error), Style::default().fg(Color::White)),
        ]));
    }

    // Aggregate bcftools stats
    if !results.bcftools_reports.is_empty() {
        let total_variants: u64 = results
            .bcftools_reports
            .iter()
            .map(|r| r.summary.num_records)
            .sum();

        if !lines.is_empty() {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(vec![
            Span::styled("  Variants:    ", Style::default().fg(Color::Gray)),
            Span::styled(format_number(total_variants), Style::default().fg(Color::White)),
        ]));
    }

    let block = Block::default()
        .title(" Quick Stats ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_gauges(frame: &mut Frame, area: Rect, results: &QcResults) {
    if results.samtools_reports.is_empty() {
        let block = Block::default()
            .title(" Quality ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        frame.render_widget(
            Paragraph::new("  No alignment data").style(Style::default().fg(Color::Gray)).block(block),
            area,
        );
        return;
    }

    // Average mapping rate across all samtools reports
    let avg_mapping: f64 = results
        .samtools_reports
        .iter()
        .map(|r| r.summary.mapping_percent())
        .sum::<f64>()
        / results.samtools_reports.len() as f64;

    let avg_dup: f64 = results
        .samtools_reports
        .iter()
        .map(|r| r.summary.duplication_percent())
        .sum::<f64>()
        / results.samtools_reports.len() as f64;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let map_gauge = Gauge::default()
        .block(
            Block::default()
                .title(" Avg Mapping ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .gauge_style(mapping_style(avg_mapping))
        .percent(avg_mapping.min(100.0) as u16)
        .label(format!("{:.1}%", avg_mapping));
    frame.render_widget(map_gauge, chunks[0]);

    let dup_color = if avg_dup <= 10.0 {
        Color::Green
    } else if avg_dup <= 20.0 {
        Color::Yellow
    } else {
        Color::Red
    };
    let dup_gauge = Gauge::default()
        .block(
            Block::default()
                .title(" Avg Duplication ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .gauge_style(Style::default().fg(dup_color))
        .percent(avg_dup.min(100.0) as u16)
        .label(format!("{:.1}%", avg_dup));
    frame.render_widget(dup_gauge, chunks[2]);
}

struct OverviewRow {
    filename: String,
    tool: String,
    summary: String,
    status: String,
    sort_key_summary: f64,
    sort_key_status: f64,
    tool_color: Color,
    status_color: Color,
    status_bold: bool,
}

fn build_overview_rows(results: &QcResults) -> Vec<OverviewRow> {
    let mut rows = Vec::new();

    for report in &results.samtools_reports {
        let fname = report
            .source_file
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        let s = &report.summary;
        let mapping = s.mapping_percent();
        rows.push(OverviewRow {
            filename: fname,
            tool: "samtools".into(),
            summary: format!("{} reads", format_large_number(s.raw_total_sequences)),
            status: format!("{:.1}% mapped", mapping),
            sort_key_summary: s.raw_total_sequences as f64,
            sort_key_status: mapping,
            tool_color: Color::Cyan,
            status_color: quality_color(mapping, 90.0, 80.0),
            status_bold: false,
        });
    }

    for report in &results.bcftools_reports {
        let fname = report
            .source_file
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        let s = &report.summary;
        rows.push(OverviewRow {
            filename: fname,
            tool: "bcftools".into(),
            summary: format!("{} records", format_number(s.num_records)),
            status: format!("Ts/Tv {:.2}", report.tstv.ts_tv_ratio),
            sort_key_summary: s.num_records as f64,
            sort_key_status: report.tstv.ts_tv_ratio,
            tool_color: Color::Magenta,
            status_color: Color::Green,
            status_bold: false,
        });
    }

    for report in &results.fastqc_reports {
        let fname = if report.sample_name.is_empty() {
            report
                .source_file
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default()
        } else {
            report.sample_name.clone()
        };

        let (fail_count, warn_count) =
            report
                .module_statuses
                .iter()
                .fold((0u32, 0u32), |(f, w), (_, s)| match s {
                    ModuleStatus::Fail => (f + 1, w),
                    ModuleStatus::Warn => (f, w + 1),
                    _ => (f, w),
                });

        let status_text = if fail_count > 0 {
            format!("{} FAIL", fail_count)
        } else if warn_count > 0 {
            format!("{} WARN", warn_count)
        } else {
            "ALL PASS".to_string()
        };

        let status_color = if fail_count > 0 {
            Color::Red
        } else if warn_count > 0 {
            Color::Yellow
        } else {
            Color::Green
        };

        // Sort key: FAIL=0 (worst), WARN=50, PASS=100 (best)
        let sort_key_status = if fail_count > 0 {
            0.0
        } else if warn_count > 0 {
            50.0
        } else {
            100.0
        };

        rows.push(OverviewRow {
            filename: fname,
            tool: "FastQC".into(),
            summary: format!(
                "{} seqs",
                format_large_number(report.basic_statistics.total_sequences)
            ),
            status: status_text,
            sort_key_summary: report.basic_statistics.total_sequences as f64,
            sort_key_status,
            tool_color: Color::Yellow,
            status_color,
            status_bold: true,
        });
    }

    rows
}

fn render_file_list(frame: &mut Frame, area: Rect, state: &AppState, results: &QcResults) {
    let mut rows = build_overview_rows(results);

    // Sort
    rows.sort_by(|a, b| {
        let cmp = match state.sort_column {
            SortColumn::File => a.filename.to_lowercase().cmp(&b.filename.to_lowercase()),
            SortColumn::Tool => a.tool.cmp(&b.tool),
            SortColumn::Summary => a
                .sort_key_summary
                .partial_cmp(&b.sort_key_summary)
                .unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::Status => a
                .sort_key_status
                .partial_cmp(&b.sort_key_status)
                .unwrap_or(std::cmp::Ordering::Equal),
        };
        if state.sort_ascending {
            cmp
        } else {
            cmp.reverse()
        }
    });

    // Filter
    let filter = if state.search_active {
        &state.search_input
    } else {
        &state.search_confirmed
    };
    if !filter.is_empty() {
        let filter_lower = filter.to_lowercase();
        rows.retain(|row| row.filename.to_lowercase().contains(&filter_lower));
    }

    // Convert to ratatui Rows
    let table_rows: Vec<Row> = rows
        .iter()
        .map(|r| {
            let mut status_style = Style::default().fg(r.status_color);
            if r.status_bold {
                status_style = status_style.add_modifier(Modifier::BOLD);
            }
            Row::new(vec![
                Cell::from(r.filename.as_str()).style(Style::default().fg(Color::White)),
                Cell::from(r.tool.as_str()).style(Style::default().fg(r.tool_color)),
                Cell::from(r.summary.as_str()).style(Style::default().fg(Color::Gray)),
                Cell::from(r.status.as_str()).style(status_style),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(35),
        Constraint::Percentage(12),
        Constraint::Percentage(28),
        Constraint::Percentage(25),
    ];

    let indicator = if state.sort_ascending {
        " \u{25b2}"
    } else {
        " \u{25bc}"
    };

    let headers = ["File", "Tool", "Summary", "Status"];
    let sort_idx = match state.sort_column {
        SortColumn::File => 0,
        SortColumn::Tool => 1,
        SortColumn::Summary => 2,
        SortColumn::Status => 3,
    };

    let header_cells: Vec<Cell> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| {
            if i == sort_idx {
                Cell::from(format!("{}{}", h, indicator)).style(table_style::header_style())
            } else {
                Cell::from(*h).style(table_style::header_style())
            }
        })
        .collect();

    let table = Table::new(table_rows, widths)
        .header(Row::new(header_cells))
        .block(
            Block::default()
                .title(" All QC Files ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .row_highlight_style(table_style::highlight_style());

    frame.render_widget(table, area);
}

fn quality_color(percent: f64, good: f64, warn: f64) -> Color {
    if percent >= good {
        Color::Green
    } else if percent >= warn {
        Color::Yellow
    } else {
        Color::Red
    }
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

fn format_large_number(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
