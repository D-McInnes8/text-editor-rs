use std::io;

use self::editor::Editor;

mod buffer;
mod document;
mod editor;
mod terminal;

fn main() {
    let mut stdout = io::stdout();
    //run(&mut stdout)

    let mut editor = Editor::new();
    editor.run();
}
