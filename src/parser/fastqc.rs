use std::io::Read;
use std::path::Path;

use crate::error::{QcForgeError, Result};
use crate::parser::types::{
    FastqcBasicStats, FastqcReport, ModuleStatus, OverrepresentedSeq, PerBaseGc, PerBaseQuality,
    PerSeqGc, PerSequenceQuality, SeqLengthEntry,
};

pub fn parse_fastqc_zip(path: &Path) -> Result<FastqcReport> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Find fastqc_data.txt inside the archive
    let data_content = find_and_read_data_txt(&mut archive, path)?;

    parse_fastqc_data(path, &data_content)
}

fn find_and_read_data_txt(
    archive: &mut zip::ZipArchive<std::fs::File>,
    path: &Path,
) -> Result<String> {
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.name().ends_with("fastqc_data.txt") {
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            return Ok(content);
        }
    }

    Err(QcForgeError::FastqcDataNotFound(
        path.display().to_string(),
    ))
}

fn parse_fastqc_data(path: &Path, content: &str) -> Result<FastqcReport> {
    let mut module_statuses = Vec::new();
    let mut basic_stats = FastqcBasicStats::default();
    let mut per_base_quality = Vec::new();
    let mut per_seq_quality = Vec::new();
    let mut per_base_gc = Vec::new();
    let mut per_seq_gc = Vec::new();
    let mut seq_length_dist = Vec::new();
    let mut overrepresented = Vec::new();

    let mut current_section: Option<String> = None;

    for line in content.lines() {
        // Section start (skip ##FastQC header line)
        if line.starts_with(">>") && !line.starts_with(">>>#") && line != ">>END_MODULE" && !line.starts_with("##") {
            let parts: Vec<&str> = line[2..].splitn(2, '\t').collect();
            let section_name = parts[0].to_string();
            let status = parts.get(1).unwrap_or(&"pass");
            module_statuses.push((section_name.clone(), ModuleStatus::from_str(status)));
            current_section = Some(section_name);
            continue;
        }

        // Section end
        if line == ">>END_MODULE" {
            current_section = None;
            continue;
        }

        // Skip comment/header lines
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();

        match current_section.as_deref() {
            Some("Basic Statistics") if fields.len() >= 2 => {
                parse_basic_stat(&mut basic_stats, fields[0], fields[1]);
            }
            Some("Per base sequence quality") if fields.len() >= 7 => {
                if let Ok(entry) = parse_per_base_quality(&fields, path) {
                    per_base_quality.push(entry);
                }
            }
            Some("Per sequence quality scores") if fields.len() >= 2 => {
                if let (Ok(q), Ok(c)) = (
                    fields[0].trim().parse::<u32>(),
                    fields[1].trim().parse::<f64>(),
                ) {
                    per_seq_quality.push(PerSequenceQuality { quality: q, count: c });
                }
            }
            Some("Per base sequence content") if fields.len() >= 5 => {
                if let (Ok(g), Ok(a), Ok(t), Ok(c)) = (
                    fields[1].trim().parse::<f64>(),
                    fields[2].trim().parse::<f64>(),
                    fields[3].trim().parse::<f64>(),
                    fields[4].trim().parse::<f64>(),
                ) {
                    per_base_gc.push(PerBaseGc {
                        base: fields[0].trim().to_string(),
                        g,
                        a,
                        t,
                        c,
                    });
                }
            }
            Some("Per sequence GC content") if fields.len() >= 2 => {
                if let (Ok(gc), Ok(count)) = (
                    fields[0].trim().parse::<f64>(),
                    fields[1].trim().parse::<f64>(),
                ) {
                    per_seq_gc.push(PerSeqGc {
                        gc_content: gc,
                        count,
                    });
                }
            }
            Some("Sequence Length Distribution") if fields.len() >= 2 => {
                if let Ok(count) = fields[1].trim().parse::<f64>() {
                    seq_length_dist.push(SeqLengthEntry {
                        length: fields[0].trim().to_string(),
                        count,
                    });
                }
            }
            Some("Overrepresented sequences") if fields.len() >= 4 => {
                if let (Ok(count), Ok(pct)) = (
                    fields[1].trim().parse::<u64>(),
                    fields[2].trim().parse::<f64>(),
                ) {
                    overrepresented.push(OverrepresentedSeq {
                        sequence: fields[0].trim().to_string(),
                        count,
                        percentage: pct,
                        possible_source: fields[3].trim().to_string(),
                    });
                }
            }
            _ => {}
        }
    }

    let sample_name = basic_stats.filename.clone();

    Ok(FastqcReport {
        source_file: path.to_path_buf(),
        sample_name,
        basic_statistics: basic_stats,
        per_base_quality,
        per_sequence_quality: per_seq_quality,
        per_base_gc_content: per_base_gc,
        per_sequence_gc_content: per_seq_gc,
        sequence_length_dist: seq_length_dist,
        overrepresented_sequences: overrepresented,
        module_statuses,
    })
}

fn parse_basic_stat(stats: &mut FastqcBasicStats, key: &str, value: &str) {
    match key.trim() {
        "Filename" => stats.filename = value.trim().to_string(),
        "File type" => stats.file_type = value.trim().to_string(),
        "Encoding" => stats.encoding = value.trim().to_string(),
        "Total Sequences" => {
            stats.total_sequences = value.trim().parse().unwrap_or(0);
        }
        "Sequences flagged as poor quality" => {
            stats.sequences_flagged_poor_quality = value.trim().parse().unwrap_or(0);
        }
        "Sequence length" => stats.sequence_length = value.trim().to_string(),
        "%GC" => {
            stats.percent_gc = value.trim().parse().unwrap_or(0.0);
        }
        _ => {}
    }
}

fn parse_per_base_quality(fields: &[&str], path: &Path) -> Result<PerBaseQuality> {
    Ok(PerBaseQuality {
        base: fields[0].trim().to_string(),
        mean: parse_f64(path, "PBQ.mean", fields[1])?,
        median: parse_f64(path, "PBQ.median", fields[2])?,
        lower_quartile: parse_f64(path, "PBQ.lq", fields[3])?,
        upper_quartile: parse_f64(path, "PBQ.uq", fields[4])?,
        percentile_10: parse_f64(path, "PBQ.p10", fields[5])?,
        percentile_90: parse_f64(path, "PBQ.p90", fields[6])?,
    })
}

fn parse_f64(path: &Path, field: &str, value: &str) -> Result<f64> {
    value
        .trim()
        .parse::<f64>()
        .map_err(|_| QcForgeError::NumericParse {
            field: format!("{}:{}", path.display(), field),
            value: value.trim().to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const SAMPLE_FASTQC_DATA: &str = "\
##FastQC\t0.11.9
>>Basic Statistics\tpass
#Measure\tValue
Filename\tsample_R1.fastq.gz
File type\tConventional base calls
Encoding\tSanger / Illumina 1.9
Total Sequences\t71019254
Sequences flagged as poor quality\t0
Sequence length\t151
%GC\t42
>>END_MODULE
>>Per base sequence quality\tpass
#Base\tMean\tMedian\tLower Quartile\tUpper Quartile\t10th Percentile\t90th Percentile
1\t32.5\t34.0\t30.0\t36.0\t26.0\t37.0
2\t32.8\t34.0\t30.0\t36.0\t26.0\t37.0
3\t32.6\t34.0\t30.0\t36.0\t26.0\t37.0
4\t34.2\t36.0\t32.0\t37.0\t28.0\t38.0
5\t34.5\t36.0\t32.0\t37.0\t28.0\t38.0
10-14\t36.1\t37.0\t35.0\t38.0\t32.0\t39.0
50-54\t36.8\t38.0\t36.0\t39.0\t33.0\t39.0
100-104\t35.2\t36.0\t33.0\t38.0\t29.0\t39.0
145-149\t30.1\t32.0\t26.0\t35.0\t20.0\t37.0
150-151\t28.5\t30.0\t24.0\t34.0\t18.0\t36.0
>>END_MODULE
>>Per sequence quality scores\tpass
#Quality\tCount
20\t150.0
25\t2500.0
30\t15000.0
35\t48000.0
36\t5369254.0
>>END_MODULE
>>Per base sequence content\twarn
#Base\tG\tA\tT\tC
1\t22.5\t30.2\t25.1\t22.2
2\t21.8\t28.5\t26.3\t23.4
3\t23.1\t27.0\t25.8\t24.1
>>END_MODULE
>>Per sequence GC content\twarn
#GC Content\tCount
0\t100.0
10\t500.0
20\t5000.0
30\t15000.0
40\t28000.0
50\t18000.0
60\t3500.0
70\t800.0
80\t100.0
100\t19.254
>>END_MODULE
>>Sequence Length Distribution\tpass
#Length\tCount
151\t71019254.0
>>END_MODULE
>>Overrepresented sequences\tfail
#Sequence\tCount\tPercentage\tPossible Source
AGATCGGAAGAGCACACGTCTGAACTCCAGTCA\t285000\t0.40\tTruSeq Adapter
AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT\t142000\t0.20\tTruSeq Adapter
>>END_MODULE";

    #[test]
    fn test_parse_basic_statistics() {
        let path = PathBuf::from("test_fastqc.zip");
        let result = parse_fastqc_data(&path, SAMPLE_FASTQC_DATA).unwrap();

        assert_eq!(result.basic_statistics.filename, "sample_R1.fastq.gz");
        assert_eq!(result.basic_statistics.total_sequences, 71019254);
        assert_eq!(result.basic_statistics.sequence_length, "151");
        assert_eq!(result.basic_statistics.percent_gc, 42.0);
        assert_eq!(result.sample_name, "sample_R1.fastq.gz");
    }

    #[test]
    fn test_parse_module_statuses() {
        let path = PathBuf::from("test_fastqc.zip");
        let result = parse_fastqc_data(&path, SAMPLE_FASTQC_DATA).unwrap();

        assert_eq!(result.module_statuses.len(), 7);
        assert_eq!(result.module_statuses[0].0, "Basic Statistics");
        assert_eq!(result.module_statuses[0].1, ModuleStatus::Pass);
        assert_eq!(result.module_statuses[3].0, "Per base sequence content");
        assert_eq!(result.module_statuses[3].1, ModuleStatus::Warn);
        assert_eq!(result.module_statuses[6].0, "Overrepresented sequences");
        assert_eq!(result.module_statuses[6].1, ModuleStatus::Fail);
    }

    #[test]
    fn test_parse_per_base_quality() {
        let path = PathBuf::from("test_fastqc.zip");
        let result = parse_fastqc_data(&path, SAMPLE_FASTQC_DATA).unwrap();

        assert_eq!(result.per_base_quality.len(), 10);
        assert_eq!(result.per_base_quality[0].base, "1");
        assert!((result.per_base_quality[0].mean - 32.5).abs() < 0.01);
        assert_eq!(result.per_base_quality[7].base, "100-104");
    }

    #[test]
    fn test_parse_per_sequence_quality() {
        let path = PathBuf::from("test_fastqc.zip");
        let result = parse_fastqc_data(&path, SAMPLE_FASTQC_DATA).unwrap();

        assert_eq!(result.per_sequence_quality.len(), 5);
        assert_eq!(result.per_sequence_quality[0].quality, 20);
        assert!((result.per_sequence_quality[4].count - 5369254.0).abs() < 0.1);
    }

    #[test]
    fn test_parse_gc_content() {
        let path = PathBuf::from("test_fastqc.zip");
        let result = parse_fastqc_data(&path, SAMPLE_FASTQC_DATA).unwrap();

        assert_eq!(result.per_sequence_gc_content.len(), 10);
        assert_eq!(result.per_base_gc_content.len(), 3);
    }

    #[test]
    fn test_parse_overrepresented() {
        let path = PathBuf::from("test_fastqc.zip");
        let result = parse_fastqc_data(&path, SAMPLE_FASTQC_DATA).unwrap();

        assert_eq!(result.overrepresented_sequences.len(), 2);
        assert_eq!(result.overrepresented_sequences[0].count, 285000);
        assert!((result.overrepresented_sequences[0].percentage - 0.40).abs() < 0.01);
        assert_eq!(
            result.overrepresented_sequences[0].possible_source,
            "TruSeq Adapter"
        );
    }
}
