use crate::parser::types::QcResults;

#[derive(Debug)]
pub enum Action {
    Tick,
    Render,
    Quit,
    NextTab,
    PrevTab,
    ScrollUp,
    ScrollDown,
    NextFile,
    PrevFile,
    ToggleHelp,
    Resize(u16, u16),
    LoadComplete(QcResults),
    Error(String),
}
