# QCForge

A terminal-based (TUI) bioinformatics QC dashboard that aggregates and visualizes quality control outputs from **FastQC**, **samtools stats**, and **bcftools stats** in a single interactive interface.

Built with Rust using [ratatui](https://github.com/ratatui/ratatui) + [crossterm](https://github.com/crossterm-rs/crossterm).

## Features

- **Auto-detection** — Scans directories recursively and automatically identifies QC output files
- **Auto-generation** — Runs `samtools stats` and `bcftools stats` directly from BAM/VCF files (`--generate`)
- **4 interactive tabs** — Overview dashboard, samtools stats, bcftools stats, FastQC
- **Visual metrics** — Gauges for mapping/duplication rates, inline bar charts for substitution types and indel distributions, colored PASS/WARN/FAIL indicators
- **JSON export** — Export all parsed QC data as JSON for downstream analysis (`--export-json`)
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
- For `--generate` flag: `samtools` and/or `bcftools` in PATH

## Usage

```bash
# Scan a directory for existing QC outputs and launch TUI
qcforge /path/to/qc_outputs/

# Auto-generate stats from BAM/VCF files, then launch TUI
qcforge --generate /path/to/bam_vcf_dir/

# Generate stats to a specific output directory
qcforge --generate --output-dir ./stats/ /path/to/data/

# Export QC data as JSON (no TUI)
qcforge --export-json results.json /path/to/qc_outputs/

# Generate + export combined
qcforge --generate --export-json results.json /path/to/data/
```

### Supported Input Files

| File Type | Detection Method |
|-----------|-----------------|
| samtools stats output | Header contains "produced by samtools stats" |
| bcftools stats output | Header contains "produced by bcftools stats" |
| FastQC zip | Filename matches `*_fastqc.zip` |
| BAM files (`--generate`) | Extension `.bam` |
| VCF files (`--generate`) | Extension `.vcf`, `.vcf.gz`, `.bcf` |

### CLI Options

```
Usage: qcforge [OPTIONS] [DIR]

Arguments:
  [DIR]  Directory to scan for QC output files [default: .]

Options:
  -g, --generate           Auto-generate stats from BAM/VCF files
      --output-dir <DIR>   Output directory for generated stats files
      --export-json <FILE> Export parsed QC data as JSON and exit
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
| `?` | Toggle help overlay |
| `Ctrl+C` | Quit |

## Tabs

### Overview
Aggregate dashboard showing total files discovered, quick stats (total reads, mapped reads, error rate, variant count), average mapping/duplication gauges, and a combined file list with per-file status.

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
├── app/            # State machine (Action pattern)
├── event/          # Async event handling (crossterm EventStream)
├── generator/      # BAM/VCF → stats generation (subprocess)
├── parser/         # File parsers (samtools, bcftools, FastQC)
├── scanner/        # Directory scanning and file type detection
└── ui/             # TUI rendering (ratatui)
    ├── tabs/       # Per-tab render modules
    └── widgets/    # Reusable widget helpers
```

## License

[MIT](LICENSE)
