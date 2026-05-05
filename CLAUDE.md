# QCForge

Bioinformatics QC dashboard TUI application. Parses FastQC, samtools stats, and bcftools stats outputs and provides interactive visualization in the terminal. Supports automatic stats generation from BAM/VCF files and running FastQC on FASTQ files.

## Tech Stack

- **Language:** Rust (edition 2021)
- **TUI:** ratatui 0.30 + crossterm 0.29 (event-stream feature)
- **Async:** tokio (full features)
- **Error handling:** thiserror (custom error types, anyhow is not used)
- **CLI:** clap 4.5 (derive feature)
- **FastQC zip:** zip 8.2
- **Serialization:** serde + serde_json (for JSON export)
- **CSV export:** csv 1.3 (for CSV/TSV export)
- **Config:** toml 0.8 (for threshold config files)
- **File discovery:** glob
- **Stream:** futures (for crossterm EventStream)

## Commands

```bash
cargo build                          # Build
cargo run -- <DIR>                   # Launch TUI (scanning directory)
cargo run -- .                       # Scan current directory
cargo run -- -g <DIR>                # Generate stats from BAM/VCF/FASTQ + TUI
cargo run -- --generate <DIR>        # Long form
cargo run -- -g --output-dir ./out/ <DIR>  # Write stats to a different directory
cargo run -- --export-json qc.json <DIR>   # JSON export (no TUI)
cargo run -- --export-csv qc.csv <DIR>     # CSV export (no TUI)
cargo run -- --export-csv qc.tsv <DIR>     # TSV export (.tsv extension = tab-delimited)
cargo run -- --export-html qc.html <DIR>   # Self-contained static HTML report (no TUI)
cargo run -- --export-json qc.json --export-csv qc.csv <DIR>  # Both at once
cargo run -- -g --export-json qc.json <DIR> # Generate + JSON export
cargo run -- --thresholds custom.toml <DIR>  # Custom threshold config
cargo run -- --strict --export-csv out.csv <DIR>  # Strict mode (exit 1 on FAIL)
cargo run -- --metadata samples.tsv <DIR>    # Cohort grouping by panel/case_type/batch/...
cargo test                           # Run all tests (48 tests)
cargo clippy                         # Lint check
cargo fmt                            # Code formatting
```

## Architecture

```
src/
├── main.rs        # Entry point, terminal setup/restore, tokio runtime
├── cli.rs         # CLI arguments via clap derive
├── error.rs       # QcForgeError enum (thiserror)
├── export.rs      # CSV/TSV export (QcRow flat struct, serde serialize, threshold-aware qc_status)
├── html_export.rs # Static HTML report (Summary + Cohort SVG boxplots + per-tool detail; no JS)
├── threshold.rs   # QC threshold engine (MetricThreshold, ThresholdConfig, TOML config)
├── app/           # Application state machine (Action pattern)
├── event/         # crossterm event handling (async EventStream, Arc<AtomicBool> search state)
├── generator/     # BAM/VCF/FASTQ → stats file generation (runs samtools/bcftools/fastqc)
├── parser/        # File parsers (samtools, bcftools, fastqc)
├── scanner/       # Directory scanning, file type detection (stats + BAM/VCF/FASTQ)
├── metadata.rs    # Sample metadata TSV reader + derive_sample_id (cohort grouping)
└── ui/            # ratatui render layer (6 tabs + widgets)
    ├── tabs/      # Summary, Overview, samtools, bcftools, FastQC, Cohort (IQR boxplot + outlier list)
    └── widgets/   # gauge, table helpers
```

## CLI Flags

| Flag | Short | Description |
|------|-------|-------------|
| `<DIR>` | | Directory to scan (default: `.`) |
| `--generate` | `-g` | Auto-run samtools/bcftools/fastqc when BAM/VCF/FASTQ files are found |
| `--output-dir` | | Output directory for generated stats files |
| `--export-json` | | JSON export (TUI does not open) |
| `--export-csv` | | CSV/TSV export (TUI does not open, .tsv extension = tab-delimited) |
| `--export-html` | | Self-contained static HTML report (TUI does not open, embedded CSS + inline SVG, no JS) |
| `--thresholds` | | QC threshold config file (TOML format, default: built-in thresholds) |
| `--strict` | | Exit with code 1 if any FAIL (for CI/CD integration) |
| `--metadata` | | Sample metadata TSV (sample_id + arbitrary annotation columns become Cohort grouping dimensions) |
| `--filter` | `-f` | Show only a specific tool (samtools/bcftools/fastqc) |
| `--max-depth` | | Recursive scan depth (default: 5) |

## Code Rules

- **Error handling:** All errors go through the `QcForgeError` enum. `unwrap()` or `expect()` may only be used in test code. Production code uses `?` operator for propagation.
- **Naming:** Rust standard snake_case. Struct names PascalCase. Module names snake_case.
- **Imports:** Internal imports via `use crate::`. Wildcard imports (`use x::*`) are forbidden; use explicit imports.
- **Clippy:** `cargo clippy` must pass with no warnings. `#[allow(...)]` only with a justification comment.
- **Tests:** Each parser module must contain its own unit tests (`#[cfg(test)] mod tests`). Test data as inline strings. Currently 52 tests (18 parser + 4 export + 11 threshold + 3 summary + 7 cohort + 5 metadata + 4 html_export).
- **Async:** File I/O runs in background via tokio::spawn. FastQC zip processing uses spawn_blocking. The TUI event loop must never block.
- **Terminal restore:** Terminal state must be restored even on panic (use panic hook).

## Important Notes

- samtools stats and bcftools stats formats are similar but different. Field indices in the SN section differ.
- bcftools stats DP section bin values can be strings (e.g. `>500`). `DpEntry.bin` type is `String`.
- FastQC zip archives may have varying paths for `*/fastqc_data.txt`; matched using `ends_with`.
- The `##FastQC` header line starts with `#`, not `>>` — be careful in the section parser.
- ratatui redraws the entire UI every frame (immediate mode). Keep state separate from UI. `draw()` always takes `&AppState`; state mutation only happens in `update()`.
- Splash screen: An animated ice-blue ASCII logo over a full-screen grid of A/T/G/C letters. Letter positions are fixed (deterministic hash); animation uses double-helix sine wave coloring — letters on the helix are bright, near helix are mid-tint, far from helix are dim (near-black). `splash_tick` increments every 250ms (in the `Action::Render` handler); transitions to the main UI after 24 ticks (~6s) + data ready. `pending_results` buffers data until splash finishes. `splash_status` shows dynamic progress messages (e.g. "Running samtools stats on sample.bam").
- crossterm event-stream feature requires futures StreamExt.
- `--generate` mode skips if stats file / `_fastqc.zip` already exists (idempotent).
- Generator checks whether samtools/bcftools/fastqc are available in PATH.
- Generator accepts an `on_progress` callback for per-file status updates. In TUI mode, the callback sends `Action::SplashStatus`; in export mode, it writes to stderr via eprintln.
- FastQC output naming: `sample.fastq.gz` → `sample_fastqc.zip`. FastQC writes to disk itself; no stdout capture needed.
- Overview tab uses `OverviewRow` struct for sort/filter infrastructure. To add a new column: update `build_overview_rows()` and the `SortColumn` enum.
- Search mode shares `Arc<AtomicBool>` between EventHandler and AppState (for async tokio task access).
- CSV export uses `Option<T>` fields; the csv crate writes None as empty string.
- `--export-json` and `--export-csv` can be used together; data is loaded once.
- Summary tab supports horizontal scroll with `h`/`l` (frozen File column + scrollable metric columns).
- `SummarySortColumn` enum supports sorting across 17 columns; `s` key is context-aware: cycles SummarySortColumn in Summary tab, SortColumn in others.
- ThresholdConfig can be loaded from TOML via `#[derive(Deserialize)]`. Default values follow bioinformatics standards.
- GC% threshold is implemented as `gc_deviation`: `abs(gc - 50.0)` is computed and evaluated with LowerIsBetter.
- `--strict` mode evaluates all samples via `check_qc_failures()` after export; exits with code 1 if any FAIL.
- CSV export adds a `qc_status` column when thresholds are provided (`#[serde(skip_serializing_if = "Option::is_none")]`).
- Cohort tab applies cohort-relative outlier detection across the loaded samples for 5 metrics (Mapping %, Dup %, Error rate, Ts/Tv, GC dev). Outliers are flagged via Tukey IQR fences (`Q1 − 1.5·IQR`, `Q3 + 1.5·IQR`) using linear-interpolation quartiles (numpy default, "Type 7"). The mutual `ThresholdConfig` is shown as an orthogonal signal: cohort outliers without absolute-FAIL render in yellow (`o`), cohort outliers that *also* fail an absolute threshold render in red (`●`). Each metric needs ≥5 samples to render a boxplot; otherwise a per-row "n=N — too small" hint is shown. If no metric qualifies, the whole tab shows a single warning message. Cohort detection is TUI-only — CSV/JSON exports are unchanged.
- `--metadata <FILE>` loads a TSV with a required `sample_id` column plus arbitrary annotation columns (each becomes a grouping dimension). The `g` key cycles through dimensions; per-dimension IQR is computed on partitioned subsets so heterogeneous cohorts (WES + WGS + CES, tumor + normal) don't pollute each other's outlier flags. Sample-to-row matching strips known suffixes (`_fastqc.zip`, `.vcf.stats`, `.stats`, single trailing extension). Samples missing from the metadata fall into the `Ungrouped` bucket. With grouping active, the Cohort layout switches to 50/50 (boxplots top, outlier list bottom) and the boxplot panel scrolls vertically with `j/k`; the outlier list uses TableState selection driven by `n/p`. Outlier list gains a `Group` column. `derive_sample_id` lives in `src/metadata.rs` and is the single source of truth for matching.
- `--export-html <FILE>` writes a single self-contained static HTML report. No external assets, no JavaScript — embedded CSS (light + dark via `prefers-color-scheme`) and inline SVG charts (Cohort boxplots use the same IQR math as the TUI but render as `<rect>`/`<line>`/`<circle>` for vector-perfect zoom). The report mirrors the TUI: Summary table with threshold-colored cells, Cohort outlier analysis with per-metric/per-group boxplots, then per-tool collapsible detail (`<details>`) sections. When `--metadata` is passed alongside, the HTML defaults to grouping by the first dimension (TUI keeps None as default; the export needs a single shot). `cohort::CohortRow`, `build_cohort_rows`, and `collect_all_outliers` are intentionally `pub` for cross-module reuse from `html_export.rs`.
