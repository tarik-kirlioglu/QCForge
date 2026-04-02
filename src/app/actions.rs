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
    CycleSortColumn,
    ToggleSortDirection,
    EnterSearchMode,
    ExitSearchMode,
    ConfirmSearch,
    SearchInput(char),
    SearchBackspace,
    ScrollLeft,
    ScrollRight,
    Resize(u16, u16),
    SplashStatus(String),
    LoadComplete(QcResults),
    Error(String),
}
