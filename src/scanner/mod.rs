use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::{QcForgeError, Result};

#[derive(Debug)]
pub enum DetectedFile {
    SamtoolsStats(PathBuf),
    BcftoolsStats(PathBuf),
    FastqcZip(PathBuf),
}

#[derive(Debug)]
pub enum RawFile {
    Bam(PathBuf),
    Vcf(PathBuf),
    Fastq(PathBuf),
}

pub fn scan_raw_files(dir: &Path, max_depth: usize) -> Result<Vec<RawFile>> {
    if !dir.is_dir() {
        return Err(QcForgeError::NoFilesFound(dir.display().to_string()));
    }

    let mut raw_files = Vec::new();
    walk_dir_raw(dir, max_depth, 0, &mut raw_files)?;
    Ok(raw_files)
}

fn walk_dir_raw(
    dir: &Path,
    max_depth: usize,
    current_depth: usize,
    results: &mut Vec<RawFile>,
) -> Result<()> {
    if current_depth > max_depth {
        return Ok(());
    }

    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            walk_dir_raw(&path, max_depth, current_depth + 1, results)?;
        } else if path.is_file() {
            if let Some(raw) = classify_raw_file(&path) {
                results.push(raw);
            }
        }
    }

    Ok(())
}

fn classify_raw_file(path: &Path) -> Option<RawFile> {
    let filename = path.file_name()?.to_str()?;
    let lower = filename.to_lowercase();

    if lower.ends_with(".bam") {
        return Some(RawFile::Bam(path.to_path_buf()));
    }

    if lower.ends_with(".vcf")
        || lower.ends_with(".vcf.gz")
        || lower.ends_with(".bcf")
    {
        return Some(RawFile::Vcf(path.to_path_buf()));
    }

    if lower.ends_with(".fastq.gz")
        || lower.ends_with(".fastq")
        || lower.ends_with(".fq.gz")
        || lower.ends_with(".fq")
    {
        return Some(RawFile::Fastq(path.to_path_buf()));
    }

    None
}

pub fn scan_directory(dir: &Path, max_depth: usize) -> Result<Vec<DetectedFile>> {
    if !dir.is_dir() {
        return Err(QcForgeError::NoFilesFound(dir.display().to_string()));
    }

    let mut detected = Vec::new();
    walk_dir(dir, max_depth, 0, &mut detected)?;

    if detected.is_empty() {
        return Err(QcForgeError::NoFilesFound(dir.display().to_string()));
    }

    Ok(detected)
}

fn walk_dir(
    dir: &Path,
    max_depth: usize,
    current_depth: usize,
    results: &mut Vec<DetectedFile>,
) -> Result<()> {
    if current_depth > max_depth {
        return Ok(());
    }

    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            walk_dir(&path, max_depth, current_depth + 1, results)?;
        } else if path.is_file() {
            if let Some(detected) = classify_file(&path) {
                results.push(detected);
            }
        }
    }

    Ok(())
}

fn classify_file(path: &Path) -> Option<DetectedFile> {
    let filename = path.file_name()?.to_str()?;

    // FastQC zip detection by filename pattern
    if filename.ends_with("_fastqc.zip") {
        return Some(DetectedFile::FastqcZip(path.to_path_buf()));
    }

    // For text files, check content header
    if let Ok(header) = read_header(path, 10) {
        if header.contains("This file was produced by samtools stats") {
            return Some(DetectedFile::SamtoolsStats(path.to_path_buf()));
        }
        if header.contains("This file was produced by bcftools stats") {
            return Some(DetectedFile::BcftoolsStats(path.to_path_buf()));
        }
    }

    None
}

fn read_header(path: &Path, max_lines: usize) -> std::result::Result<String, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut buf = vec![0u8; 4096];
    let n = file.read(&mut buf)?;
    let content = String::from_utf8_lossy(&buf[..n]);

    let header: String = content
        .lines()
        .take(max_lines)
        .collect::<Vec<_>>()
        .join("\n");

    Ok(header)
}
