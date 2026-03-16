pub mod actions;
pub mod state;

use actions::Action;
use state::{ActiveTab, AppState};

impl AppState {
    pub fn update(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::NextTab => {
                let tabs = ActiveTab::all();
                if let Some(idx) = tabs.iter().position(|t| *t == self.active_tab) {
                    self.active_tab = tabs[(idx + 1) % tabs.len()];
                    self.scroll_offset = 0;
                }
            }
            Action::PrevTab => {
                let tabs = ActiveTab::all();
                if let Some(idx) = tabs.iter().position(|t| *t == self.active_tab) {
                    self.active_tab = tabs[(idx + tabs.len() - 1) % tabs.len()];
                    self.scroll_offset = 0;
                }
            }
            Action::ScrollDown => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
            }
            Action::ScrollUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            Action::NextFile => {
                if let Some(ref results) = self.qc_results {
                    match self.active_tab {
                        ActiveTab::Samtools => {
                            let max = results.samtools_reports.len().saturating_sub(1);
                            self.samtools_selected = (self.samtools_selected + 1).min(max);
                        }
                        ActiveTab::Bcftools => {
                            let max = results.bcftools_reports.len().saturating_sub(1);
                            self.bcftools_selected = (self.bcftools_selected + 1).min(max);
                        }
                    }
                    self.scroll_offset = 0;
                }
            }
            Action::PrevFile => {
                match self.active_tab {
                    ActiveTab::Samtools => {
                        self.samtools_selected = self.samtools_selected.saturating_sub(1);
                    }
                    ActiveTab::Bcftools => {
                        self.bcftools_selected = self.bcftools_selected.saturating_sub(1);
                    }
                }
                self.scroll_offset = 0;
            }
            Action::ToggleHelp => {
                self.show_help = !self.show_help;
            }
            Action::LoadComplete(results) => {
                self.qc_results = Some(results);
                self.loading = false;
            }
            Action::Error(msg) => {
                self.error_message = Some(msg);
                self.loading = false;
            }
            Action::Resize(_, _) | Action::Tick | Action::Render => {}
        }
    }
}
