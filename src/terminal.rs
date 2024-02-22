use std::io::stdout;
use std::io::Write;

use crossterm::cursor;
use crossterm::execute;
use crossterm::terminal;
use log::trace;

pub struct Terminal {}

impl Terminal {
    pub fn new() -> Terminal {
        Terminal {}
    }

    pub fn startup(&self) -> std::io::Result<()> {
        execute!(stdout(), terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()
    }

    pub fn shutdown(&self) -> std::io::Result<()> {
        stdout().flush()?;
        execute!(stdout(), terminal::LeaveAlternateScreen)
    }

    pub fn size(&self) -> TerminalSize {
        let (width, height) = terminal::size().expect("");
        TerminalSize { width, height }
    }

    pub fn cursor_pos(&self) -> CursorPosition {
        let (x, y) = cursor::position().expect("");
        CursorPosition { x, y }
    }

    pub fn move_cursor_to(&self, pos: CursorPosition) {
        execute!(stdout(), cursor::MoveTo(pos.x, pos.y)).expect("");
    }

    pub fn render(&self, frame: String) -> std::io::Result<()> {
        trace!("Rendering");

        let (x, y) = crossterm::cursor::position()?;

        // Clear the terminal
        execute!(stdout(), crossterm::cursor::Hide)?;
        execute!(stdout(), crossterm::cursor::MoveTo(0, 0))?;
        execute!(stdout(), terminal::Clear(terminal::ClearType::Purge)).unwrap();

        print!("{}", frame);
        stdout().flush()?;

        execute!(stdout(), crossterm::cursor::MoveTo(x, y))?;
        execute!(stdout(), crossterm::cursor::Show)?;

        Ok(())
    }
}

pub struct CursorPosition {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}
