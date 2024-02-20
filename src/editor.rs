use console::style;
use crossterm::cursor;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::execute;
use crossterm::terminal;
use std::io;
use std::io::stdout;
use std::path::PathBuf;

use crate::document::Document;
use crate::terminal::Terminal;

pub struct Editor {
    document: Option<Document>,
    exit: bool,
    status: String,
    terminal: Terminal,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            document: None,
            exit: false,
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

        self.document = Some(Document::new());
        self.terminal.startup()?;

        while !self.exit {
            self.read_key_press()?;
        }

        self.terminal.shutdown()?;

        Ok(())
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn load(&mut self, file: Option<PathBuf>) {
        if let Some(path) = file {
            self.document = Some(Document::load(Some(path)));
        }
    }

    fn read_key_press(&mut self) -> std::io::Result<()> {
        match event::read()? {
            Event::FocusGained => {}
            Event::FocusLost => {}
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: _,
                kind: KeyEventKind::Press,
                state: _,
            }) => {
                execute!(stdout(), cursor::MoveLeft(1))?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: _,
                kind: KeyEventKind::Press,
                state: _,
            }) => {
                execute!(stdout(), cursor::MoveRight(1))?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: _,
                kind: KeyEventKind::Press,
                state: _,
            }) => {
                let size = self.terminal.size();
                if cursor::position()?.1 < size.height - 2 {
                    execute!(stdout(), cursor::MoveDown(1))?;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: _,
                kind: KeyEventKind::Press,
                state: _,
            }) => {
                execute!(stdout(), cursor::MoveUp(1))?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                kind: KeyEventKind::Press,
                modifiers: _,
                state: _,
            }) => {
                self.handle_key_press(c)?;
            }
            /*Event::Key(c) if c.kind == KeyEventKind::Press => {

            }*/
            Event::Key(c) => {}
            Event::Mouse(b) => {}
            Event::Paste(a) => {}
            Event::Resize(x, y) => {}
        };
        self.render()?;
        Ok(())
    }

    fn handle_key_press(&mut self, c: char) -> std::io::Result<()> {
        if c == 'q' {
            self.exit();
        }
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
        for row in 0..size.height {
            if row == size.height - 1 {
                buffer += self.render_status_line().as_str();
            } else {
                buffer += "\n";
            }
        }

        self.terminal.render(buffer)
    }
}
