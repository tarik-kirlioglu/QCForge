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
pub fn generate_stats(
    raw_files: &[RawFile],
    output_dir: Option<&Path>,
) -> Result<Vec<PathBuf>> {
    let mut generated = Vec::new();
    let mut need_samtools = false;
    let mut need_bcftools = false;

    for f in raw_files {
        match f {
            RawFile::Bam(_) => need_samtools = true,
            RawFile::Vcf(_) => need_bcftools = true,
        }
    }

    if need_samtools {
        check_tool("samtools")?;
    }
    if need_bcftools {
        check_tool("bcftools")?;
    }

    for raw_file in raw_files {
        match raw_file {
            RawFile::Bam(bam_path) => {
                let stats_path = make_output_path(bam_path, ".stats", output_dir);
                if stats_path.exists() {
                    eprintln!("  [skip] {} (already exists)", stats_path.display());
                    generated.push(stats_path);
                    continue;
                }

                eprintln!("  [run]  samtools stats {} ...", bam_path.display());
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
                eprintln!("  [done] {}", stats_path.display());
                generated.push(stats_path);
            }
            RawFile::Vcf(vcf_path) => {
                let stats_path = make_output_path(vcf_path, ".vcf.stats", output_dir);
                if stats_path.exists() {
                    eprintln!("  [skip] {} (already exists)", stats_path.display());
                    generated.push(stats_path);
                    continue;
                }

                eprintln!("  [run]  bcftools stats {} ...", vcf_path.display());
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
                eprintln!("  [done] {}", stats_path.display());
                generated.push(stats_path);
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
                .unwrap_or(&name);
            clean.to_string()
        })
        .unwrap_or_else(|| "unknown".to_string());

    let dir = output_dir.unwrap_or_else(|| input.parent().unwrap_or(Path::new(".")));
    dir.join(format!("{}{}", stem, suffix))
}
