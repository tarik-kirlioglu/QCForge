use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::parser::types::QcResults;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    File,
    Tool,
    Summary,
    Status,
}

impl SortColumn {
    pub fn next(&self) -> Self {
        match self {
            SortColumn::File => SortColumn::Tool,
            SortColumn::Tool => SortColumn::Summary,
            SortColumn::Summary => SortColumn::Status,
            SortColumn::Status => SortColumn::File,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Samtools,
    Bcftools,
    Fastqc,
    Overview,
}

impl ActiveTab {
    pub fn title(&self) -> &'static str {
        match self {
            ActiveTab::Overview => "Overview",
            ActiveTab::Samtools => "samtools",
            ActiveTab::Bcftools => "bcftools",
            ActiveTab::Fastqc => "FastQC",
        }
    }

    pub fn all() -> &'static [ActiveTab] {
        &[
            ActiveTab::Overview,
            ActiveTab::Samtools,
            ActiveTab::Bcftools,
            ActiveTab::Fastqc,
        ]
    }
}

#[derive(Debug)]
pub struct AppState {
    pub active_tab: ActiveTab,
    pub should_quit: bool,
    pub show_help: bool,
    pub loading: bool,
    pub error_message: Option<String>,
    pub qc_results: Option<QcResults>,
    pub samtools_selected: usize,
    pub bcftools_selected: usize,
    pub fastqc_selected: usize,
    pub scroll_offset: u16,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub search_active: bool,
    pub search_input: String,
    pub search_confirmed: String,
    pub search_active_flag: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(search_active_flag: Arc<AtomicBool>) -> Self {
        Self {
            active_tab: ActiveTab::Overview,
            should_quit: false,
            show_help: false,
            loading: true,
            error_message: None,
            qc_results: None,
            samtools_selected: 0,
            bcftools_selected: 0,
            fastqc_selected: 0,
            scroll_offset: 0,
            sort_column: SortColumn::File,
            sort_ascending: true,
            search_active: false,
            search_input: String::new(),
            search_confirmed: String::new(),
            search_active_flag,
        }
    }

    pub(crate) fn set_search_active(&mut self, active: bool) {
        self.search_active = active;
        self.search_active_flag.store(active, Ordering::Relaxed);
    }
}
