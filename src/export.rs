use std::path::Path;

use serde::Serialize;

use crate::error::{QcForgeError, Result};
use crate::parser::types::{ModuleStatus, QcResults};
use crate::threshold::ThresholdConfig;

#[derive(Serialize)]
struct QcRow {
    filename: String,
    tool: String,
    reads: Option<u64>,
    mapped_percent: Option<f64>,
    duplication_percent: Option<f64>,
    error_rate: Option<f64>,
    variants: Option<u64>,
    snps: Option<u64>,
    indels: Option<u64>,
    ts_tv_ratio: Option<f64>,
    total_sequences: Option<u64>,
    gc_percent: Option<f64>,
    pass_modules: Option<u32>,
    warn_modules: Option<u32>,
    fail_modules: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    qc_status: Option<String>,
}

pub fn write_csv(path: &Path, results: &QcResults, thresholds: Option<&ThresholdConfig>) -> Result<()> {
    let is_tsv = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("tsv"));

    let mut writer = csv::WriterBuilder::new()
        .delimiter(if is_tsv { b'\t' } else { b',' })
        .from_path(path)?;

    for report in &results.samtools_reports {
        let filename = report
            .source_file
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        writer.serialize(QcRow {
            filename,
            tool: "samtools".into(),
            reads: Some(report.summary.raw_total_sequences),
            mapped_percent: Some(report.summary.mapping_percent()),
            duplication_percent: Some(report.summary.duplication_percent()),
            error_rate: Some(report.summary.error_rate),
            variants: None,
            snps: None,
            indels: None,
            ts_tv_ratio: None,
            total_sequences: None,
            gc_percent: None,
            pass_modules: None,
            warn_modules: None,
            fail_modules: None,
            qc_status: thresholds.map(|t| {
                t.evaluate_sample(
                    Some(report.summary.mapping_percent()),
                    Some(report.summary.duplication_percent()),
                    Some(report.summary.error_rate),
                    None,
                    None,
                )
                .label()
                .to_string()
            }),
        })?;
    }

    for report in &results.bcftools_reports {
        let filename = report
            .source_file
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        writer.serialize(QcRow {
            filename,
            tool: "bcftools".into(),
            reads: None,
            mapped_percent: None,
            duplication_percent: None,
            error_rate: None,
            variants: Some(report.summary.num_records),
            snps: Some(report.summary.num_snps),
            indels: Some(report.summary.num_indels),
            ts_tv_ratio: Some(report.tstv.ts_tv_ratio),
            total_sequences: None,
            gc_percent: None,
            pass_modules: None,
            warn_modules: None,
            fail_modules: None,
            qc_status: thresholds.map(|t| {
                t.evaluate_sample(None, None, None, Some(report.tstv.ts_tv_ratio), None)
                    .label()
                    .to_string()
            }),
        })?;
    }

    for report in &results.fastqc_reports {
        let filename = if report.sample_name.is_empty() {
            report
                .source_file
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        } else {
            report.sample_name.clone()
        };
        let (pass_count, warn_count, fail_count) =
            report
                .module_statuses
                .iter()
                .fold((0u32, 0u32, 0u32), |(p, w, f), (_, status)| match status {
                    ModuleStatus::Pass => (p + 1, w, f),
                    ModuleStatus::Warn => (p, w + 1, f),
                    ModuleStatus::Fail => (p, w, f + 1),
                });
        writer.serialize(QcRow {
            filename,
            tool: "fastqc".into(),
            reads: None,
            mapped_percent: None,
            duplication_percent: None,
            error_rate: None,
            variants: None,
            snps: None,
            indels: None,
            ts_tv_ratio: None,
            total_sequences: Some(report.basic_statistics.total_sequences),
            gc_percent: Some(report.basic_statistics.percent_gc),
            pass_modules: Some(pass_count),
            warn_modules: Some(warn_count),
            fail_modules: Some(fail_count),
            qc_status: thresholds.map(|t| {
                t.evaluate_sample(
                    None,
                    None,
                    None,
                    None,
                    Some(report.basic_statistics.percent_gc),
                )
                .label()
                .to_string()
            }),
        })?;
    }

    writer.flush().map_err(QcForgeError::Io)?;
    Ok(())
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
    fn test_csv_output() {
        let results = make_results();
        let mut buf = Vec::new();
        {
            let mut writer = csv::Writer::from_writer(&mut buf);
            // Manually replicate write_csv logic for in-memory test
            for report in &results.samtools_reports {
                writer
                    .serialize(QcRow {
                        filename: "sample1.stats".into(),
                        tool: "samtools".into(),
                        reads: Some(report.summary.raw_total_sequences),
                        mapped_percent: Some(report.summary.mapping_percent()),
                        duplication_percent: Some(report.summary.duplication_percent()),
                        error_rate: Some(report.summary.error_rate),
                        variants: None,
                        snps: None,
                        indels: None,
                        ts_tv_ratio: None,
                        total_sequences: None,
                        gc_percent: None,
                        pass_modules: None,
                        warn_modules: None,
                        fail_modules: None,
                        qc_status: None,
                    })
                    .unwrap();
            }
            writer.flush().unwrap();
        }
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("filename,tool,"));
        assert!(output.contains("sample1.stats,samtools,"));
        assert!(output.contains("1000,"));
    }

    #[test]
    fn test_csv_empty_results() {
        let results = QcResults::default();
        let mut buf = Vec::new();
        {
            let mut writer = csv::Writer::from_writer(&mut buf);
            // Empty — just header
            writer
                .serialize(QcRow {
                    filename: String::new(),
                    tool: String::new(),
                    reads: None,
                    mapped_percent: None,
                    duplication_percent: None,
                    error_rate: None,
                    variants: None,
                    snps: None,
                    indels: None,
                    ts_tv_ratio: None,
                    total_sequences: None,
                    gc_percent: None,
                    pass_modules: None,
                    warn_modules: None,
                    fail_modules: None,
                    qc_status: None,
                })
                .ok();
            // Just verify no panic with empty data
            let _ = results;
        }
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("filename,tool,"));
    }

    #[test]
    fn test_write_csv_to_file() {
        let results = make_results();
        let dir = std::env::temp_dir();
        let csv_path = dir.join("qcforge_test.csv");
        write_csv(&csv_path, &results, None).unwrap();

        let content = std::fs::read_to_string(&csv_path).unwrap();
        assert!(content.contains("filename,tool,"));
        assert!(content.contains("sample1.stats,samtools,"));
        assert!(content.contains("sample1.vcf.stats,bcftools,"));
        assert!(content.contains("sample1,fastqc,"));

        // Cleanup
        let _ = std::fs::remove_file(&csv_path);
    }

    #[test]
    fn test_write_tsv_to_file() {
        let results = make_results();
        let dir = std::env::temp_dir();
        let tsv_path = dir.join("qcforge_test.tsv");
        write_csv(&tsv_path, &results, None).unwrap();

        let content = std::fs::read_to_string(&tsv_path).unwrap();
        assert!(content.contains("filename\ttool\t"));
        assert!(content.contains("sample1.stats\tsamtools\t"));

        // Cleanup
        let _ = std::fs::remove_file(&tsv_path);
    }
}
