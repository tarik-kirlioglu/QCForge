use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::actions::Action;

pub fn map_key_event(key: KeyEvent) -> Option<Action> {
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
        _ => None,
    }
}
