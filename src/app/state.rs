use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::parser::types::QcResults;
use crate::threshold::ThresholdConfig;

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
pub enum SummarySortColumn {
    File,
    Tool,
    Reads,
    MappedPct,
    DupPct,
    ErrorRate,
    AvgQuality,
    Variants,
    Snps,
    Indels,
    TsTv,
    TotalSeqs,
    GcPct,
    PassModules,
    WarnModules,
    FailModules,
    OverallQc,
}

impl SummarySortColumn {
    pub fn next(&self) -> Self {
        match self {
            SummarySortColumn::File => SummarySortColumn::Tool,
            SummarySortColumn::Tool => SummarySortColumn::Reads,
            SummarySortColumn::Reads => SummarySortColumn::MappedPct,
            SummarySortColumn::MappedPct => SummarySortColumn::DupPct,
            SummarySortColumn::DupPct => SummarySortColumn::ErrorRate,
            SummarySortColumn::ErrorRate => SummarySortColumn::AvgQuality,
            SummarySortColumn::AvgQuality => SummarySortColumn::Variants,
            SummarySortColumn::Variants => SummarySortColumn::Snps,
            SummarySortColumn::Snps => SummarySortColumn::Indels,
            SummarySortColumn::Indels => SummarySortColumn::TsTv,
            SummarySortColumn::TsTv => SummarySortColumn::TotalSeqs,
            SummarySortColumn::TotalSeqs => SummarySortColumn::GcPct,
            SummarySortColumn::GcPct => SummarySortColumn::PassModules,
            SummarySortColumn::PassModules => SummarySortColumn::WarnModules,
            SummarySortColumn::WarnModules => SummarySortColumn::FailModules,
            SummarySortColumn::FailModules => SummarySortColumn::OverallQc,
            SummarySortColumn::OverallQc => SummarySortColumn::File,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            SummarySortColumn::File => 0,
            SummarySortColumn::Tool => 1,
            SummarySortColumn::Reads => 2,
            SummarySortColumn::MappedPct => 3,
            SummarySortColumn::DupPct => 4,
            SummarySortColumn::ErrorRate => 5,
            SummarySortColumn::AvgQuality => 6,
            SummarySortColumn::Variants => 7,
            SummarySortColumn::Snps => 8,
            SummarySortColumn::Indels => 9,
            SummarySortColumn::TsTv => 10,
            SummarySortColumn::TotalSeqs => 11,
            SummarySortColumn::GcPct => 12,
            SummarySortColumn::PassModules => 13,
            SummarySortColumn::WarnModules => 14,
            SummarySortColumn::FailModules => 15,
            SummarySortColumn::OverallQc => 16,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Summary,
    Overview,
    Samtools,
    Bcftools,
    Fastqc,
}

impl ActiveTab {
    pub fn title(&self) -> &'static str {
        match self {
            ActiveTab::Summary => "Summary",
            ActiveTab::Overview => "Overview",
            ActiveTab::Samtools => "samtools",
            ActiveTab::Bcftools => "bcftools",
            ActiveTab::Fastqc => "FastQC",
        }
    }

    pub fn all() -> &'static [ActiveTab] {
        &[
            ActiveTab::Summary,
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
    pub summary_sort_column: SummarySortColumn,
    pub summary_horizontal_offset: u16,
    pub summary_selected: usize,
    pub thresholds: ThresholdConfig,
}

impl AppState {
    pub fn new(search_active_flag: Arc<AtomicBool>, thresholds: ThresholdConfig) -> Self {
        Self {
            active_tab: ActiveTab::Summary,
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
            summary_sort_column: SummarySortColumn::File,
            summary_horizontal_offset: 0,
            summary_selected: 0,
            thresholds,
        }
    }

    pub(crate) fn set_search_active(&mut self, active: bool) {
        self.search_active = active;
        self.search_active_flag.store(active, Ordering::Relaxed);
    }
}
