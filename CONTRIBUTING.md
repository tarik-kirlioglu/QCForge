# Contributing to QCForge

Thank you for considering contributing to QCForge! This guide will help you get started.

## Getting Started

### Prerequisites

- **Rust** (edition 2021) — install via [rustup](https://rustup.rs/)
- **Optional (for `--generate` mode):** samtools, bcftools, fastqc in your PATH

### Setup

```bash
git clone https://github.com/tarik-kirlioglu/QCForge.git
cd QCForge
cargo build
cargo test
```

### Running

```bash
cargo run -- demo_data/          # Launch TUI with demo data
cargo run -- -g <DIR>            # Generate stats from BAM/VCF/FASTQ + TUI
cargo run -- --export-json qc.json <DIR>  # JSON export (no TUI)
```

## Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Ensure all checks pass:
   ```bash
   cargo fmt                # Code formatting
   cargo clippy             # Lint — must pass with zero warnings
   cargo test               # All tests must pass
   ```
5. Commit your changes with a clear message
6. Push to your fork and open a Pull Request

## Code Guidelines

### Error Handling

- All errors go through the `QcForgeError` enum in `src/error.rs`
- Use the `?` operator for error propagation
- `unwrap()` and `expect()` are only allowed in test code

### Naming

- Functions and variables: `snake_case`
- Structs and enums: `PascalCase`
- Modules: `snake_case`

### Imports

- Internal imports via `use crate::`
- No wildcard imports (`use x::*`)
- Use explicit imports only

### Clippy

- `cargo clippy` must pass with zero warnings
- `#[allow(...)]` only with a justification comment

### Tests

- Each parser module contains its own unit tests (`#[cfg(test)] mod tests`)
- Use inline strings for test data (no external test files)
- Add tests for any new parser or feature

### Async & TUI

- File I/O runs in background via `tokio::spawn`
- FastQC zip processing uses `spawn_blocking`
- The TUI event loop must never block
- State mutation only happens in `update()` — render functions take `&AppState`

## Architecture Overview

```
src/
├── main.rs        # Entry point, terminal setup/restore
├── cli.rs         # CLI arguments (clap derive)
├── error.rs       # QcForgeError enum (thiserror)
├── export.rs      # JSON/CSV/TSV export
├── threshold.rs   # QC threshold engine
├── app/           # Application state machine (Action pattern)
├── event/         # Crossterm event handling
├── generator/     # BAM/VCF/FASTQ → stats generation
├── parser/        # File parsers (samtools, bcftools, fastqc)
├── scanner/       # Directory scanning, file type detection
└── ui/            # ratatui render layer (5 tabs + widgets)
```

## What Can I Work On?

- **New parser support** — add parsers for other bioinformatics QC tools
- **New UI tabs** — extend the dashboard with additional visualizations
- **Export formats** — add new export targets (HTML report, PDF, etc.)
- **Threshold improvements** — more granular or per-tool threshold configs
- **Bug fixes** — check the [Issues](https://github.com/tarik-kirlioglu/QCForge/issues) page
- **Documentation** — improve README, add examples

## Reporting Issues

When reporting a bug, please include:

- QCForge version (`qcforge --version`)
- OS and terminal emulator
- Steps to reproduce
- Expected vs actual behavior
- Relevant input file format (samtools stats, bcftools stats, FastQC zip)

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.
