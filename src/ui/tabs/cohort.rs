use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::parser::types::QcResults;
use crate::threshold::{QcLevel, ThresholdConfig};
use crate::ui::widgets::table as table_style;

const MIN_SAMPLES: usize = 5;
const AXIS_LABEL_WIDTH: u16 = 14;
const AXIS_VALUE_LABEL_WIDTH: u16 = 8;
const AXIS_OUTLIER_TAIL_WIDTH: u16 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CohortMetric {
    MappingRate,
    DuplicationRate,
    ErrorRate,
    TsTvRatio,
    GcDeviation,
}

impl CohortMetric {
    pub fn label(&self) -> &'static str {
        match self {
            CohortMetric::MappingRate => "Mapping %",
            CohortMetric::DuplicationRate => "Dup %",
            CohortMetric::ErrorRate => "Error rate",
            CohortMetric::TsTvRatio => "Ts/Tv",
            CohortMetric::GcDeviation => "GC dev",
        }
    }

    pub fn format_value(&self, v: f64) -> String {
        match self {
            CohortMetric::MappingRate | CohortMetric::DuplicationRate => format!("{:.1}", v),
            CohortMetric::ErrorRate => format!("{:.4}", v),
            CohortMetric::TsTvRatio => format!("{:.2}", v),
            CohortMetric::GcDeviation => format!("{:.1}", v),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CohortDataPoint {
    pub filename: String,
    pub value: f64,
    pub threshold_fail: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct BoxStats {
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
    pub lower_fence: f64,
    pub upper_fence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutlierDirection {
    Below,
    Above,
}

#[derive(Debug, Clone)]
pub struct Outlier {
    pub filename: String,
    pub metric: CohortMetric,
    pub value: f64,
    pub deviation_magnitude: f64,
    pub direction: OutlierDirection,
    pub threshold_fail: bool,
}

/// Linear-interpolation quartile (numpy default / "Type 7").
/// `sorted_values` must be sorted ascending and non-empty.
fn quantile(sorted_values: &[f64], p: f64) -> f64 {
    let n = sorted_values.len();
    if n == 1 {
        return sorted_values[0];
    }
    let idx = p * (n - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    if lo == hi {
        sorted_values[lo]
    } else {
        let frac = idx - lo as f64;
        sorted_values[lo] + frac * (sorted_values[hi] - sorted_values[lo])
    }
}

pub fn compute_box_stats(values: &[f64]) -> Option<BoxStats> {
    if values.len() < MIN_SAMPLES {
        return None;
    }
    let mut sorted: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if sorted.len() < MIN_SAMPLES {
        return None;
    }
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let q1 = quantile(&sorted, 0.25);
    let median = quantile(&sorted, 0.5);
    let q3 = quantile(&sorted, 0.75);
    let iqr = q3 - q1;
    let lower_fence = q1 - 1.5 * iqr;
    let upper_fence = q3 + 1.5 * iqr;
    let min = *sorted.first().unwrap();
    let max = *sorted.last().unwrap();

    Some(BoxStats {
        min,
        q1,
        median,
        q3,
        max,
        lower_fence,
        upper_fence,
    })
}

pub fn detect_outliers(
    points: &[CohortDataPoint],
    stats: &BoxStats,
    metric: CohortMetric,
) -> Vec<Outlier> {
    let mut out = Vec::new();
    for p in points {
        if p.value < stats.lower_fence {
            out.push(Outlier {
                filename: p.filename.clone(),
                metric,
                value: p.value,
                deviation_magnitude: stats.lower_fence - p.value,
                direction: OutlierDirection::Below,
                threshold_fail: p.threshold_fail,
            });
        } else if p.value > stats.upper_fence {
            out.push(Outlier {
                filename: p.filename.clone(),
                metric,
                value: p.value,
                deviation_magnitude: p.value - stats.upper_fence,
                direction: OutlierDirection::Above,
                threshold_fail: p.threshold_fail,
            });
        }
    }
    out
}

pub fn build_cohort_data(
    results: &QcResults,
    thresholds: &ThresholdConfig,
) -> Vec<(CohortMetric, Vec<CohortDataPoint>)> {
    let mut mapping = Vec::new();
    let mut duplication = Vec::new();
    let mut error = Vec::new();
    let mut tstv = Vec::new();
    let mut gc = Vec::new();

    for r in &results.samtools_reports {
        let fname = file_label(&r.source_file);
        let m = r.summary.mapping_percent();
        let d = r.summary.duplication_percent();
        let e = r.summary.error_rate;
        mapping.push(CohortDataPoint {
            filename: fname.clone(),
            value: m,
            threshold_fail: thresholds.mapping_rate.evaluate(m) == QcLevel::Fail,
        });
        duplication.push(CohortDataPoint {
            filename: fname.clone(),
            value: d,
            threshold_fail: thresholds.duplication_rate.evaluate(d) == QcLevel::Fail,
        });
        error.push(CohortDataPoint {
            filename: fname,
            value: e,
            threshold_fail: thresholds.error_rate.evaluate(e) == QcLevel::Fail,
        });
    }

    for r in &results.bcftools_reports {
        let fname = file_label(&r.source_file);
        let v = r.tstv.ts_tv_ratio;
        tstv.push(CohortDataPoint {
            filename: fname,
            value: v,
            threshold_fail: thresholds.ts_tv_ratio.evaluate(v) == QcLevel::Fail,
        });
    }

    for r in &results.fastqc_reports {
        let fname = if r.sample_name.is_empty() {
            file_label(&r.source_file)
        } else {
            r.sample_name.clone()
        };
        let dev = (r.basic_statistics.percent_gc - 50.0).abs();
        gc.push(CohortDataPoint {
            filename: fname,
            value: dev,
            threshold_fail: thresholds.gc_deviation.evaluate(dev) == QcLevel::Fail,
        });
    }

    vec![
        (CohortMetric::MappingRate, mapping),
        (CohortMetric::DuplicationRate, duplication),
        (CohortMetric::ErrorRate, error),
        (CohortMetric::TsTvRatio, tstv),
        (CohortMetric::GcDeviation, gc),
    ]
}

fn file_label(path: &std::path::Path) -> String {
    path.file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn collect_outliers(
    cohort: &[(CohortMetric, Vec<CohortDataPoint>)],
) -> (Vec<Outlier>, Vec<(CohortMetric, Option<BoxStats>)>) {
    let mut all = Vec::new();
    let mut stats_per_metric = Vec::with_capacity(cohort.len());
    for (metric, points) in cohort {
        let values: Vec<f64> = points.iter().map(|p| p.value).collect();
        let stats = compute_box_stats(&values);
        if let Some(s) = stats {
            all.extend(detect_outliers(points, &s, *metric));
        }
        stats_per_metric.push((*metric, stats));
    }
    all.sort_by(|a, b| {
        b.deviation_magnitude
            .partial_cmp(&a.deviation_magnitude)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    (all, stats_per_metric)
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let results = match &state.qc_results {
        Some(r) => r,
        None => return,
    };

    let cohort = build_cohort_data(results, &state.thresholds);
    let total_points: usize = cohort.iter().map(|(_, v)| v.len()).sum();
    let any_metric_qualifies = cohort.iter().any(|(_, v)| v.len() >= MIN_SAMPLES);

    if !any_metric_qualifies {
        let msg = format!(
            "Cohort analysis requires \u{2265}{} samples per metric (current totals across metrics: {}).",
            MIN_SAMPLES, total_points
        );
        let p = Paragraph::new(Line::from(Span::styled(
            msg,
            Style::default().fg(Color::Yellow),
        )))
        .block(
            Block::default()
                .title(" Cohort ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_widget(p, area);
        return;
    }

    let (outliers, stats_per_metric) = collect_outliers(&cohort);

    // Top: 5 metric rows with one blank spacer between each (5 rows + 4 spacers = 9 inner)
    // + 2 borders = 11 total height for the block.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(11), Constraint::Min(0)])
        .split(area);

    render_boxplots(frame, chunks[0], &cohort, &stats_per_metric);
    render_outlier_table(frame, chunks[1], &outliers, state.cohort_selected);
}

fn render_boxplots(
    frame: &mut Frame,
    area: Rect,
    cohort: &[(CohortMetric, Vec<CohortDataPoint>)],
    stats_per_metric: &[(CohortMetric, Option<BoxStats>)],
) {
    let block = Block::default()
        .title(" Cohort distribution (IQR boxplot) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    let inner_w = inner.width;
    if inner_w < AXIS_LABEL_WIDTH + 2 * AXIS_VALUE_LABEL_WIDTH + 12 {
        lines.push(Line::from(Span::styled(
            "Terminal too narrow (need \u{2265}80 cols).",
            Style::default().fg(Color::Yellow),
        )));
        let p = Paragraph::new(lines);
        frame.render_widget(p, inner);
        return;
    }

    let axis_width = inner_w
        .saturating_sub(AXIS_LABEL_WIDTH)
        .saturating_sub(2 * AXIS_VALUE_LABEL_WIDTH)
        .saturating_sub(AXIS_OUTLIER_TAIL_WIDTH);
    let axis_width = axis_width.max(15);

    for (i, (metric, points)) in cohort.iter().enumerate() {
        if i > 0 {
            lines.push(Line::from(""));
        }

        let stats = stats_per_metric
            .iter()
            .find(|(m, _)| m == metric)
            .and_then(|(_, s)| s.as_ref());

        let label_span = Span::styled(
            format!(
                "{:width$}",
                metric.label(),
                width = AXIS_LABEL_WIDTH as usize
            ),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

        match stats {
            Some(s) => {
                let line = build_boxplot_line(*metric, points, s, axis_width, label_span);
                lines.push(line);
            }
            None => {
                let n = points.len();
                let warn = format!("n={} \u{2014} too small (need \u{2265}{})", n, MIN_SAMPLES);
                let line = Line::from(vec![
                    label_span,
                    Span::styled(warn, Style::default().fg(Color::DarkGray)),
                ]);
                lines.push(line);
            }
        }
    }

    let p = Paragraph::new(lines);
    frame.render_widget(p, inner);
}

fn build_boxplot_line(
    metric: CohortMetric,
    points: &[CohortDataPoint],
    stats: &BoxStats,
    axis_width: u16,
    label_span: Span<'static>,
) -> Line<'static> {
    let axis_min = stats.min.min(stats.lower_fence);
    let axis_max = stats.max.max(stats.upper_fence);
    let span = (axis_max - axis_min).max(f64::EPSILON);

    let to_col = |v: f64| -> usize {
        let frac = ((v - axis_min) / span).clamp(0.0, 1.0);
        (frac * (axis_width as f64 - 1.0)).round() as usize
    };

    let q1_col = to_col(stats.q1);
    let q3_col = to_col(stats.q3);
    let med_col = to_col(stats.median);
    let lf_col = to_col(stats.lower_fence);
    let uf_col = to_col(stats.upper_fence);

    // Identify outlier columns (deduplicated).
    let mut outlier_cols: Vec<(usize, bool)> = Vec::new(); // (col, threshold_fail)
    let mut outlier_names: Vec<(String, bool, f64)> = Vec::new();
    for p in points {
        if p.value < stats.lower_fence || p.value > stats.upper_fence {
            let col = to_col(p.value);
            // collapse to highest-severity for this column
            if let Some(existing) = outlier_cols.iter_mut().find(|(c, _)| *c == col) {
                existing.1 = existing.1 || p.threshold_fail;
            } else {
                outlier_cols.push((col, p.threshold_fail));
            }
            outlier_names.push((p.filename.clone(), p.threshold_fail, p.value));
        }
    }
    outlier_names.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    // Build axis cells.
    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(label_span);
    spans.push(Span::styled(
        format!(
            "{:>width$} ",
            metric.format_value(axis_min),
            width = (AXIS_VALUE_LABEL_WIDTH as usize).saturating_sub(1)
        ),
        Style::default().fg(Color::DarkGray),
    ));

    for col in 0..(axis_width as usize) {
        let outlier_here = outlier_cols.iter().find(|(c, _)| *c == col).copied();
        let in_box = col >= q1_col && col <= q3_col;
        let on_whisker = (col >= lf_col && col < q1_col) || (col > q3_col && col <= uf_col);

        let (ch, color, bold) = if let Some((_, fail)) = outlier_here {
            if fail {
                ("\u{25cf}", Color::Red, true) // ●
            } else {
                ("o", Color::Yellow, true)
            }
        } else if col == med_col && in_box {
            ("\u{2502}", Color::White, true) // │
        } else if col == q1_col {
            ("[", Color::Cyan, true)
        } else if col == q3_col {
            ("]", Color::Cyan, true)
        } else if in_box {
            ("\u{2588}", Color::Cyan, false) // █
        } else if col == lf_col {
            ("\u{251c}", Color::DarkGray, false) // ├
        } else if col == uf_col {
            ("\u{2524}", Color::DarkGray, false) // ┤
        } else if on_whisker {
            ("\u{2500}", Color::DarkGray, false) // ─
        } else {
            (" ", Color::DarkGray, false)
        };

        let mut style = Style::default().fg(color);
        if bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        spans.push(Span::styled(ch.to_string(), style));
    }

    spans.push(Span::styled(
        format!(
            " {:<width$}",
            metric.format_value(axis_max),
            width = (AXIS_VALUE_LABEL_WIDTH as usize).saturating_sub(1)
        ),
        Style::default().fg(Color::DarkGray),
    ));

    if !outlier_names.is_empty() {
        let summary = if outlier_names.len() == 1 {
            format!(
                "  {} ({})",
                truncate(&outlier_names[0].0, 14),
                metric.format_value(outlier_names[0].2)
            )
        } else {
            format!(
                "  {} +{} more",
                truncate(&outlier_names[0].0, 12),
                outlier_names.len() - 1
            )
        };
        let any_fail = outlier_names.iter().any(|(_, f, _)| *f);
        spans.push(Span::styled(
            summary,
            Style::default().fg(if any_fail { Color::Red } else { Color::Yellow }),
        ));
    }

    Line::from(spans)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(max.saturating_sub(1)).collect();
        t.push('\u{2026}');
        t
    }
}

fn render_outlier_table(frame: &mut Frame, area: Rect, outliers: &[Outlier], selected: usize) {
    let title = format!(" Outliers ({}) ", outliers.len());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    if outliers.is_empty() {
        let p = Paragraph::new(Line::from(Span::styled(
            "No cohort-relative outliers detected.",
            Style::default().fg(Color::Green),
        )))
        .block(block);
        frame.render_widget(p, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Sample").style(table_style::header_style()),
        Cell::from("Metric").style(table_style::header_style()),
        Cell::from("Value").style(table_style::header_style()),
        Cell::from("Fence").style(table_style::header_style()),
        Cell::from("\u{0394}").style(table_style::header_style()),
        Cell::from("Side").style(table_style::header_style()),
        Cell::from("Threshold").style(table_style::header_style()),
    ]);

    let max = outliers.len().saturating_sub(1);
    let clamped_selected = selected.min(max);

    let rows: Vec<Row> = outliers
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let is_sel = i == clamped_selected;
            let base = if is_sel {
                table_style::highlight_style()
            } else {
                Style::default().fg(Color::White)
            };

            let threshold_color = if o.threshold_fail {
                Color::Red
            } else {
                Color::Green
            };
            let threshold_text = if o.threshold_fail { "FAIL" } else { "PASS" };

            let side = match o.direction {
                OutlierDirection::Below => "Below",
                OutlierDirection::Above => "Above",
            };

            Row::new(vec![
                Cell::from(o.filename.clone()).style(base),
                Cell::from(o.metric.label()).style(base),
                Cell::from(o.metric.format_value(o.value)).style(base),
                Cell::from(o.metric.format_value(fence_value(o))).style(base),
                Cell::from(o.metric.format_value(o.deviation_magnitude)).style(base),
                Cell::from(side).style(base),
                Cell::from(threshold_text)
                    .style(base.fg(threshold_color).add_modifier(Modifier::BOLD)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(28),
        Constraint::Length(12),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(7),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths).header(header).block(block);
    frame.render_widget(table, area);
}

fn fence_value(o: &Outlier) -> f64 {
    match o.direction {
        OutlierDirection::Below => o.value + o.deviation_magnitude,
        OutlierDirection::Above => o.value - o.deviation_magnitude,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::types::{
        BcftoolsStats, BcftoolsSummary, FastqcBasicStats, FastqcReport, ModuleStatus,
        SamtoolsStats, SamtoolsSummary, TsTvStats,
    };
    use std::path::PathBuf;

    #[test]
    fn test_compute_box_stats_basic() {
        // values 1..=9 → Q1=3, median=5, Q3=7, IQR=4, fences 3-6=-3, 7+6=13
        let values: Vec<f64> = (1..=9).map(|v| v as f64).collect();
        let s = compute_box_stats(&values).unwrap();
        assert!((s.q1 - 3.0).abs() < 1e-9);
        assert!((s.median - 5.0).abs() < 1e-9);
        assert!((s.q3 - 7.0).abs() < 1e-9);
        assert!((s.lower_fence + 3.0).abs() < 1e-9);
        assert!((s.upper_fence - 13.0).abs() < 1e-9);
    }

    #[test]
    fn test_compute_box_stats_too_small() {
        let values = vec![1.0, 2.0, 3.0, 4.0]; // n=4 < MIN_SAMPLES
        assert!(compute_box_stats(&values).is_none());
    }

    #[test]
    fn test_detect_outliers_iqr() {
        let points: Vec<CohortDataPoint> = (1..=9)
            .map(|v| CohortDataPoint {
                filename: format!("s{}", v),
                value: v as f64,
                threshold_fail: false,
            })
            .chain(std::iter::once(CohortDataPoint {
                filename: "extreme".into(),
                value: 50.0,
                threshold_fail: true,
            }))
            .collect();
        let values: Vec<f64> = points.iter().map(|p| p.value).collect();
        let stats = compute_box_stats(&values).unwrap();
        let outliers = detect_outliers(&points, &stats, CohortMetric::DuplicationRate);
        assert_eq!(outliers.len(), 1);
        assert_eq!(outliers[0].filename, "extreme");
        assert_eq!(outliers[0].direction, OutlierDirection::Above);
        assert!(outliers[0].threshold_fail);
    }

    fn make_results(samtools_count: usize) -> QcResults {
        let samtools: Vec<SamtoolsStats> = (0..samtools_count)
            .map(|i| SamtoolsStats {
                source_file: PathBuf::from(format!("/data/sample{}.stats", i)),
                summary: SamtoolsSummary {
                    raw_total_sequences: 1000,
                    reads_mapped: if i == 0 { 600 } else { 950 },
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
            })
            .collect();
        QcResults {
            scan_path: PathBuf::from("."),
            samtools_reports: samtools,
            bcftools_reports: vec![BcftoolsStats {
                source_file: PathBuf::from("/data/v.vcf.stats"),
                summary: BcftoolsSummary {
                    num_records: 100,
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
                source_file: PathBuf::from("/data/s_fastqc.zip"),
                sample_name: "s".into(),
                basic_statistics: FastqcBasicStats {
                    total_sequences: 100,
                    percent_gc: 45.0,
                    ..Default::default()
                },
                per_base_quality: vec![],
                per_sequence_quality: vec![],
                per_base_gc_content: vec![],
                per_sequence_gc_content: vec![],
                sequence_length_dist: vec![],
                overrepresented_sequences: vec![],
                module_statuses: vec![("Per base sequence quality".into(), ModuleStatus::Pass)],
            }],
        }
    }

    #[test]
    fn test_build_cohort_data_metrics() {
        let results = make_results(6);
        let thresholds = ThresholdConfig::default();
        let data = build_cohort_data(&results, &thresholds);
        assert_eq!(data.len(), 5);

        let mapping = &data
            .iter()
            .find(|(m, _)| *m == CohortMetric::MappingRate)
            .unwrap()
            .1;
        assert_eq!(mapping.len(), 6);
        // sample0 has 60% mapping, others 95%
        let s0 = mapping
            .iter()
            .find(|p| p.filename == "sample0.stats")
            .unwrap();
        assert!((s0.value - 60.0).abs() < 1e-6);
        assert!(s0.threshold_fail); // 60% < 80% fail threshold

        let tstv = &data
            .iter()
            .find(|(m, _)| *m == CohortMetric::TsTvRatio)
            .unwrap()
            .1;
        assert_eq!(tstv.len(), 1);

        let gc = &data
            .iter()
            .find(|(m, _)| *m == CohortMetric::GcDeviation)
            .unwrap()
            .1;
        assert_eq!(gc.len(), 1);
        // |45 - 50| = 5
        assert!((gc[0].value - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_outlier_with_threshold_fail() {
        let results = make_results(6);
        let thresholds = ThresholdConfig::default();
        let data = build_cohort_data(&results, &thresholds);

        let mapping = &data
            .iter()
            .find(|(m, _)| *m == CohortMetric::MappingRate)
            .unwrap()
            .1;
        let values: Vec<f64> = mapping.iter().map(|p| p.value).collect();
        let stats = compute_box_stats(&values).unwrap();
        let outliers = detect_outliers(mapping, &stats, CohortMetric::MappingRate);

        // sample0 (60%) is far below the 95%-cluster fence
        assert_eq!(outliers.len(), 1);
        assert_eq!(outliers[0].direction, OutlierDirection::Below);
        assert!(outliers[0].threshold_fail);
    }
}
