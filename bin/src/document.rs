use std::error::Error;
use std::ffi::OsStr;
use std::ops::Range;
use std::path::PathBuf;

use log::info;
use text_buffer::TextBuffer;

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

    pub fn load(file: PathBuf) -> Result<Document, Box<dyn Error>> {
        let contents = std::fs::read_to_string(&file)?;
        info!(
            "Loaded {} characters from document {:?}",
            contents.len(),
            file
        );

        Ok(Document {
            buffer: TextBuffer::new(Some(contents)),
            path: Some(file),
        })
    }

    pub fn file_name(&self) -> Option<&OsStr> {
        if let Some(path) = &self.path {
            return path.file_name();
        }
        None
    }

    pub fn get_lines(&self, lines: Range<u32>) -> Vec<String> {
        let mut results = vec![];
        info!("Fetching lines from document with range {:?}", lines);

        for line in lines {
            if let Some(content) = self.buffer.get_line_content(line) {
                results.push(content);
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use test_case::test_case;

    use super::*;

    fn setup(file: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join(file)
    }

    fn read_lines(filename: &PathBuf) -> Vec<String> {
        std::fs::read_to_string(filename)
            .unwrap()
            .lines()
            .map(String::from)
            .collect()
    }

    #[test]
    fn load_empty_file() {
        let path = setup("empty_file");
        let document = Document::load(path).unwrap();

        let expected: Vec<String> = vec![];
        let actual = document.get_lines(Range { start: 1, end: 10 });
        assert_eq!(expected, actual);
    }

    #[test]
    fn load_file_with_single_line() {
        let path = setup("single_line");
        let document = Document::load(path).unwrap();

        let expected = vec![String::from(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        )];
        let actual = document.get_lines(Range { start: 1, end: 10 });
        assert_eq!(expected, actual);
    }

    #[test]
    fn load_file_with_single_paragraph() {
        let path = setup("single_paragraph");
        let document = Document::load(path).unwrap();

        let expected = read_lines(&setup("single_paragraph"));
        let actual = document.get_lines(Range { start: 1, end: 10 });
        assert_eq!(expected, actual);
    }

    #[test]
    fn load_file_with_multiple_paragraphs() {
        let path = setup("multiple_paragraphs");
        let document = Document::load(path.to_owned()).unwrap();

        let expected = read_lines(&path);
        let actual = document.get_lines(Range { start: 1, end: 10 });
        assert_eq!(expected, actual);
    }

    #[test_case(1, 2;   "single_line_at_start")]
    #[test_case(24, 25; "single_line_at_end")]
    #[test_case(10, 11; "single_line_in_middle")]
    #[test_case(1, 20;  "block_at_start")]
    #[test_case(15, 25; "block_at_end")]
    #[test_case(10, 20; "block_in_middle")]
    #[test_case(1, 25;  "entire_document")]
    fn load_document_and_read_lines(start: usize, end: usize) {
        let path = setup("document");
        let document = Document::load(path.to_owned()).unwrap();

        let expected = &read_lines(&path)[start - 1..end - 1];
        let actual = document.get_lines(Range {
            start: start as u32,
            end: end as u32,
        });
        assert_eq!(expected, actual);
    }
}
