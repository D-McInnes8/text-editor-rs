use log::{debug, error, info, warn};

#[derive(Debug)]
pub struct TextBuffer {
    original: String,
    add: String,
    table: Vec<Span>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BufferType {
    Original,
    Add,
}

#[derive(Debug, Clone)]
pub struct Span {
    buffer: BufferType,
    start: usize,
    end: usize,
    len: usize,
    lines: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct DocumentPiece {
    index: usize,
    span: Span,
    doc: DocumentSpan,
}

#[derive(Debug, Clone, Copy)]
pub struct DocumentSpan {
    start: usize,
    end: usize,
}

impl Span {
    pub fn new(buffer: BufferType, start: usize, len: usize, lines: Vec<usize>) -> Span {
        Span {
            buffer,
            start,
            end: start + len,
            len,
            lines,
        }
    }
}

impl TextBuffer {
    /// Constructs a new 'TextBuffer'.
    ///
    /// # Arguments
    /// * 'text' - An optional parameter used to load text into the original buffer
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer = TextBuffer::new(Stromg(String::from("Lorem ipsum dolor sit amet")));
    /// ```
    pub fn new(text: Option<String>) -> TextBuffer {
        if let Some(txt) = text {
            let mut buffer = TextBuffer {
                original: txt,
                add: String::new(),
                table: Vec::with_capacity(500),
            };

            buffer
                .table
                .push(buffer.create_span(BufferType::Original, 0, buffer.original.len()));
            return buffer;
        } else {
            return TextBuffer {
                original: String::new(),
                add: String::new(),
                table: Vec::with_capacity(500),
            };
        }
    }

    /// Appends a section of text to the end of the document
    ///
    /// # Arguments
    ///
    /// * 'text' - The text that will be inserted at the end of the document
    pub fn append(&mut self, text: &str) {
        let pos = self.add_to_buffer(text);
        self.table
            .push(self.create_span(BufferType::Add, pos, text.len()));
    }

    /// Prepends a section of text to the start of the document.
    ///
    /// # Arguments
    ///
    /// * 'text' - The text that will be inserted at the start of the document
    pub fn prepend(&mut self, text: &str) {
        let pos = self.add_to_buffer(text);
        self.table
            .insert(0, self.create_span(BufferType::Add, pos, text.len()));
    }

    /// Inserts a section of text into the given position in the document. If the position is at
    /// the start/end of the document, a new piece will be prepended/appended onto the table.
    ///
    /// If the position is in the middle of a piece, the piece will be split into two and a new
    /// piece inserted between them.
    ///
    /// # Arguments
    ///
    /// * 'pos' - The position in the document where the text will be insert_end_of_line
    /// * 'text' - The text that will be inserted at the speicified position
    pub fn insert(&mut self, pos: usize, text: &str) {
        info!("Inserting {} at position {}", text, pos);

        // position is at the start
        if pos == 0 {
            debug!("Prepending text to the start of the piece table");
            self.prepend(text);
            return;
        }

        // position is at the end
        if pos == self.doc_len() {
            debug!("Appending text to the end of the piece table");
            self.append(text);
            return;
        }

        // position is in the middle
        if let Some(piece) = &self.get_piece_at_position(pos) {
            debug!(
                "Splitting row {} of the piece table into multiple pieces",
                piece.index
            );
            let pos_in_add_buffer = self.add_to_buffer(text);

            let piece1 =
                self.create_span(piece.span.buffer, piece.span.start, pos - piece.doc.start); //pos_in_document + pos);
            let piece2 = self.create_span(BufferType::Add, pos_in_add_buffer, text.len());
            let piece3 = self.create_span(
                piece.span.buffer,
                piece1.start + piece1.len,
                piece.span.len - (piece1.start + piece1.len),
            );

            self.table[piece.index] = piece1;
            self.table.insert(piece.index + 1, piece3);
            self.table.insert(piece.index + 1, piece2);
        } else {
            warn!("Position {} is too large", pos);
        }
    }

    /// Deletes a section of text from the table. This function will perform the following
    /// depending on whether or not the start and end position are in the same piece:
    ///
    /// start and end are in the same piece:
    ///     1. split the piece into two new pieces.
    /// start and end are in different pieces:
    ///     1. set new length for start piece.
    ///     2. set new start for end piece.
    ///     3. remove any pieces between these two pieces.
    ///
    /// # Arguments
    ///
    /// * 'start' - The position in the document where the text to be deleted starts
    /// * 'end' - The position in the document where the text to tbe deleted ends
    pub fn delete(&mut self, start: usize, end: usize) {
        let len = end - start;
        let p1 = self.get_piece_at_position(start);
        let p2 = self.get_piece_at_position(end);

        match (p1, p2) {
            (Some(p1), Some(p2)) if p1.index == p2.index => {
                let start_relative = start - p1.doc.start;
                let end_relative = start + len;
                self.delete_split_piece(p1.index, start_relative, end_relative);
            }
            (Some(p1), Some(p2)) => {
                self.delete_multiple(&p1, &p2, start, end);
            }
            (Some(p), None) => {}
            _ => {
                eprintln!("none");
            }
        };
    }

    /// Deletes a section of text when it only resides on in a single piece.
    /// Will split the piece into two new pieces.
    ///
    /// # Arguments
    ///
    /// * 'index' - The index of the piece to split in the piece table
    /// * 'start' - The position within the span that the text to be deleted starts, relative to
    /// the start of the span.
    /// * 'end' - The position with the span that the text to be deleted ends, relative to the
    /// start of the span.
    fn delete_split_piece(&mut self, index: usize, start: usize, end: usize) {
        // buffer   start length
        // original 0     22
        //
        // delete 15-20
        //
        // buffer   start length func
        // original 0     15     (ex.start) (start)
        // original 20    22     (ex.start + end) (ex.length - end)
        let ex = &self.table[index];
        let p1 = self.create_span(ex.buffer, ex.start, start);
        let p2 = self.create_span(ex.buffer, ex.start + end, ex.len - end);

        self.table[index] = p1;
        self.table.insert(index + 1, p2);
    }

    /// Deletes a section of text from the piece table when it resides over multiple pieces.
    /// Will modify the start/end of the first/last piece and delete any pieces between them.
    ///
    /// # Arguments
    ///
    /// * 'p1' - The piece where the start of the text to be deleted is located
    /// * 'p2' - The piece where the end of the text to be deleted is located
    /// * 'start' - The position in the document where the text to be deleted starts
    /// * 'end' - The position in the document where the text to be deleted ends
    fn delete_multiple(
        &mut self,
        p1: &DocumentPiece,
        p2: &DocumentPiece,
        start: usize,
        end: usize,
    ) {
        // update the first piece.
        let p1_len_to_delete = p1.doc.end - start;
        let p1_new_len = p1.span.len - p1_len_to_delete;

        self.table[p1.index] = self.create_span(p1.span.buffer, p1.span.start, p1_new_len);

        // update the final piece.
        let p2_new_len = p2.doc.end - end;
        let p2_new_start = p2.span.end - p2_new_len;

        self.table[p2.index] = self.create_span(p2.span.buffer, p2_new_start, p2_new_len);

        // remove and pieces between the two pieces.
        if p2.index - p1.index > 1 {
            for i in p1.index + 1..p2.index {
                debug!("Removing index {} from piece table", i);
                self.table.remove(i);
            }
        }
    }

    /// Constructs the document stored in the piece table. If the table is empty it will return an
    /// empty string. Note that this is an expensive operation, especially for large documents.
    pub fn text(&self) -> String {
        let mut text = String::new();

        for row in &self.table {
            text += self.get_span_contents(row);
        }

        text
    }

    /// Generates the text for a single span in the piece table.
    ///
    /// # Arguments
    ///
    /// * 'span' - The span to generate text for
    pub fn get_span_contents(&self, span: &Span) -> &str {
        assert!(span.start <= span.end, "Attempting to get the contents for a span with a start index ({}) greater than it's end index ({}).", span.start, span.end);

        let buffer = match span.buffer {
            BufferType::Add => &self.add,
            BufferType::Original => &self.original,
        };

        assert!(span.start <= buffer.len(), "Out of bounds index for {:?} buffer. Attempting to access index {} on a buffer of size {}", span.buffer,span.start, buffer.len());
        assert!(span.end <= buffer.len(), "Out of bounds index for {:?} buffer. Attempting to access index {} on a buffer of size {}", span.buffer, span.end, buffer.len());

        &buffer[span.start..span.end]
    }

    /// Generates the text for a single span in the piece table with an initial offset.
    ///
    /// # Arguments
    ///
    /// * 'span' - The span to generate text for
    /// * 'offset' - Will offset the span by this amount. Is relative to the start of the span
    pub fn get_span_contents_with_offset(&self, span: &Span, offset: usize) -> &str {
        assert!(span.start <= span.end, "Attempting to get the contents for a span with a start index ({}) greater than it's end index ({}).", span.start, span.end);

        let start_with_offset = span.start + offset;
        match span.buffer {
            BufferType::Add => &self.add[start_with_offset..span.end],
            BufferType::Original => &self.original[start_with_offset..span.end],
        }
    }

    pub fn get_buffer_contents(&self, buffer_type: BufferType, start: usize, end: usize) -> &str {
        assert!(start <= end, "Attempting to get the contents for a span with a start index ({}) greater than it's end index ({}).", start, end);

        let buffer = match buffer_type {
            BufferType::Add => &self.add,
            BufferType::Original => &self.original,
        };

        assert!(start <= buffer.len(), "Out of bounds index for {:?} buffer. Attempting to access index {} on a buffer of size {}", buffer_type, start, buffer.len());
        assert!(end <= buffer.len(), "Out of bounds index for {:?} buffer. Attempting to access index {} on a buffer of size {}", buffer_type, end, buffer.len());

        &buffer[start..end]
    }

    /// Generates the text for a line within the document. Does not include new line characters in
    /// the result. Line numbers start from 1, so requesting line 0 will always return a None result.
    ///
    /// # Arguments
    ///
    /// * 'line' - The line number to generate the text for.
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer = TextBuffer::new(Some(String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nPraesent ultricies lacus ut molestie dapibus.")));
    /// let content = buffer.get_line_content(2);
    /// assert_eq!(Some(String::from("Praesent ultricies lacus ut molestie dapibus.")), content);
    /// ```
    pub fn get_line_content(&self, line: u32) -> Option<String> {
        if self.table.is_empty() {
            return None;
        }

        let mut result = String::new();

        // special case if accessing the first line number
        if line == 1 {
            for span in &self.table {
                let text = self.get_span_contents(&span);

                // find the next new line character and return once it's found.
                for pos in &span.lines {
                    result += &text[..*pos];
                    return Some(result);
                }

                // no new line characters in this piece, so add the entire piece to the result.
                result += text;
            }

            // already on the last line, so just return the entire result.
            return Some(result);
        }

        // main case where line number != 1
        let mut current_line = 1;
        let mut index = 0;

        for piece in &self.table {
            for pos in &piece.lines {
                current_line += 1;
                if current_line == line {
                    return Some(self.get_line_content_until_next_linebreak(index, *pos));
                }
            }

            index += 1;
        }

        None
    }

    fn get_line_content_until_next_linebreak(&self, index: usize, offset: usize) -> String {
        let mut result = String::new();
        let mut i = index;

        while i < self.table.len() {
            let span = &self.table[i];
            let text = if i == index {
                self.get_span_contents_with_offset(&span, offset + 1)
            } else {
                self.get_span_contents(&span)
            };

            // find the next new line character and return once it's found.
            for pos in &span.lines {
                if i == index && *pos <= offset {
                    continue;
                }

                let end_pos = if i == index { *pos - offset - 1 } else { *pos };

                result += &text[..end_pos];
                return result;
            }

            // no new line characters in this piece. If it's the origina span, calculate the
            // offset, otherwise add the entire piece to the result and continue to the next piece.
            result += text;
            i += 1;
        }

        // already on the last line, so just return the entire result.
        result
    }

    pub fn get_line_count(&self) -> u32 {
        let mut count = 1;

        for span in &self.table {
            let text = self.get_span_contents(&span);
            for c in text.chars() {
                if is_newline_char(c) {
                    count += 1;
                }
            }
        }

        count
    }

    fn add_to_buffer(&mut self, text: &str) -> usize {
        let pos = self.add.len();
        self.add += text;
        pos
    }

    fn create_span(&self, buffer: BufferType, start: usize, len: usize) -> Span {
        let end = start + len;
        assert!(start <= end, "Attempting to create a span for the {:?} buffer with a start index ({}) greater than it's end index ({}).", buffer, start, end);
        debug_assert!(len != 0, "Attempting to create a span with 0 length.");

        // Cache new line character positions so we don't have to iterate over the text each time
        // we want to get line numbers.
        let mut lines = vec![];
        let contents = self.get_buffer_contents(buffer, start, end);
        for (pos, c) in contents.chars().enumerate() {
            if is_newline_char(c) {
                lines.push(pos);
            }
        }

        Span::new(buffer, start, len, lines)
    }

    fn get_piece_at_position(&self, pos: usize) -> Option<DocumentPiece> {
        let mut current_pos = 0;

        for (i, piece) in self.table.iter().enumerate() {
            if current_pos + piece.len >= pos {
                return Some(DocumentPiece {
                    index: i,
                    span: piece.clone(),
                    doc: DocumentSpan {
                        start: current_pos,
                        end: current_pos + piece.len,
                    },
                });
            }

            current_pos += piece.len;
        }

        error!(
            "Invalid position. Pos: {}, Current pos: {}",
            pos, current_pos
        );
        None
    }

    fn doc_len(&self) -> usize {
        let mut current_pos = 0;
        for (_, piece) in self.table.iter().enumerate() {
            current_pos += piece.len;
        }
        current_pos
    }
}

#[inline]
fn is_newline_char(c: char) -> bool {
    c == 0xA as char
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_text() {
        let buffer = TextBuffer {
            original: String::from("ipsum sit amet"),
            add: String::from("Lorem deletedtext dolor"),
            table: vec![
                Span {
                    buffer: BufferType::Add,
                    start: 0,
                    len: 6,
                    end: 6,
                    lines: vec![],
                },
                Span {
                    buffer: BufferType::Original,
                    start: 0,
                    len: 5,
                    end: 5,
                    lines: vec![],
                },
                Span {
                    buffer: BufferType::Add,
                    start: 17,
                    len: 6,
                    end: 23,
                    lines: vec![],
                },
                Span {
                    buffer: BufferType::Original,
                    start: 5,
                    len: 9,
                    end: 14,
                    lines: vec![],
                },
            ],
        };

        let expected = "Lorem ipsum dolor sit amet";
        let actual = buffer.text();
        assert_eq!(expected, actual);
    }

    #[test]
    fn insert_start_of_line() {
        let mut buffer = TextBuffer::new(Some(String::from("dolor sit amet")));
        buffer.insert(0, "ipsum ");
        buffer.prepend("Lorem ");

        let expected = "Lorem ipsum dolor sit amet";
        let actual = buffer.text();
        assert_eq!(expected, actual);
    }

    #[test]
    fn insert_end_of_line() {
        let mut buffer = TextBuffer::new(Some(String::from("Lorem ipsum dolor")));
        buffer.insert(17, " sit");
        buffer.append(" amet");

        let expected = "Lorem ipsum dolor sit amet";
        let actual = buffer.text();
        assert_eq!(expected, actual);
    }

    #[test]
    fn insert_middle_of_line() {
        let mut buffer = TextBuffer::new(Some(String::from("Lorem ipsum  sit amet")));
        buffer.insert(12, "dolor");

        let expected = "Lorem ipsum dolor sit amet";
        let actual = buffer.text();
        assert_eq!(expected, actual);
    }

    #[test]
    fn delete_start_of_line() {
        let mut buffer = TextBuffer::new(Some(String::from("Lorem ipsum dolor sit amet")));
        buffer.delete(0, 6);

        let expected = "ipsum dolor sit amet";
        let actual = buffer.text();
        assert_eq!(expected, actual);
    }

    #[test]
    fn delete_end_of_line() {
        let mut buffer = TextBuffer::new(Some(String::from("Lorem ipsum dolor sit amet")));
        buffer.delete(21, 26);

        let expected = "Lorem ipsum dolor sit";
        let actual = buffer.text();
        assert_eq!(expected, actual);
    }

    #[test]
    fn delete_middle_of_line() {
        let mut buffer = TextBuffer::new(Some(String::from("Lorem ipsum dolor sit amet")));
        buffer.delete(9, 19);

        let expected = "Lorem ipsit amet";
        let actual = buffer.text();
        assert_eq!(expected, actual);
    }

    #[test]
    fn delete_end_out_of_bounds() {
        let mut buffer = TextBuffer::new(Some(String::from("Lorem ipsum dolor sit amet")));
        buffer.delete(21, 29);

        let expected = "Lorem ipsum dolor sit";
        let actual = buffer.text();
        assert_eq!(expected, actual);
    }

    #[test]
    fn delete_start_and_end_out_of_bounds() {
        let mut buffer = TextBuffer::new(Some(String::from("Lorem ipsum dolor sit amet")));
        buffer.delete(28, 31);

        let expected = "Lorem ipsum dolor sit amet";
        let actual = buffer.text();
        assert_eq!(expected, actual);
    }

    #[test]
    fn insert_and_delete() {
        let mut buffer = TextBuffer::new(Some(String::from("ipsum sit amet")));
        buffer.insert(0, "Lorem ");
        buffer.insert(11, "deletedtext");
        buffer.insert(11, " dolor");
        buffer.delete(17, 28);

        let expected = "Lorem ipsum dolor sit amet";
        let actual = buffer.text();

        assert_eq!(expected, actual);
    }

    #[test]
    fn get_line_contents_empty() {
        let buffer = TextBuffer::new(None);
        let actual = buffer.get_line_content(1);
        assert_eq!(None, actual);
    }

    #[test]
    fn get_line_contents_single() {
        let buffer = TextBuffer::new(Some(String::from(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        )));

        let expected = Some(String::from(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        ));
        let actual = buffer.get_line_content(1);
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_line_contents_first_line() {
        let mut buffer = TextBuffer::new(Some(String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nPraesent ultricies lacus ut molestie dapibus.")));
        buffer.append("\nNam diam lorem, efficitur nec mauris eget, ultrices molestie mi.");
        buffer.append("\nSed varius magna quis maximus mattis.");

        let expected = Some(String::from(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        ));
        let actual = buffer.get_line_content(1);
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_line_contents_last_line() {
        let mut buffer = TextBuffer::new(Some(String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nPraesent ultricies lacus ut molestie dapibus.")));
        buffer.append("\nNam diam lorem, efficitur nec mauris eget, ultrices molestie mi.");
        buffer.append("\nSed varius magna quis maximus mattis.");

        let expected = Some(String::from("Sed varius magna quis maximus mattis."));
        let actual = buffer.get_line_content(4);
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_line_contents_newline_at_start_of_line() {
        let mut buffer = TextBuffer::new(Some(String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nPraesent ultricies lacus ut molestie dapibus.")));
        buffer.append("\nNam diam lorem, efficitur nec mauris eget, ultrices molestie mi.");
        buffer.append("\nSed varius magna quis maximus mattis.");

        let expected = Some(String::from(
            "Praesent ultricies lacus ut molestie dapibus.",
        ));
        let actual = buffer.get_line_content(2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_line_contents_newline_at_end_of_line() {
        let mut buffer = TextBuffer::new(Some(String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nPraesent ultricies lacus ut molestie dapibus.\n")));
        buffer.append("Nam diam lorem, efficitur nec mauris eget, ultrices molestie mi.\n");
        buffer.append("Sed varius magna quis maximus mattis.");

        let expected = Some(String::from(
            "Nam diam lorem, efficitur nec mauris eget, ultrices molestie mi.",
        ));
        let actual = buffer.get_line_content(3);
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_line_contents_newline_in_middle_of_line() {
        let mut buffer = TextBuffer::new(Some(String::from(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.\n",
        )));
        buffer.append("Praesent ultricies lacus ut molestie dapibus.\nNam diam lorem, e");
        buffer.append("fficitur nec mauris eget, ultrices molestie mi.\nSed varius magna quis maximus mattis.");

        let expected = Some(String::from(
            "Nam diam lorem, efficitur nec mauris eget, ultrices molestie mi.",
        ));
        eprintln!("{:?}", &buffer.table);
        let actual = buffer.get_line_content(3);
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_line_contents_invalid() {
        let mut buffer = TextBuffer::new(Some(String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nPraesent ultricies lacus ut molestie dapibus.")));
        buffer.append("\nNam diam lorem, efficitur nec mauris eget, ultrices molestie mi.");
        buffer.append("\nSed varius magna quis maximus mattis.");

        let expected = None;
        let actual = buffer.get_line_content(5);
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_line_content_large_document() {
        let ipsum_path = concat!(env!("CARGO_MANIFEST_DIR"), "/benches/ipsum");
        let text = std::fs::read_to_string(ipsum_path).expect("Unable to find file.");
        let buffer = TextBuffer::new(Some(text));

        let expected = Some(String::from("Nullam mollis orci et mi gravida semper."));
        let actual = buffer.get_line_content(50000);
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_line_count_empty() {
        let buffer = TextBuffer::new(None);
        assert_eq!(1, buffer.get_line_count());
    }

    #[test]
    fn get_line_count_single() {
        let buffer = TextBuffer::new(Some(String::from(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        )));

        assert_eq!(1, buffer.get_line_count());
    }

    #[test]
    fn get_line_count_multiple() {
        let mut buffer = TextBuffer::new(Some(String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nPraesent ultricies lacus ut molestie dapibus.")));
        buffer.append("\nNam diam lorem, efficitur nec mauris eget, ultrices molestie mi.\nSed varius magna quis maximus mattis.");
        assert_eq!(4, buffer.get_line_count());
    }

    #[test]
    fn cache_line_numbers_no_new_line_characters() {
        let mut buffer = TextBuffer::new(None);
        buffer.append("Lorem ipsum dolor sit amet, consectetur adipiscing elit.");

        let expected: &Vec<usize> = &vec![];
        let actual = &buffer.table.first().expect("Piece table is empty").lines;
        assert_eq!(expected, actual);
    }

    #[test]
    fn cache_line_numbers_multiple_new_line_characters() {
        let mut buffer = TextBuffer::new(None);
        buffer.append("Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nPraesent ultricies lacus ut molestie dapibus.");

        let expected = &vec![56];
        let actual = &buffer.table.first().expect("Piece table is empty").lines;
        assert_eq!(expected, actual);
    }
}
