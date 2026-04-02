use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;

use crate::app::state::{AppState, SummarySortColumn};
use crate::parser::types::{ModuleStatus, QcResults};
use crate::threshold::{QcLevel, ThresholdConfig};
use crate::ui::widgets::table as table_style;

pub struct SummaryRow {
    pub filename: String,
    pub tool: String,
    pub reads: Option<u64>,
    pub mapped_pct: Option<f64>,
    pub dup_pct: Option<f64>,
    pub error_rate: Option<f64>,
    pub avg_quality: Option<f64>,
    pub variants: Option<u64>,
    pub snps: Option<u64>,
    pub indels: Option<u64>,
    pub ts_tv: Option<f64>,
    pub total_seqs: Option<u64>,
    pub gc_pct: Option<f64>,
    pub pass_modules: Option<u32>,
    pub warn_modules: Option<u32>,
    pub fail_modules: Option<u32>,
    pub overall_qc: QcLevel,
}

impl SummaryRow {
    fn sort_key(&self, col: SummarySortColumn) -> SortKey {
        match col {
            SummarySortColumn::File => SortKey::Str(self.filename.to_lowercase()),
            SummarySortColumn::Tool => SortKey::Str(self.tool.clone()),
            SummarySortColumn::Reads => SortKey::Num(self.reads.map(|v| v as f64)),
            SummarySortColumn::MappedPct => SortKey::Num(self.mapped_pct),
            SummarySortColumn::DupPct => SortKey::Num(self.dup_pct),
            SummarySortColumn::ErrorRate => SortKey::Num(self.error_rate),
            SummarySortColumn::AvgQuality => SortKey::Num(self.avg_quality),
            SummarySortColumn::Variants => SortKey::Num(self.variants.map(|v| v as f64)),
            SummarySortColumn::Snps => SortKey::Num(self.snps.map(|v| v as f64)),
            SummarySortColumn::Indels => SortKey::Num(self.indels.map(|v| v as f64)),
            SummarySortColumn::TsTv => SortKey::Num(self.ts_tv),
            SummarySortColumn::TotalSeqs => SortKey::Num(self.total_seqs.map(|v| v as f64)),
            SummarySortColumn::GcPct => SortKey::Num(self.gc_pct),
            SummarySortColumn::PassModules => SortKey::Num(self.pass_modules.map(|v| v as f64)),
            SummarySortColumn::WarnModules => SortKey::Num(self.warn_modules.map(|v| v as f64)),
            SummarySortColumn::FailModules => SortKey::Num(self.fail_modules.map(|v| v as f64)),
            SummarySortColumn::OverallQc => SortKey::Num(Some(match self.overall_qc {
                QcLevel::Pass => 2.0,
                QcLevel::Warn => 1.0,
                QcLevel::Fail => 0.0,
            })),
        }
    }
}

enum SortKey {
    Str(String),
    Num(Option<f64>),
}

impl SortKey {
    fn cmp(&self, other: &SortKey) -> std::cmp::Ordering {
        match (self, other) {
            (SortKey::Str(a), SortKey::Str(b)) => a.cmp(b),
            (SortKey::Num(a), SortKey::Num(b)) => {
                match (a, b) {
                    (Some(x), Some(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                    (Some(_), None) => std::cmp::Ordering::Less, // values before None
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            }
            _ => std::cmp::Ordering::Equal,
        }
    }
}

pub fn build_summary_rows(results: &QcResults, thresholds: &ThresholdConfig) -> Vec<SummaryRow> {
    let mut rows = Vec::new();

    for report in &results.samtools_reports {
        let fname = report
            .source_file
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        let s = &report.summary;
        let mapped = s.mapping_percent();
        let dup = s.duplication_percent();
        let overall = thresholds.evaluate_sample(
            Some(mapped),
            Some(dup),
            Some(s.error_rate),
            None,
            None,
        );
        rows.push(SummaryRow {
            filename: fname,
            tool: "samtools".into(),
            reads: Some(s.raw_total_sequences),
            mapped_pct: Some(mapped),
            dup_pct: Some(dup),
            error_rate: Some(s.error_rate),
            avg_quality: Some(s.average_quality),
            variants: None,
            snps: None,
            indels: None,
            ts_tv: None,
            total_seqs: None,
            gc_pct: None,
            pass_modules: None,
            warn_modules: None,
            fail_modules: None,
            overall_qc: overall,
        });
    }

    for report in &results.bcftools_reports {
        let fname = report
            .source_file
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        let overall = thresholds.evaluate_sample(None, None, None, Some(report.tstv.ts_tv_ratio), None);
        rows.push(SummaryRow {
            filename: fname,
            tool: "bcftools".into(),
            reads: None,
            mapped_pct: None,
            dup_pct: None,
            error_rate: None,
            avg_quality: None,
            variants: Some(report.summary.num_records),
            snps: Some(report.summary.num_snps),
            indels: Some(report.summary.num_indels),
            ts_tv: Some(report.tstv.ts_tv_ratio),
            total_seqs: None,
            gc_pct: None,
            pass_modules: None,
            warn_modules: None,
            fail_modules: None,
            overall_qc: overall,
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
        let (p, w, f) =
            report
                .module_statuses
                .iter()
                .fold((0u32, 0u32, 0u32), |(p, w, f), (_, s)| match s {
                    ModuleStatus::Pass => (p + 1, w, f),
                    ModuleStatus::Warn => (p, w + 1, f),
                    ModuleStatus::Fail => (p, w, f + 1),
                });
        let gc = report.basic_statistics.percent_gc;
        let overall = thresholds.evaluate_sample(None, None, None, None, Some(gc));
        // If FastQC has FAIL modules, override to at least Warn
        let overall = if f > 0 {
            overall.worst(QcLevel::Warn)
        } else {
            overall
        };
        rows.push(SummaryRow {
            filename: fname,
            tool: "FastQC".into(),
            reads: None,
            mapped_pct: None,
            dup_pct: None,
            error_rate: None,
            avg_quality: None,
            variants: None,
            snps: None,
            indels: None,
            ts_tv: None,
            total_seqs: Some(report.basic_statistics.total_sequences),
            gc_pct: Some(gc),
            pass_modules: Some(p),
            warn_modules: Some(w),
            fail_modules: Some(f),
            overall_qc: overall,
        });
    }

    rows
}

// Column definitions: (header_name, width)
const METRIC_COLUMNS: &[(&str, u16)] = &[
    ("Tool", 10),
    ("Reads", 12),
    ("Map%", 8),
    ("Dup%", 8),
    ("Error", 9),
    ("AvgQ", 7),
    ("Vars", 10),
    ("SNPs", 10),
    ("InDels", 8),
    ("Ts/Tv", 8),
    ("Seqs", 12),
    ("GC%", 7),
    ("P", 5),
    ("W", 5),
    ("F", 5),
    ("QC", 6),
];

fn format_opt_u64(v: Option<u64>) -> String {
    match v {
        Some(n) => format_number(n),
        None => "-".into(),
    }
}

fn format_opt_f64(v: Option<f64>, decimals: usize) -> String {
    match v {
        Some(n) => format!("{:.prec$}", n, prec = decimals),
        None => "-".into(),
    }
}

fn format_opt_u32(v: Option<u32>) -> String {
    match v {
        Some(n) => n.to_string(),
        None => "-".into(),
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

fn metric_cell_value(row: &SummaryRow, col_idx: usize) -> String {
    match col_idx {
        0 => row.tool.clone(),
        1 => format_opt_u64(row.reads),
        2 => format_opt_f64(row.mapped_pct, 1),
        3 => format_opt_f64(row.dup_pct, 1),
        4 => format_opt_f64(row.error_rate, 4),
        5 => format_opt_f64(row.avg_quality, 1),
        6 => format_opt_u64(row.variants),
        7 => format_opt_u64(row.snps),
        8 => format_opt_u64(row.indels),
        9 => format_opt_f64(row.ts_tv, 2),
        10 => format_opt_u64(row.total_seqs),
        11 => format_opt_f64(row.gc_pct, 1),
        12 => format_opt_u32(row.pass_modules),
        13 => format_opt_u32(row.warn_modules),
        14 => format_opt_u32(row.fail_modules),
        15 => row.overall_qc.label().to_string(),
        _ => String::new(),
    }
}

fn metric_cell_color(row: &SummaryRow, col_idx: usize, thresholds: &ThresholdConfig) -> Color {
    match col_idx {
        0 => match row.tool.as_str() {
            "samtools" => Color::Cyan,
            "bcftools" => Color::Magenta,
            "FastQC" => Color::Yellow,
            _ => Color::White,
        },
        2 => row.mapped_pct.map_or(Color::DarkGray, |v| thresholds.mapping_rate.color(v)),
        3 => row.dup_pct.map_or(Color::DarkGray, |v| thresholds.duplication_rate.color(v)),
        4 => row.error_rate.map_or(Color::DarkGray, |v| thresholds.error_rate.color(v)),
        9 => row.ts_tv.map_or(Color::DarkGray, |v| thresholds.ts_tv_ratio.color(v)),
        11 => row.gc_pct.map_or(Color::DarkGray, |v| {
            thresholds.gc_deviation.color((v - 50.0).abs())
        }),
        15 => match row.overall_qc {
            QcLevel::Pass => Color::Green,
            QcLevel::Warn => Color::Yellow,
            QcLevel::Fail => Color::Red,
        },
        _ => Color::White,
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let results = match &state.qc_results {
        Some(r) => r,
        None => return,
    };

    let mut rows = build_summary_rows(results, &state.thresholds);

    // Sort
    rows.sort_by(|a, b| {
        let cmp = a.sort_key(state.summary_sort_column).cmp(&b.sort_key(state.summary_sort_column));
        if state.sort_ascending { cmp } else { cmp.reverse() }
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

    // Calculate visible columns based on horizontal offset
    let h_offset = state.summary_horizontal_offset as usize;
    let clamped_offset = h_offset.min(METRIC_COLUMNS.len().saturating_sub(1));

    // Split area: frozen file column + scrollable metrics
    let chunks = Layout::horizontal([
        Constraint::Min(22),
        Constraint::Fill(1),
    ])
    .split(area);

    let file_area = chunks[0];
    let metric_area = chunks[1];

    // Determine which metric columns are visible
    let available_width = metric_area.width.saturating_sub(2); // borders
    let mut visible_cols: Vec<(usize, &str, u16)> = Vec::new();
    let mut used_width = 0u16;
    for (i, (name, width)) in METRIC_COLUMNS.iter().enumerate().skip(clamped_offset) {
        if used_width + width > available_width {
            break;
        }
        visible_cols.push((i, name, *width));
        used_width += width;
    }

    let visible_end = if visible_cols.is_empty() {
        clamped_offset
    } else {
        visible_cols.last().unwrap().0 + 1
    };

    // Sort indicator
    let indicator = if state.sort_ascending { "\u{25b2}" } else { "\u{25bc}" };
    let sort_col_idx = state.summary_sort_column.index();
    // Sort indicator for File column (index 0 in SummarySortColumn)
    let file_header = if sort_col_idx == 0 {
        format!("File {}", indicator)
    } else {
        "File".to_string()
    };

    // Build file column table (frozen)
    let file_rows: Vec<Row> = rows
        .iter()
        .map(|r| Row::new(vec![Cell::from(r.filename.as_str()).style(Style::default().fg(Color::White))]))
        .collect();

    let file_table = Table::new(
        file_rows,
        [Constraint::Fill(1)],
    )
    .header(Row::new(vec![
        Cell::from(file_header).style(table_style::header_style()),
    ]))
    .block(
        Block::default()
            .title(" Summary ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    )
    .row_highlight_style(table_style::highlight_style());

    frame.render_widget(file_table, file_area);

    // Build metric columns table (scrollable)
    let metric_widths: Vec<Constraint> = visible_cols
        .iter()
        .map(|(_, _, w)| Constraint::Length(*w))
        .collect();

    let metric_header: Vec<Cell> = visible_cols
        .iter()
        .map(|(i, name, _)| {
            // sort_col_idx is 0-based for SummarySortColumn but metric columns start at index 1
            // (File=0, Tool=1, Reads=2, ...). The metric column array is 0-indexed but maps to
            // SummarySortColumn index = metric_array_index + 1
            let is_sort_col = sort_col_idx == i + 1;
            let text = if is_sort_col {
                format!("{} {}", name, indicator)
            } else {
                name.to_string()
            };
            Cell::from(text).style(table_style::header_style())
        })
        .collect();

    let metric_rows: Vec<Row> = rows
        .iter()
        .map(|r| {
            let cells: Vec<Cell> = visible_cols
                .iter()
                .map(|(col_idx, _, _)| {
                    let value = metric_cell_value(r, *col_idx);
                    let color = metric_cell_color(r, *col_idx, &state.thresholds);
                    let mut style = Style::default().fg(color);
                    if *col_idx == 15 {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    Cell::from(value).style(style)
                })
                .collect();
            Row::new(cells)
        })
        .collect();

    let col_info = format!(
        " [{}-{}/{}] h/l:Scroll ",
        clamped_offset + 1,
        visible_end,
        METRIC_COLUMNS.len()
    );

    let metric_table = Table::new(metric_rows, metric_widths)
        .header(Row::new(metric_header))
        .block(
            Block::default()
                .title(col_info)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .row_highlight_style(table_style::highlight_style());

    frame.render_widget(metric_table, metric_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::types::{
        BcftoolsStats, BcftoolsSummary, FastqcBasicStats, FastqcReport, SamtoolsStats,
        SamtoolsSummary, TsTvStats,
    };
    use std::path::PathBuf;

    fn make_results() -> QcResults {
        QcResults {
            scan_path: PathBuf::from("."),
            samtools_reports: vec![SamtoolsStats {
                source_file: PathBuf::from("/data/sample1.stats"),
                summary: SamtoolsSummary {
                    raw_total_sequences: 1000,
                    reads_mapped: 950,
                    reads_duplicated: 50,
                    error_rate: 0.001,
                    average_quality: 30.0,
                    ..Default::default()
                },
                coverage_histogram: vec![],
                insert_size_histogram: vec![],
                read_length_histogram: vec![],
                gc_content_first: vec![],
                gc_content_last: vec![],
            }],
            bcftools_reports: vec![BcftoolsStats {
                source_file: PathBuf::from("/data/sample1.vcf.stats"),
                summary: BcftoolsSummary {
                    num_records: 500,
                    num_snps: 400,
                    num_indels: 100,
                    ..Default::default()
                },
                tstv: TsTvStats {
                    ts_tv_ratio: 2.1,
                    ..Default::default()
                },
                substitution_types: vec![],
                allele_freq: vec![],
                qual_dist: vec![],
                indel_dist: vec![],
                depth_dist: vec![],
            }],
            fastqc_reports: vec![FastqcReport {
                source_file: PathBuf::from("/data/sample1_fastqc.zip"),
                sample_name: "sample1".into(),
                basic_statistics: FastqcBasicStats {
                    total_sequences: 2000,
                    percent_gc: 45.0,
                    ..Default::default()
                },
                per_base_quality: vec![],
                per_sequence_quality: vec![],
                per_base_gc_content: vec![],
                per_sequence_gc_content: vec![],
                sequence_length_dist: vec![],
                overrepresented_sequences: vec![],
                module_statuses: vec![
                    ("Per base sequence quality".into(), ModuleStatus::Pass),
                    ("Overrepresented sequences".into(), ModuleStatus::Warn),
                ],
            }],
        }
    }

    #[test]
    fn test_build_summary_rows() {
        let results = make_results();
        let thresholds = ThresholdConfig::default();
        let rows = build_summary_rows(&results, &thresholds);

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].tool, "samtools");
        assert_eq!(rows[0].reads, Some(1000));
        assert!(rows[0].mapped_pct.unwrap() > 94.0);
        assert_eq!(rows[0].overall_qc, QcLevel::Pass);

        assert_eq!(rows[1].tool, "bcftools");
        assert_eq!(rows[1].variants, Some(500));
        assert_eq!(rows[1].ts_tv, Some(2.1));

        assert_eq!(rows[2].tool, "FastQC");
        assert_eq!(rows[2].total_seqs, Some(2000));
        assert_eq!(rows[2].pass_modules, Some(1));
        assert_eq!(rows[2].warn_modules, Some(1));
    }

    #[test]
    fn test_summary_row_none_fields() {
        let results = make_results();
        let thresholds = ThresholdConfig::default();
        let rows = build_summary_rows(&results, &thresholds);

        // samtools row should have None for bcftools fields
        assert!(rows[0].variants.is_none());
        assert!(rows[0].ts_tv.is_none());

        // bcftools row should have None for samtools fields
        assert!(rows[1].reads.is_none());
        assert!(rows[1].mapped_pct.is_none());
    }

    #[test]
    fn test_summary_row_threshold_coloring() {
        let results = QcResults {
            scan_path: PathBuf::from("."),
            samtools_reports: vec![SamtoolsStats {
                source_file: PathBuf::from("bad.stats"),
                summary: SamtoolsSummary {
                    raw_total_sequences: 1000,
                    reads_mapped: 700, // 70% → FAIL
                    reads_duplicated: 400, // 40% → FAIL
                    error_rate: 0.02, // FAIL
                    ..Default::default()
                },
                coverage_histogram: vec![],
                insert_size_histogram: vec![],
                read_length_histogram: vec![],
                gc_content_first: vec![],
                gc_content_last: vec![],
            }],
            bcftools_reports: vec![],
            fastqc_reports: vec![],
        };
        let thresholds = ThresholdConfig::default();
        let rows = build_summary_rows(&results, &thresholds);
        assert_eq!(rows[0].overall_qc, QcLevel::Fail);
    }
}
