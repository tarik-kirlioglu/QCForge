use crate::parser::types::QcResults;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Samtools,
    Bcftools,
    // Fastqc,   // Phase 3
    // Overview, // Phase 4
}

impl ActiveTab {
    pub fn title(&self) -> &'static str {
        match self {
            ActiveTab::Samtools => "samtools",
            ActiveTab::Bcftools => "bcftools",
        }
    }

    pub fn all() -> &'static [ActiveTab] {
        &[ActiveTab::Samtools, ActiveTab::Bcftools]
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
    pub scroll_offset: u16,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            active_tab: ActiveTab::Samtools,
            should_quit: false,
            show_help: false,
            loading: true,
            error_message: None,
            qc_results: None,
            samtools_selected: 0,
            bcftools_selected: 0,
            scroll_offset: 0,
        }
    }
}
