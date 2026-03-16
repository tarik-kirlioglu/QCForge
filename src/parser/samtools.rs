use std::collections::BTreeMap;
use std::path::Path;

use crate::error::{QcForgeError, Result};
use crate::parser::types::{
    CoverageEntry, GcEntry, InsertSizeEntry, ReadLengthEntry, SamtoolsStats, SamtoolsSummary,
};

pub fn parse_samtools_stats(path: &Path, content: &str) -> Result<SamtoolsStats> {
    let mut raw = BTreeMap::new();
    let mut coverage = Vec::new();
    let mut insert_size = Vec::new();
    let mut read_length = Vec::new();
    let mut gc_first = Vec::new();
    let mut gc_last = Vec::new();

    for line in content.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields.is_empty() {
            continue;
        }

        match fields[0] {
            "SN" if fields.len() >= 3 => {
                let key = fields[1].trim().trim_end_matches(':').to_string();
                let value = fields[2].trim().to_string();
                raw.insert(key, value);
            }
            "COV" if fields.len() >= 4 => {
                coverage.push(CoverageEntry {
                    range: fields[1].trim().to_string(),
                    depth: parse_num(path, "COV.depth", fields[2])?,
                    count: parse_num(path, "COV.count", fields[3])?,
                });
            }
            "IS" if fields.len() >= 6 => {
                insert_size.push(InsertSizeEntry {
                    insert_size: parse_num(path, "IS.insert_size", fields[1])?,
                    pairs_total: parse_num(path, "IS.pairs_total", fields[2])?,
                    pairs_inward: parse_num(path, "IS.pairs_inward", fields[3])?,
                    pairs_outward: parse_num(path, "IS.pairs_outward", fields[4])?,
                    pairs_other: parse_num(path, "IS.pairs_other", fields[5])?,
                });
            }
            "RL" if fields.len() >= 3 => {
                read_length.push(ReadLengthEntry {
                    length: parse_num(path, "RL.length", fields[1])?,
                    count: parse_num(path, "RL.count", fields[2])?,
                });
            }
            "GCF" if fields.len() >= 3 => {
                gc_first.push(GcEntry {
                    gc_percent: parse_num(path, "GCF.gc_percent", fields[1])?,
                    count: parse_num(path, "GCF.count", fields[2])?,
                });
            }
            "GCL" if fields.len() >= 3 => {
                gc_last.push(GcEntry {
                    gc_percent: parse_num(path, "GCL.gc_percent", fields[1])?,
                    count: parse_num(path, "GCL.count", fields[2])?,
                });
            }
            _ => {} // skip unknown sections
        }
    }

    let summary = build_summary(raw);

    Ok(SamtoolsStats {
        source_file: path.to_path_buf(),
        summary,
        coverage_histogram: coverage,
        insert_size_histogram: insert_size,
        read_length_histogram: read_length,
        gc_content_first: gc_first,
        gc_content_last: gc_last,
    })
}

fn build_summary(raw: BTreeMap<String, String>) -> SamtoolsSummary {
    let get_u64 = |key: &str| -> u64 {
        raw.get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    };
    let get_f64 = |key: &str| -> f64 {
        raw.get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0)
    };

    SamtoolsSummary {
        raw_total_sequences: get_u64("raw total sequences"),
        filtered_sequences: get_u64("filtered sequences"),
        reads_mapped: get_u64("reads mapped"),
        reads_unmapped: get_u64("reads unmapped"),
        reads_duplicated: get_u64("reads duplicated"),
        reads_mq0: get_u64("reads MQ0"),
        reads_qc_failed: get_u64("reads QC failed"),
        reads_properly_paired: get_u64("reads properly paired"),
        total_length: get_u64("total length"),
        bases_mapped: get_u64("bases mapped"),
        bases_mapped_cigar: get_u64("bases mapped (cigar)"),
        error_rate: get_f64("error rate"),
        average_length: get_f64("average length"),
        average_quality: get_f64("average quality"),
        insert_size_average: get_f64("insert size average"),
        insert_size_std_deviation: get_f64("insert size standard deviation"),
        pairs_on_different_chromosomes: get_u64("pairs on different chromosomes"),
        raw,
    }
}

fn parse_num<T: std::str::FromStr>(path: &Path, field: &str, value: &str) -> Result<T> {
    value.trim().parse::<T>().map_err(|_| QcForgeError::NumericParse {
        field: format!("{}:{}", path.display(), field),
        value: value.trim().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const SAMPLE_STATS: &str = "\
# This file was produced by samtools stats
# The command line was: samtools stats input.bam
SN\traw total sequences:\t1000000\t# excludes supplementary and secondary reads
SN\tfiltered sequences:\t0
SN\treads mapped:\t950000
SN\treads mapped and paired:\t940000
SN\treads unmapped:\t50000
SN\treads duplicated:\t120000
SN\treads MQ0:\t5000
SN\treads QC failed:\t100
SN\treads properly paired:\t930000
SN\ttotal length:\t150000000
SN\tbases mapped:\t142500000
SN\tbases mapped (cigar):\t140000000
SN\terror rate:\t0.002
SN\taverage length:\t150
SN\taverage quality:\t35.2
SN\tinsert size average:\t250.5
SN\tinsert size standard deviation:\t50.3
SN\tpairs on different chromosomes:\t1200
COV\t[1-1]\t1\t50000
COV\t[2-2]\t2\t80000
COV\t[3-4]\t3\t120000
IS\t200\t5000\t4800\t100\t100
IS\t250\t8000\t7700\t150\t150
RL\t150\t950000
RL\t100\t50000
GCF\t20.00\t15000
GCF\t50.00\t85000
GCL\t20.00\t14000
GCL\t50.00\t86000";

    #[test]
    fn test_parse_summary_numbers() {
        let path = PathBuf::from("test.stats");
        let result = parse_samtools_stats(&path, SAMPLE_STATS).unwrap();

        assert_eq!(result.summary.raw_total_sequences, 1_000_000);
        assert_eq!(result.summary.reads_mapped, 950_000);
        assert_eq!(result.summary.reads_unmapped, 50_000);
        assert_eq!(result.summary.reads_duplicated, 120_000);
        assert_eq!(result.summary.error_rate, 0.002);
        assert_eq!(result.summary.average_length, 150.0);
        assert_eq!(result.summary.average_quality, 35.2);
    }

    #[test]
    fn test_mapping_percent() {
        let path = PathBuf::from("test.stats");
        let result = parse_samtools_stats(&path, SAMPLE_STATS).unwrap();

        let pct = result.summary.mapping_percent();
        assert!((pct - 95.0).abs() < 0.01);
    }

    #[test]
    fn test_coverage_histogram() {
        let path = PathBuf::from("test.stats");
        let result = parse_samtools_stats(&path, SAMPLE_STATS).unwrap();

        assert_eq!(result.coverage_histogram.len(), 3);
        assert_eq!(result.coverage_histogram[0].range, "[1-1]");
        assert_eq!(result.coverage_histogram[0].count, 50000);
    }

    #[test]
    fn test_insert_size() {
        let path = PathBuf::from("test.stats");
        let result = parse_samtools_stats(&path, SAMPLE_STATS).unwrap();

        assert_eq!(result.insert_size_histogram.len(), 2);
        assert_eq!(result.insert_size_histogram[0].insert_size, 200);
        assert_eq!(result.insert_size_histogram[1].pairs_total, 8000);
    }

    #[test]
    fn test_gc_content() {
        let path = PathBuf::from("test.stats");
        let result = parse_samtools_stats(&path, SAMPLE_STATS).unwrap();

        assert_eq!(result.gc_content_first.len(), 2);
        assert_eq!(result.gc_content_last.len(), 2);
    }
}
