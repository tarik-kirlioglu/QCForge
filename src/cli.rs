use clap::Parser;
use std::path::PathBuf;

/// QCForge - Terminal QC Dashboard for Bioinformatics
#[derive(Parser, Debug)]
#[command(name = "qcforge", version, about, long_about = None)]
pub struct Cli {
    /// Directory to scan for QC output files
    #[arg(value_name = "DIR", default_value = ".")]
    pub input_dir: PathBuf,

    /// Only show results for a specific tool
    #[arg(short, long, value_enum)]
    pub filter: Option<ToolFilter>,

    /// Maximum directory depth for recursive scan
    #[arg(long, default_value_t = 5)]
    pub max_depth: usize,

    /// Export parsed QC data as JSON and exit (no TUI)
    #[arg(long, value_name = "FILE")]
    pub export_json: Option<PathBuf>,

    /// Export parsed QC summary as CSV/TSV and exit (no TUI)
    /// Use .tsv extension for tab-delimited output
    #[arg(long, value_name = "FILE")]
    pub export_csv: Option<PathBuf>,

    /// Auto-generate stats from BAM/VCF files using samtools/bcftools
    #[arg(short, long)]
    pub generate: bool,

    /// Output directory for generated stats files (default: same as input)
    #[arg(long, value_name = "DIR")]
    pub output_dir: Option<PathBuf>,

    /// QC threshold config file (TOML format, uses defaults if not specified)
    #[arg(long, value_name = "FILE")]
    pub thresholds: Option<PathBuf>,

    /// Exit with code 1 if any sample fails QC thresholds
    #[arg(long)]
    pub strict: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ToolFilter {
    Samtools,
    Bcftools,
    Fastqc,
}
