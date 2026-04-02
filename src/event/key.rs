use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::actions::Action;

pub fn map_key_event(key: KeyEvent, search_active: bool) -> Option<Action> {
    if search_active {
        return match key.code {
            KeyCode::Esc => Some(Action::ExitSearchMode),
            KeyCode::Enter => Some(Action::ConfirmSearch),
            KeyCode::Backspace => Some(Action::SearchBackspace),
            KeyCode::Char(c) => Some(Action::SearchInput(c)),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::Quit),
        KeyCode::Esc => Some(Action::Quit),
        KeyCode::Right | KeyCode::Tab => Some(Action::NextTab),
        KeyCode::Left | KeyCode::BackTab => Some(Action::PrevTab),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::ScrollDown),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::ScrollUp),
        KeyCode::Char('n') => Some(Action::NextFile),
        KeyCode::Char('p') => Some(Action::PrevFile),
        KeyCode::Char('?') => Some(Action::ToggleHelp),
        KeyCode::Char('s') => Some(Action::CycleSortColumn),
        KeyCode::Char('S') => Some(Action::ToggleSortDirection),
        KeyCode::Char('/') => Some(Action::EnterSearchMode),
        _ => None,
    }
}
