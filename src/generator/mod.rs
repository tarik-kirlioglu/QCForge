use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{QcForgeError, Result};
use crate::scanner::RawFile;

/// Check if a command is available in PATH
fn check_tool(name: &str) -> Result<()> {
    Command::new("which")
        .arg(name)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .ok_or_else(|| {
            QcForgeError::Terminal(format!(
                "'{}' not found in PATH. Install it or add it to your PATH.",
                name
            ))
        })?;
    Ok(())
}

/// Generate stats files from raw BAM/VCF files
/// Returns paths to newly generated stats files
/// `on_progress` is called with status messages for each file being processed
pub fn generate_stats(
    raw_files: &[RawFile],
    output_dir: Option<&Path>,
    on_progress: impl Fn(&str),
) -> Result<Vec<PathBuf>> {
    let mut generated = Vec::new();
    let mut need_samtools = false;
    let mut need_bcftools = false;
    let mut need_fastqc = false;

    for f in raw_files {
        match f {
            RawFile::Bam(_) => need_samtools = true,
            RawFile::Vcf(_) => need_bcftools = true,
            RawFile::Fastq(_) => need_fastqc = true,
        }
    }

    if need_samtools {
        check_tool("samtools")?;
    }
    if need_bcftools {
        check_tool("bcftools")?;
    }
    if need_fastqc {
        check_tool("fastqc")?;
    }

    for raw_file in raw_files {
        match raw_file {
            RawFile::Bam(bam_path) => {
                let stats_path = make_output_path(bam_path, ".stats", output_dir);
                if stats_path.exists() {
                    on_progress(&format!("[skip] {}", stats_path.display()));
                    generated.push(stats_path);
                    continue;
                }

                on_progress(&format!("Running samtools stats on {}", bam_path.file_name().unwrap_or_default().to_string_lossy()));
                let output = Command::new("samtools")
                    .arg("stats")
                    .arg(bam_path)
                    .output()
                    .map_err(|e| QcForgeError::Terminal(format!("samtools stats failed: {}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(QcForgeError::Terminal(format!(
                        "samtools stats failed for {}: {}",
                        bam_path.display(),
                        stderr.trim()
                    )));
                }

                std::fs::write(&stats_path, &output.stdout)?;
                generated.push(stats_path);
            }
            RawFile::Vcf(vcf_path) => {
                let stats_path = make_output_path(vcf_path, ".vcf.stats", output_dir);
                if stats_path.exists() {
                    on_progress(&format!("[skip] {}", stats_path.display()));
                    generated.push(stats_path);
                    continue;
                }

                on_progress(&format!("Running bcftools stats on {}", vcf_path.file_name().unwrap_or_default().to_string_lossy()));
                let output = Command::new("bcftools")
                    .arg("stats")
                    .arg(vcf_path)
                    .output()
                    .map_err(|e| QcForgeError::Terminal(format!("bcftools stats failed: {}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(QcForgeError::Terminal(format!(
                        "bcftools stats failed for {}: {}",
                        vcf_path.display(),
                        stderr.trim()
                    )));
                }

                std::fs::write(&stats_path, &output.stdout)?;
                generated.push(stats_path);
            }
            RawFile::Fastq(fq_path) => {
                let zip_path = make_output_path(fq_path, "_fastqc.zip", output_dir);
                if zip_path.exists() {
                    on_progress(&format!("[skip] {}", zip_path.display()));
                    generated.push(zip_path);
                    continue;
                }

                let out_dir = output_dir
                    .unwrap_or_else(|| fq_path.parent().unwrap_or(Path::new(".")));

                on_progress(&format!("Running fastqc on {}", fq_path.file_name().unwrap_or_default().to_string_lossy()));
                let output = Command::new("fastqc")
                    .arg(fq_path)
                    .arg("--outdir")
                    .arg(out_dir)
                    .arg("--quiet")
                    .output()
                    .map_err(|e| QcForgeError::Terminal(format!("fastqc failed: {}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(QcForgeError::Terminal(format!(
                        "fastqc failed for {}: {}",
                        fq_path.display(),
                        stderr.trim()
                    )));
                }

                generated.push(zip_path);
            }
        }
    }

    Ok(generated)
}

fn make_output_path(input: &Path, suffix: &str, output_dir: Option<&Path>) -> PathBuf {
    let stem = input
        .file_name()
        .map(|f| {
            let name = f.to_string_lossy();
            // Strip known extensions to get a clean stem
            let clean = name
                .strip_suffix(".bam")
                .or_else(|| name.strip_suffix(".vcf.gz"))
                .or_else(|| name.strip_suffix(".vcf"))
                .or_else(|| name.strip_suffix(".bcf"))
                .or_else(|| name.strip_suffix(".fastq.gz"))
                .or_else(|| name.strip_suffix(".fastq"))
                .or_else(|| name.strip_suffix(".fq.gz"))
                .or_else(|| name.strip_suffix(".fq"))
                .unwrap_or(&name);
            clean.to_string()
        })
        .unwrap_or_else(|| "unknown".to_string());

    let dir = output_dir.unwrap_or_else(|| input.parent().unwrap_or(Path::new(".")));
    dir.join(format!("{}{}", stem, suffix))
}
