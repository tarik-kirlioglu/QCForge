# QCForge

A terminal-based (TUI) bioinformatics QC dashboard that aggregates and visualizes quality control outputs from **FastQC**, **samtools stats**, and **bcftools stats** in a single interactive interface.

Built with Rust using [ratatui](https://github.com/ratatui/ratatui) + [crossterm](https://github.com/crossterm-rs/crossterm).

## Features

- **Auto-detection** — Scans directories recursively and automatically identifies QC output files
- **Auto-generation** — Runs `samtools stats`, `bcftools stats`, and `fastqc` directly from BAM/VCF/FASTQ files (`--generate`)
- **5 interactive tabs** — Summary (MultiQC-style), Overview dashboard, samtools stats, bcftools stats, FastQC
- **QC thresholds** — Built-in quality thresholds with color-coded cells (green/yellow/red), customizable via TOML config
- **CI/CD integration** — `--strict` flag exits with code 1 if any sample fails QC thresholds
- **Visual metrics** — Gauges for mapping/duplication rates, inline bar charts for substitution types and indel distributions, colored PASS/WARN/FAIL indicators
- **JSON export** — Export all parsed QC data as JSON for downstream analysis (`--export-json`)
- **CSV/TSV export** — Export flat summary table as CSV or TSV (`--export-csv`, auto-detects `.tsv` for tab-delimited)
- **Sortable overview** — Sort the file list by any column with `s`/`S` keys
- **Search & filter** — Filter files by name with `/` key, real-time filtering as you type
- **Animated splash screen** — ASCII logo with fade-in effect and spark particle animation during loading
- **Async & responsive** — Non-blocking file loading with tokio, TUI stays responsive during parsing

## Installation

### From source

```bash
git clone https://github.com/tarik-kirlioglu/QCForge.git
cd QCForge
cargo install --path .
```

This installs the `qcforge` binary to `~/.cargo/bin/`. Make sure it's in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Requirements

- Rust 1.78+
- For `--generate` flag: `samtools`, `bcftools`, and/or `fastqc` in PATH (only needed tools are checked)

## Usage

```bash
# Scan a directory for existing QC outputs and launch TUI
qcforge /path/to/qc_outputs/

# Auto-generate stats from BAM/VCF/FASTQ files, then launch TUI
qcforge --generate /path/to/data_dir/

# Generate stats to a specific output directory
qcforge --generate --output-dir ./stats/ /path/to/data/

# Export QC data as JSON (no TUI)
qcforge --export-json results.json /path/to/qc_outputs/

# Export QC summary as CSV or TSV (no TUI)
qcforge --export-csv summary.csv /path/to/qc_outputs/
qcforge --export-csv summary.tsv /path/to/qc_outputs/

# Export both JSON and CSV at once
qcforge --export-json results.json --export-csv summary.csv /path/to/data/

# Generate + export combined
qcforge --generate --export-json results.json /path/to/data/

# Use custom QC thresholds
qcforge --thresholds custom_thresholds.toml /path/to/qc_outputs/

# Strict mode for CI/CD (exit code 1 if any FAIL)
qcforge --strict --export-csv summary.csv /path/to/qc_outputs/
```

### Supported Input Files

| File Type | Detection Method |
|-----------|-----------------|
| samtools stats output | Header contains "produced by samtools stats" |
| bcftools stats output | Header contains "produced by bcftools stats" |
| FastQC zip | Filename matches `*_fastqc.zip` |
| BAM files (`--generate`) | Extension `.bam` |
| VCF files (`--generate`) | Extension `.vcf`, `.vcf.gz`, `.bcf` |
| FASTQ files (`--generate`) | Extension `.fastq`, `.fastq.gz`, `.fq`, `.fq.gz` |

### CLI Options

```
Usage: qcforge [OPTIONS] [DIR]

Arguments:
  [DIR]  Directory to scan for QC output files [default: .]

Options:
  -g, --generate           Auto-generate stats from BAM/VCF/FASTQ files
      --output-dir <DIR>   Output directory for generated stats files
      --export-json <FILE> Export parsed QC data as JSON and exit
      --export-csv <FILE>  Export QC summary as CSV/TSV and exit (.tsv = tab-delimited)
      --thresholds <FILE>  QC threshold config file (TOML format)
      --strict             Exit with code 1 if any sample fails QC thresholds
  -f, --filter <FILTER>    Only show results for a specific tool [samtools|bcftools|fastqc]
      --max-depth <N>      Maximum directory depth for recursive scan [default: 5]
  -h, --help               Print help
  -V, --version            Print version
```

## Keybindings

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `←` / `→` / `Tab` | Switch tabs |
| `j` / `k` / `↑` / `↓` | Scroll |
| `n` / `p` | Next / Previous file |
| `s` | Cycle sort column (File → Tool → Summary → Status) |
| `S` | Toggle sort direction (ascending / descending) |
| `/` | Search mode (type to filter, Enter to confirm, Esc to clear) |
| `h` / `l` | Scroll columns left/right (Summary tab) |
| `?` | Toggle help overlay |
| `Ctrl+C` | Quit |

## Tabs

### Summary
MultiQC-style table showing all samples with all key metrics in a single wide view. Columns include reads, mapping%, duplication%, error rate, average quality, variants, SNPs, indels, Ts/Tv ratio, sequences, GC%, and module pass/warn/fail counts. Each metric cell is color-coded based on QC thresholds (green=pass, yellow=warn, red=fail). Use `h`/`l` to scroll columns horizontally (File column is frozen).

### Overview
Aggregate dashboard showing total files discovered, quick stats (total reads, mapped reads, error rate, variant count), average mapping/duplication gauges, and a combined file list with per-file status. The file list is sortable by any column (`s`/`S`) and filterable by filename (`/`).

### samtools
Summary Numbers table with all key metrics, plus Mapping Rate, Duplication Rate, and Properly Paired gauges with color-coded thresholds.

### bcftools
Variant summary table, Ts/Tv statistics, substitution type distribution with inline bar charts (transitions in cyan, transversions in magenta), and indel length distribution (deletions in red, insertions in green).

### FastQC
Basic statistics, module status list with colored PASS/WARN/FAIL indicators, and per-base quality bar chart with mean quality color coding.

## Project Structure

```
src/
├── main.rs         # Entry point, terminal setup, tokio runtime
├── cli.rs          # CLI argument definitions (clap)
├── error.rs        # Custom error types (thiserror)
├── export.rs       # CSV/TSV export module
├── threshold.rs    # QC threshold engine
├── app/            # State machine (Action pattern)
├── event/          # Async event handling (crossterm EventStream)
├── generator/      # BAM/VCF/FASTQ → stats generation (subprocess)
├── parser/         # File parsers (samtools, bcftools, FastQC)
├── scanner/        # Directory scanning and file type detection
└── ui/             # TUI rendering (ratatui)
    ├── tabs/       # Per-tab render modules
    └── widgets/    # Reusable widget helpers
```

## QC Thresholds

Default thresholds are built-in and can be overridden with a TOML config file:

```toml
[mapping_rate]
warn = 90.0
fail = 80.0
direction = "HigherIsBetter"

[duplication_rate]
warn = 15.0
fail = 30.0
direction = "LowerIsBetter"

[error_rate]
warn = 0.005
fail = 0.01
direction = "LowerIsBetter"

[ts_tv_ratio]
warn = 1.8
fail = 1.5
direction = "HigherIsBetter"

[gc_deviation]
warn = 15.0
fail = 25.0
direction = "LowerIsBetter"
```

Use `--thresholds custom.toml` to load custom thresholds, or `--strict` to exit with code 1 if any sample fails.

## License

[MIT](LICENSE)
