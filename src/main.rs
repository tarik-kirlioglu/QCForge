mod app;
mod cli;
mod error;
mod event;
mod generator;
mod parser;
mod scanner;
mod ui;

use std::io;
use std::panic;

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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Generate stats from BAM/VCF if requested
    if cli.generate {
        eprintln!("Scanning for BAM/VCF files...");
        let raw_files = scanner::scan_raw_files(&cli.input_dir, cli.max_depth)?;
        if raw_files.is_empty() {
            eprintln!("No BAM/VCF files found in {}", cli.input_dir.display());
        } else {
            eprintln!("Found {} BAM/VCF file(s). Generating stats...", raw_files.len());
            generator::generate_stats(&raw_files, cli.output_dir.as_deref())?;
            eprintln!("Stats generation complete.\n");
        }
    }

    // JSON export mode: no TUI, just parse and dump
    if let Some(ref json_path) = cli.export_json {
        let results = load_qc_data(&cli.input_dir, cli.max_depth).await?;
        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| error::QcForgeError::Terminal(e.to_string()))?;
        std::fs::write(json_path, json)?;
        eprintln!("QC data exported to {}", json_path.display());
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

    // Spawn event handler
    let _event_handler = event::EventHandler::new(action_tx.clone());

    // Spawn file loading task
    let scan_path = cli.input_dir.clone();
    let max_depth = cli.max_depth;
    let tx = action_tx.clone();
    tokio::spawn(async move {
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
    let mut state = AppState::new();

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
