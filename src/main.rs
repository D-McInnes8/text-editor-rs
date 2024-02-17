use std::fs::File;

use structured_logger::json::new_writer;
use structured_logger::Builder;

use self::editor::Editor;

mod buffer;
mod document;
mod editor;
mod terminal;

fn main() {
    // Initialize the logger.
    let log_file = File::options()
        .create(true)
        .append(true)
        .open("app.log")
        .unwrap();

    Builder::new()
        .with_target_writer("*", new_writer(log_file))
        .init();

    //let mut stdout = io::stdout();
    //run(&mut stdout)

    let mut editor = Editor::new();
    editor.run();
}
