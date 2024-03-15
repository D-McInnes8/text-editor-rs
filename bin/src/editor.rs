use console::style;
use crossterm::cursor;
use crossterm::event;
use crossterm::event::Event as TerminalEvent;
use crossterm::terminal;
use std::error::Error;
use std::io;
use std::ops::Range;
use std::path::PathBuf;
use unicode_width::UnicodeWidthStr;

use crate::document::Document;
use crate::keymaps::KeyMaps;
use crate::terminal::CursorPosition;
use crate::terminal::Terminal;

pub struct Editor {
    column: u16,
    row: u32,
    document: Option<Document>,
    exit: bool,
    keymaps: KeyMaps,
    lines: Vec<String>,
    should_render: bool,
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
    NewLine,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            column: 0,
            row: 1,
            document: None,
            exit: false,
            keymaps: KeyMaps {},
            lines: vec![],
            should_render: true,
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

            let size = self.terminal.size();
            self.lines = document.get_lines(std::ops::Range {
                start: 1,
                end: (size.height) as u32,
            });

            self.document = Some(document);
            self.terminal.move_cursor_to(CursorPosition { x: 0, y: 0 });

            self.render()?;
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
            if self.should_render {
                self.should_render = false;
                self.render()?;
            }
        }
        Ok(())
    }

    fn process_event(&mut self, event: Event) -> std::io::Result<()> {
        self.should_render = true;
        match event {
            Event::KeyPress(c) => self.handle_key_press(c)?,
            Event::Exit => self.exit(),
            Event::MoveCursor(pos) => self.terminal.move_cursor_to(pos),
            Event::MoveCursorUp(o) => self.move_cursor_up(o)?,
            Event::MoveCursorDown(o) => self.move_cursor_down(o)?,
            Event::MoveCursorLeft(o) => self.move_cursor_left(o)?,
            Event::MoveCursorRight(o) => self.move_cursor_right(o)?,
            Event::NewLine => self.handle_new_line(),
        };
        Ok(())
    }

    fn handle_key_press(&mut self, c: char) -> std::io::Result<()> {
        //print!("{}", c);
        if let Some(document) = self.document.as_mut() {
            document.insert(self.row, self.column as u32, c);
            self.should_render = true;
        }
        Ok(())
    }

    fn move_cursor_up(&mut self, offset: u16) -> std::io::Result<()> {
        let pos = self.terminal.cursor_pos();

        if pos.y > 0 {
            self.terminal.move_cursor_up(offset)?;
            self.check_cursor_pos()?;
            self.row -= 1;
        } else {
            if self.row != 1 {
                self.row -= 1;
                if let Some(document) = &self.document {
                    let size = self.terminal.size();
                    self.lines = document.get_lines(Range {
                        start: self.row,
                        end: self.row + size.height as u32,
                    });
                    self.check_cursor_pos()?;
                }
            }
        }
        Ok(())
    }

    fn move_cursor_down(&mut self, offset: u16) -> std::io::Result<()> {
        let size = self.terminal.size();
        let pos = self.terminal.cursor_pos();
        if pos.y < size.height - 2 {
            if pos.y as usize <= self.lines.len() - 1 {
                self.row += 1;
                self.terminal.move_cursor_down(offset)?;
                self.check_cursor_pos()?;
            }
        } else {
            if let Some(document) = &self.document {
                let line_count = document.line_count();
                let size = self.terminal.size();

                if self.row < line_count {
                    self.row += 1;
                    self.lines = document.get_lines(Range {
                        start: self.row - size.height as u32,
                        end: self.row,
                    });
                    self.check_cursor_pos()?;
                }
            }
        }
        Ok(())
    }

    fn move_cursor_left(&mut self, offset: u16) -> std::io::Result<()> {
        self.terminal.move_cursor_left(offset)?;
        self.column = self.terminal.cursor_pos().x;
        Ok(())
    }

    fn move_cursor_right(&mut self, offset: u16) -> std::io::Result<()> {
        let pos = self.terminal.cursor_pos();

        if (pos.x as usize) < (self.lines[pos.y as usize].len()) {
            self.terminal.move_cursor_right(offset)?;
            self.column = self.terminal.cursor_pos().x;
        }
        Ok(())
    }

    fn check_cursor_pos(&mut self) -> std::io::Result<()> {
        let pos = self.terminal.cursor_pos();

        let y_index = pos.y as usize;
        if pos.x != self.column && self.column as usize <= (self.lines[y_index].len()) {
            self.terminal.move_cursor_to(CursorPosition {
                x: self.column,
                y: pos.y,
            });
        }
        if self.column as usize > (self.lines[y_index].len()) {
            self.terminal.move_cursor_to(CursorPosition {
                x: self.lines[pos.y as usize].len() as u16,
                y: pos.y,
            });
        }
        Ok(())
    }

    fn handle_new_line(&mut self) {
        if self.terminal.cursor_pos().y < self.terminal.size().height - 2 {
            self.terminal.move_cursor_to(CursorPosition {
                x: 0,
                y: self.terminal.cursor_pos().y + 1,
            });
        }
    }

    fn render_status_line(&self) -> String {
        // Cursor position
        let (x, y) = cursor::position().expect("");
        let pos = format!("{}, {}", x + 1, self.row);

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
                if (row as usize) < self.lines.len() {
                    if self.lines[row as usize].len() > size.width as usize {
                        buffer += &self.lines[row as usize][0..size.width as usize];
                    } else {
                        buffer += &self.lines[row as usize];
                    }
                }
                buffer += "\r\n";
            }
        }

        self.terminal.render(buffer)
    }
}
