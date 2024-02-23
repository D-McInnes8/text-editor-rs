use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::editor::Event;

pub struct KeyMaps {}

impl KeyMaps {
    pub fn map_key_press_to_event(&self, event: KeyEvent) -> Option<Event> {
        match event {
            KeyEvent {
                code: KeyCode::Left,
                modifiers: _,
                kind: KeyEventKind::Press,
                state: _,
            } => Some(Event::MoveCursorLeft(1)),
            KeyEvent {
                code: KeyCode::Right,
                modifiers: _,
                kind: KeyEventKind::Press,
                state: _,
            } => Some(Event::MoveCursorRight(1)),
            KeyEvent {
                code: KeyCode::Up,
                modifiers: _,
                kind: KeyEventKind::Press,
                state: _,
            } => Some(Event::MoveCursorUp(1)),
            KeyEvent {
                code: KeyCode::Down,
                modifiers: _,
                kind: KeyEventKind::Press,
                state: _,
            } => Some(Event::MoveCursorDown(1)),
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: _,
            } if c == 'q' => Some(Event::Exit),
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: _,
            } => Some(Event::KeyPress(c)),
            _ => None,
        }
    }
}
