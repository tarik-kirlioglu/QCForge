use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// All QC results discovered and parsed from a directory
#[derive(Debug, Default, Serialize)]
pub struct QcResults {
    pub scan_path: PathBuf,
    pub samtools_reports: Vec<SamtoolsStats>,
    pub bcftools_reports: Vec<BcftoolsStats>,
    pub fastqc_reports: Vec<FastqcReport>,
}

// ── samtools stats ──────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SamtoolsStats {
    pub source_file: PathBuf,
    pub summary: SamtoolsSummary,
    pub coverage_histogram: Vec<CoverageEntry>,
    pub insert_size_histogram: Vec<InsertSizeEntry>,
    pub read_length_histogram: Vec<ReadLengthEntry>,
    pub gc_content_first: Vec<GcEntry>,
    pub gc_content_last: Vec<GcEntry>,
}

#[derive(Debug, Default, Serialize)]
pub struct SamtoolsSummary {
    pub raw_total_sequences: u64,
    pub filtered_sequences: u64,
    pub reads_mapped: u64,
    pub reads_unmapped: u64,
    pub reads_duplicated: u64,
    pub reads_mq0: u64,
    pub reads_qc_failed: u64,
    pub reads_properly_paired: u64,
    pub total_length: u64,
    pub bases_mapped: u64,
    pub bases_mapped_cigar: u64,
    pub error_rate: f64,
    pub average_length: f64,
    pub average_quality: f64,
    pub insert_size_average: f64,
    pub insert_size_std_deviation: f64,
    pub pairs_on_different_chromosomes: u64,
    pub raw: BTreeMap<String, String>,
}

impl SamtoolsSummary {
    pub fn mapping_percent(&self) -> f64 {
        if self.raw_total_sequences == 0 {
            return 0.0;
        }
        (self.reads_mapped as f64 / self.raw_total_sequences as f64) * 100.0
    }

    pub fn duplication_percent(&self) -> f64 {
        if self.raw_total_sequences == 0 {
            return 0.0;
        }
        (self.reads_duplicated as f64 / self.raw_total_sequences as f64) * 100.0
    }

    pub fn properly_paired_percent(&self) -> f64 {
        if self.raw_total_sequences == 0 {
            return 0.0;
        }
        (self.reads_properly_paired as f64 / self.raw_total_sequences as f64) * 100.0
    }
}

#[derive(Debug, Serialize)]
pub struct CoverageEntry {
    pub range: String,
    pub depth: u32,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct InsertSizeEntry {
    pub insert_size: u32,
    pub pairs_total: u64,
    pub pairs_inward: u64,
    pub pairs_outward: u64,
    pub pairs_other: u64,
}

#[derive(Debug, Serialize)]
pub struct ReadLengthEntry {
    pub length: u32,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct GcEntry {
    pub gc_percent: f64,
    pub count: u64,
}

// ── bcftools stats ──────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct BcftoolsStats {
    pub source_file: PathBuf,
    pub summary: BcftoolsSummary,
    pub tstv: TsTvStats,
    pub substitution_types: Vec<StEntry>,
    pub allele_freq: Vec<AfEntry>,
    pub qual_dist: Vec<QualEntry>,
    pub indel_dist: Vec<IddEntry>,
    pub depth_dist: Vec<DpEntry>,
}

#[derive(Debug, Default, Serialize)]
pub struct BcftoolsSummary {
    pub num_samples: u64,
    pub num_records: u64,
    pub num_no_alts: u64,
    pub num_snps: u64,
    pub num_mnps: u64,
    pub num_indels: u64,
    pub num_others: u64,
    pub num_multiallelic_sites: u64,
    pub num_multiallelic_snp_sites: u64,
    pub raw: BTreeMap<String, String>,
}

impl BcftoolsSummary {
    pub fn snp_percent(&self) -> f64 {
        if self.num_records == 0 {
            return 0.0;
        }
        (self.num_snps as f64 / self.num_records as f64) * 100.0
    }

    pub fn indel_percent(&self) -> f64 {
        if self.num_records == 0 {
            return 0.0;
        }
        (self.num_indels as f64 / self.num_records as f64) * 100.0
    }
}

#[derive(Debug, Default, Serialize)]
pub struct TsTvStats {
    pub ts: u64,
    pub tv: u64,
    pub ts_tv_ratio: f64,
    pub ts_first_alt: u64,
    pub tv_first_alt: u64,
    pub ts_tv_ratio_first_alt: f64,
}

#[derive(Debug, Serialize)]
pub struct StEntry {
    pub sub_type: String,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct AfEntry {
    pub allele_freq: f64,
    pub num_snps: u64,
    pub num_indels: u64,
}

#[derive(Debug, Serialize)]
pub struct QualEntry {
    pub quality: f64,
    pub num_snps: u64,
    pub num_indels: u64,
    pub num_tstv: f64,
}

#[derive(Debug, Serialize)]
pub struct IddEntry {
    pub length: i32,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct DpEntry {
    pub bin: u32,
    pub num_genotypes: u64,
    pub frac_genotypes: f64,
    pub num_sites: u64,
    pub frac_sites: f64,
}

// ── FastQC (Phase 3 placeholder) ────────────────────────

#[derive(Debug, Serialize)]
pub struct FastqcReport {
    pub source_file: PathBuf,
}
