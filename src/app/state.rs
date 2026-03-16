use crate::parser::types::QcResults;

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
}

impl AppState {
    pub fn new() -> Self {
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
        }
    }
}
