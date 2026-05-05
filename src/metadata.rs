use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use crate::error::{QcForgeError, Result};

/// Sample-level annotations loaded from a TSV file.
///
/// The TSV must have a `sample_id` column. Every other column becomes a
/// grouping dimension (e.g. `panel`, `case_type`, `batch`) — the user picks
/// which dimension is active at runtime via the `g` key.
#[derive(Debug, Clone)]
pub struct SampleMetadata {
    /// `sample_id` → (`column_name` → `value`)
    pub samples: HashMap<String, HashMap<String, String>>,
    /// Grouping dimension column names in TSV header order (sample_id excluded).
    pub dimensions: Vec<String>,
}

impl SampleMetadata {
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        Self::load_from_reader(file, &path.display().to_string())
    }

    pub fn load_from_reader<R: Read>(rdr: R, source: &str) -> Result<Self> {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(rdr);

        let headers = reader.headers()?.clone();
        let id_idx = headers
            .iter()
            .position(|h| h == "sample_id")
            .ok_or_else(|| {
                QcForgeError::MetadataError(format!(
                    "missing required `sample_id` column in {}",
                    source
                ))
            })?;

        let dimensions: Vec<String> = headers
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != id_idx)
            .map(|(_, h)| h.to_string())
            .collect();

        let mut samples: HashMap<String, HashMap<String, String>> = HashMap::new();
        for record in reader.records() {
            let record = record?;
            let sample_id = record
                .get(id_idx)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            if sample_id.is_empty() {
                continue;
            }
            let mut row = HashMap::new();
            for dim in &dimensions {
                if let Some(idx) = headers.iter().position(|h| h == dim) {
                    if let Some(val) = record.get(idx) {
                        row.insert(dim.clone(), val.trim().to_string());
                    }
                }
            }
            samples.insert(sample_id, row);
        }

        Ok(Self {
            samples,
            dimensions,
        })
    }

    pub fn group_for(&self, sample_id: &str, dimension: &str) -> Option<&str> {
        self.samples
            .get(sample_id)
            .and_then(|row| row.get(dimension))
            .map(|v| v.as_str())
            .filter(|v| !v.is_empty())
    }

    #[cfg(test)]
    fn group_values(&self, dimension: &str) -> Vec<String> {
        let mut seen: Vec<String> = Vec::new();
        for row in self.samples.values() {
            if let Some(v) = row.get(dimension) {
                if !v.is_empty() && !seen.contains(v) {
                    seen.push(v.clone());
                }
            }
        }
        seen.sort();
        seen
    }
}

/// Derive a logical sample identifier from a QC output filename by stripping
/// known QCForge suffix conventions.
///
/// Examples:
/// - `sample_A.stats` → `sample_A`
/// - `tumor.vcf.stats` → `tumor`
/// - `sample_A_fastqc.zip` → `sample_A`
/// - `sample_A.fastq.gz_fastqc.zip` → `sample_A.fastq.gz` (only `_fastqc.zip` is stripped here;
///   FastQC parser uses the same convention)
pub fn derive_sample_id(filename: &str) -> String {
    let mut s = filename.to_string();

    if let Some(stripped) = s.strip_suffix("_fastqc.zip") {
        return stripped.to_string();
    }

    if let Some(stripped) = s.strip_suffix(".vcf.stats") {
        return stripped.to_string();
    }

    if let Some(stripped) = s.strip_suffix(".stats") {
        s = stripped.to_string();
    }

    if let Some(dot) = s.rfind('.') {
        let ext = &s[dot + 1..];
        if !ext.is_empty() && !ext.contains('/') {
            return s[..dot].to_string();
        }
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn load(content: &str) -> Result<SampleMetadata> {
        SampleMetadata::load_from_reader(Cursor::new(content), "<test>")
    }

    #[test]
    fn test_derive_sample_id_variants() {
        assert_eq!(derive_sample_id("sample_A.stats"), "sample_A");
        assert_eq!(derive_sample_id("tumor.vcf.stats"), "tumor");
        assert_eq!(derive_sample_id("sample_A_fastqc.zip"), "sample_A");
        assert_eq!(derive_sample_id("normal_R1_fastqc.zip"), "normal_R1");
        assert_eq!(derive_sample_id("sample_A.txt"), "sample_A");
        assert_eq!(derive_sample_id("sample_A"), "sample_A");
    }

    #[test]
    fn test_load_metadata_basic() {
        let md = load("sample_id\tpanel\tcase_type\nsample_A\tWES\tgermline\ntumor_X\tWGS\ttumor\nnormal_X\tWGS\tnormal\n").unwrap();
        assert_eq!(md.dimensions, vec!["panel", "case_type"]);
        assert_eq!(md.samples.len(), 3);
        assert_eq!(md.group_for("sample_A", "panel"), Some("WES"));
        assert_eq!(md.group_for("tumor_X", "case_type"), Some("tumor"));
    }

    #[test]
    fn test_load_metadata_missing_sample_id() {
        let err = load("name\tpanel\nfoo\tWES\n").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("sample_id"), "unexpected error: {}", msg);
    }

    #[test]
    fn test_group_for_missing_sample() {
        let md = load("sample_id\tpanel\nsample_A\tWES\n").unwrap();
        assert_eq!(md.group_for("not_present", "panel"), None);
        assert_eq!(md.group_for("sample_A", "no_such_dim"), None);
    }

    #[test]
    fn test_group_values_distinct_and_sorted() {
        let md = load("sample_id\tpanel\nA\tWES\nB\tWGS\nC\tWES\nD\tCES\n").unwrap();
        assert_eq!(md.group_values("panel"), vec!["CES", "WES", "WGS"]);
    }
}
