mod app;
mod cli;
mod error;
mod event;
mod export;
mod generator;
mod threshold;
mod parser;
mod scanner;
mod ui;

use std::io;
use std::panic;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use clap::Parser;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;

use app::actions::Action;
use app::state::AppState;
use cli::Cli;
use error::Result;
use parser::types::QcResults;
use threshold::ThresholdConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load thresholds
    let thresholds = if let Some(ref path) = cli.thresholds {
        ThresholdConfig::load_from_file(path)?
    } else {
        ThresholdConfig::default()
    };

    // Export mode: no TUI, just parse and dump (generate runs synchronously here)
    if cli.export_json.is_some() || cli.export_csv.is_some() {
        if cli.generate {
            eprintln!("Scanning for BAM/VCF/FASTQ files...");
            let raw_files = scanner::scan_raw_files(&cli.input_dir, cli.max_depth)?;
            if raw_files.is_empty() {
                eprintln!("No BAM/VCF/FASTQ files found in {}", cli.input_dir.display());
            } else {
                eprintln!("Found {} BAM/VCF/FASTQ file(s). Generating stats...", raw_files.len());
                generator::generate_stats(&raw_files, cli.output_dir.as_deref(), |msg| {
                    eprintln!("  {}", msg);
                })?;
                eprintln!("Stats generation complete.\n");
            }
        }

        let results = load_qc_data(&cli.input_dir, cli.max_depth).await?;

        if let Some(ref json_path) = cli.export_json {
            let json = serde_json::to_string_pretty(&results)
                .map_err(|e| error::QcForgeError::Terminal(e.to_string()))?;
            std::fs::write(json_path, json)?;
            eprintln!("QC data exported to {}", json_path.display());
        }

        if let Some(ref csv_path) = cli.export_csv {
            export::write_csv(csv_path, &results, Some(&thresholds))?;
            eprintln!("QC summary exported to {}", csv_path.display());
        }

        if cli.strict {
            let has_fail = check_qc_failures(&results, &thresholds);
            if has_fail {
                eprintln!("QC FAIL: one or more samples failed threshold checks");
                std::process::exit(1);
            }
            eprintln!("QC PASS: all samples passed threshold checks");
        }

        return Ok(());
    }

    // Install panic hook to restore terminal
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Action channel
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();

    // Shared search state for event handler
    let search_active_flag = Arc::new(AtomicBool::new(false));

    // Spawn event handler
    let _event_handler = event::EventHandler::new(action_tx.clone(), search_active_flag.clone());

    // Spawn background task: generate (if requested) + load QC data
    let scan_path = cli.input_dir.clone();
    let max_depth = cli.max_depth;
    let do_generate = cli.generate;
    let output_dir = cli.output_dir.clone();
    let tx = action_tx.clone();
    tokio::spawn(async move {
        // Generate stats if requested
        if do_generate {
            let _ = tx.send(Action::SplashStatus("Scanning for BAM/VCF/FASTQ files".to_string()));
            let scan_path_gen = scan_path.clone();
            let raw_files = match tokio::task::spawn_blocking(move || {
                scanner::scan_raw_files(&scan_path_gen, max_depth)
            }).await {
                Ok(Ok(files)) => files,
                Ok(Err(e)) => {
                    let _ = tx.send(Action::Error(e.to_string()));
                    return;
                }
                Err(e) => {
                    let _ = tx.send(Action::Error(e.to_string()));
                    return;
                }
            };

            if !raw_files.is_empty() {
                let _ = tx.send(Action::SplashStatus(
                    format!("Generating stats ({} files)", raw_files.len()),
                ));
                let out_dir = output_dir.clone();
                let tx_gen = tx.clone();
                if let Err(e) = tokio::task::spawn_blocking(move || {
                    generator::generate_stats(&raw_files, out_dir.as_deref(), |msg| {
                        let _ = tx_gen.send(Action::SplashStatus(msg.to_string()));
                    })
                }).await.unwrap_or_else(|e| Err(error::QcForgeError::Terminal(e.to_string()))) {
                    let _ = tx.send(Action::Error(e.to_string()));
                    return;
                }
            }
        }

        // Load QC data
        let _ = tx.send(Action::SplashStatus("Loading QC data".to_string()));
        match load_qc_data(&scan_path, max_depth).await {
            Ok(results) => {
                let _ = tx.send(Action::LoadComplete(results));
            }
            Err(e) => {
                let _ = tx.send(Action::Error(e.to_string()));
            }
        }
    });

    // App state
    let mut state = AppState::new(search_active_flag, thresholds.clone());

    // Main loop
    loop {
        terminal.draw(|frame| ui::draw(frame, &state))?;

        if let Some(action) = action_rx.recv().await {
            state.update(action);
        }

        if state.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

async fn load_qc_data(
    scan_path: &std::path::Path,
    max_depth: usize,
) -> Result<QcResults> {
    let scan_path_buf = scan_path.to_path_buf();
    let detected = scanner::scan_directory(scan_path, max_depth)?;

    let mut results = QcResults {
        scan_path: scan_path_buf,
        ..Default::default()
    };

    for file in detected {
        match file {
            scanner::DetectedFile::SamtoolsStats(path) => {
                let content =
                    tokio::fs::read_to_string(&path).await.map_err(error::QcForgeError::Io)?;
                let stats = parser::samtools::parse_samtools_stats(&path, &content)?;
                results.samtools_reports.push(stats);
            }
            scanner::DetectedFile::BcftoolsStats(path) => {
                let content =
                    tokio::fs::read_to_string(&path).await.map_err(error::QcForgeError::Io)?;
                let stats = parser::bcftools::parse_bcftools_stats(&path, &content)?;
                results.bcftools_reports.push(stats);
            }
            scanner::DetectedFile::FastqcZip(path) => {
                let report = tokio::task::spawn_blocking(move || {
                    parser::fastqc::parse_fastqc_zip(&path)
                })
                .await
                .map_err(|e| error::QcForgeError::Terminal(e.to_string()))??;
                results.fastqc_reports.push(report);
            }
        }
    }

    Ok(results)
}

fn check_qc_failures(results: &QcResults, thresholds: &ThresholdConfig) -> bool {
    use threshold::QcLevel;

    for r in &results.samtools_reports {
        let level = thresholds.evaluate_sample(
            Some(r.summary.mapping_percent()),
            Some(r.summary.duplication_percent()),
            Some(r.summary.error_rate),
            None,
            None,
        );
        if level == QcLevel::Fail {
            return true;
        }
    }

    for r in &results.bcftools_reports {
        let level = thresholds.evaluate_sample(None, None, None, Some(r.tstv.ts_tv_ratio), None);
        if level == QcLevel::Fail {
            return true;
        }
    }

    for r in &results.fastqc_reports {
        let level = thresholds.evaluate_sample(
            None,
            None,
            None,
            None,
            Some(r.basic_statistics.percent_gc),
        );
        if level == QcLevel::Fail {
            return true;
        }
    }

    false
}
