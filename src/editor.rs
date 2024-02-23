use console::style;
use crossterm::cursor;
use crossterm::event;
use crossterm::event::Event as TerminalEvent;
use crossterm::terminal;
use std::error::Error;
use std::io;
use std::path::PathBuf;
use unicode_width::UnicodeWidthStr;

use crate::document::Document;
use crate::keymaps::KeyMaps;
use crate::terminal::CursorPosition;
use crate::terminal::Terminal;

pub struct Editor {
    document: Option<Document>,
    exit: bool,
    keymaps: KeyMaps,
    status: String,
    terminal: Terminal,
}

pub enum Event {
    KeyPress(char),
    Exit,
    MoveCursor(CursorPosition),
    MoveCursorUp(u16),
    MoveCursorDown(u16),
    MoveCursorLeft(u16),
    MoveCursorRight(u16),
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            document: None,
            exit: false,
            keymaps: KeyMaps {},
            status: String::from("Document"),
            terminal: Terminal::new(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        /*queue!(
            stdout(),
            style::ResetColor,
            terminal::Clear(ClearType::All),
            style::Print(format!("This is some text!"))
        )?;*/

        if self.document.is_none() {
            self.document = Some(Document::new());
        }
        self.terminal.startup()?;

        while !self.exit {
            //self.read_key_press()?;
            self.handle_event()?;
        }

        self.terminal.shutdown()?;

        Ok(())
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn load(&mut self, file: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
        if let Some(path) = file {
            let document = Document::load(path)?;
            self.document = Some(document);
        }
        Ok(())
    }

    fn handle_event(&mut self) -> std::io::Result<()> {
        let a = match event::read()? {
            TerminalEvent::FocusGained => None,
            TerminalEvent::FocusLost => None,
            TerminalEvent::Key(e) => self.keymaps.map_key_press_to_event(e),
            TerminalEvent::Mouse(_) => None,
            TerminalEvent::Paste(_) => None,
            TerminalEvent::Resize(_, _) => None,
        };

        if let Some(event) = a {
            self.process_event(event)?;
            self.render()?;
        }
        Ok(())
    }

    fn process_event(&mut self, event: Event) -> std::io::Result<()> {
        match event {
            Event::KeyPress(c) => self.handle_key_press(c)?,
            Event::Exit => self.exit(),
            Event::MoveCursor(pos) => self.terminal.move_cursor_to(pos),
            Event::MoveCursorUp(u) => self.terminal.move_cursor_up(u)?,
            Event::MoveCursorDown(u) => {
                let size = self.terminal.size();
                if cursor::position()?.1 < size.height - 2 {
                    self.terminal.move_cursor_down(u)?;
                }
            }
            Event::MoveCursorLeft(u) => self.terminal.move_cursor_left(u)?,
            Event::MoveCursorRight(u) => self.terminal.move_cursor_right(u)?,
        };
        Ok(())
    }

    fn handle_key_press(&mut self, c: char) -> std::io::Result<()> {
        print!("{}", c);
        Ok(())
    }

    fn render_status_line(&self) -> String {
        // Cursor position
        let (x, y) = cursor::position().expect("");
        let pos = format!("{}, {}", x + 1, y + 1);

        let (width, _) = terminal::size().expect("");
        let space_length = width as usize - self.status.len() - pos.len();
        let spaces = std::iter::repeat(' ')
            .take(space_length)
            .collect::<String>();

        format!("{}{}{}", style(&self.status).bold().green(), spaces, pos)
    }

    pub fn render(&self) -> std::io::Result<()> {
        let size = self.terminal.size();

        let mut buffer = String::new();
        if let Some(document) = &self.document {
            let lines = document.get_lines(std::ops::Range {
                start: 1,
                end: (size.height - 1) as u32,
            });

            for row in 0..size.height {
                if row == size.height - 1 {
                    buffer += self.render_status_line().as_str();
                } else {
                    /*let line = lines[row as usize].as_str();
                    info!(
                        "Unicode Width: {}, Normal Width: {}",
                        UnicodeWidthStr::width_cjk(line),
                        line.len()
                    );*/
                    if (row as usize) < lines.len() {
                        if lines[row as usize].len() > size.width as usize {
                            buffer += &lines[row as usize][0..size.width as usize];
                        } else {
                            buffer += &lines[row as usize];
                        }
                    }
                    buffer += "\r\n";
                    //buffer += "\n";
                }
            }
        }

        self.terminal.render(buffer)
    }
}
