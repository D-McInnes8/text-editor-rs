use std::ffi::OsString;
use std::fs::File;
use std::path::Path;

use clap::Parser;
use structured_logger::json::new_writer;
use structured_logger::Builder;

use self::editor::Editor;

mod document;
mod editor;
mod keymaps;
mod terminal;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(name = "Document")]
    doc: Option<OsString>,
}

fn main() {
    let args = Args::parse();
    let file = args.doc.and_then(|file| {
        std::env::current_dir().map_or(None, |dir| Some(Path::new(&dir).join(file)))
    });

    // Initialize the logger.
    let log_file = File::options()
        .create(true)
        .append(true)
        .open("app.log")
        .unwrap();

    Builder::new()
        .with_target_writer("*", new_writer(log_file))
        .init();

    /*panic::set_hook(Box::new(|e| {
        if Terminal::exit().is_ok() {
            error!("{}", e);
            eprintln!("{}", e);
        }
    }));*/

    //let mut stdout = io::stdout();
    //run(&mut stdout)

    let mut editor = Editor::new();
    editor.load(file);
    editor.run();
}
