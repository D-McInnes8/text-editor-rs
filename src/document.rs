use crate::buffer::TextBuffer;

pub struct Document {
    buffer: TextBuffer,
    path: Option<String>,
}

impl Document {
    pub fn new() -> Document {
        Document {
            buffer: TextBuffer::new(None),
            path: None,
        }
    }

    pub fn load(file: Option<&str>) -> Document {
        Document {
            buffer: TextBuffer::new(file),
            path: file.map(|x| x.to_owned()),
        }
    }
}
