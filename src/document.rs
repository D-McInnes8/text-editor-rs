use std::path::PathBuf;

use crate::buffer::TextBuffer;

pub struct Document {
    buffer: TextBuffer,
    path: Option<PathBuf>,
}

impl Document {
    pub fn new() -> Document {
        Document {
            buffer: TextBuffer::new(None),
            path: None,
        }
    }

    pub fn load(file: Option<PathBuf>) -> Document {
        Document {
            buffer: TextBuffer::new(Some("")),
            path: file,
        }
    }
}
