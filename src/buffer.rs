use log::{debug, error, info, warn};

#[derive(Debug)]
pub struct TextBuffer {
    original: String,
    add: String,
    table: Vec<Span>,
    lines: Vec<usize>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BufferType {
    Original,
    Add,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    buffer: BufferType,
    start: usize,
    end: usize,
    len: usize,
}

#[derive(Debug, Clone, Copy)]
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
    pub fn new(buffer: BufferType, start: usize, len: usize) -> Span {
        Span {
            buffer,
            start,
            end: start + len,
            len,
        }
    }
}

impl TextBuffer {
    pub fn new(text: Option<&str>) -> TextBuffer {
        let mut buffer = TextBuffer {
            original: text.unwrap_or("").to_owned(),
            add: String::new(),
            table: Vec::with_capacity(500),
            lines: Vec::with_capacity(10),
        };

        buffer.table.push(Span::new(
            BufferType::Original,
            0,
            text.map_or(0, |x| x.len()),
        ));

        buffer
    }

    pub fn append(&mut self, text: &str) {
        let pos = self.add_to_buffer(text);
        self.table.push(Span::new(BufferType::Add, pos, text.len()));
    }

    pub fn prepend(&mut self, text: &str) {
        let pos = self.add_to_buffer(text);
        self.table
            .insert(0, Span::new(BufferType::Add, pos, text.len()));
    }

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

            let piece1 = Span::new(piece.span.buffer, piece.span.start, pos - piece.doc.start); //pos_in_document + pos);
            let piece2 = Span::new(BufferType::Add, pos_in_add_buffer, text.len());
            let piece3 = Span::new(
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

    pub fn delete(&mut self, start: usize, end: usize) {
        let len = end - start;

        // start and end are in the same piece:
        //     1. split the piece into two new pieces.
        // start and end are in different pieces:
        //     1. set new length for start piece.
        //     2. set new start for end piece.
        //     3. remove any pieces between these two pieces.
        let p1 = self.get_piece_at_position(start);
        let p2 = self.get_piece_at_position(end);

        match (p1, p2) {
            (Some(p1), Some(p2)) if p1.index == p2.index => {
                eprintln!("delete from single piece.");
                let start_relative = start - p1.doc.start;
                let end_relative = start + len;
                self.delete_split_piece(p1.index, start_relative, end_relative);
            }
            (Some(p1), Some(p2)) => {
                eprintln!("delete from multiple pieces.");
                self.delete_multiple(&p1, &p2, start, end);
            }
            (Some(p), None) => {}
            _ => {
                eprintln!("none");
            }
        };
    }

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
        let p1 = Span::new(ex.buffer, ex.start, start);
        let p2 = Span::new(ex.buffer, ex.start + end, ex.len - end);

        self.table[index] = p1;
        self.table.insert(index + 1, p2);
    }

    fn delete_multiple(
        &mut self,
        p1: &DocumentPiece,
        p2: &DocumentPiece,
        start: usize,
        end: usize,
    ) {
        // update the first piece.
        let p1_len_to_delete = p1.doc.end - start;
        let p1_new_end = p1.span.end - p1_len_to_delete;
        let p1_new_len = p1.span.len - p1_len_to_delete;

        self.table[p1.index].end = p1_new_end;
        self.table[p1.index].len = p1_new_len;

        // update the final piece.
        let p2_new_len = p2.doc.end - end;
        let p2_new_start = p2.span.end - p2_new_len;

        self.table[p2.index].len = p2_new_len;
        self.table[p2.index].start = p2_new_start;

        // remove and pieces between the two pieces.
        if p2.index - p1.index > 1 {
            for i in p1.index + 1..p2.index {
                debug!("Removing index {} from piece table", i);
                self.table.remove(i);
            }
        }
    }

    pub fn text(&self) -> String {
        let mut text = String::new();

        for row in &self.table {
            match row.buffer {
                BufferType::Original => {
                    let c = &self.original[row.start..(row.start + row.len)];
                    text += c;
                }
                BufferType::Add => {
                    let c = &self.add[row.start..(row.start + row.len)];
                    text += c;
                }
            }
        }

        text
    }

    pub fn get_span_contents(&self, span: &Span) -> &str {
        match span.buffer {
            BufferType::Add => &self.add[span.start..span.end],
            BufferType::Original => &self.original[span.start..span.end],
        }
    }

    pub fn get_span_contents_with_offset(&self, span: &Span, offset: usize) -> &str {
        let start_with_offset = span.start + offset;
        match span.buffer {
            BufferType::Add => &self.add[start_with_offset..span.end],
            BufferType::Original => &self.original[start_with_offset..span.end],
        }
    }

    pub fn get_line_content(&self, line: u32) -> Option<String> {
        let mut result = String::new();

        // special case if accessing the first line number
        if line == 1 {
            for span in &self.table {
                let text = self.get_span_contents(&span);

                // find the next new line character and return once it's found.
                for (pos, c) in text.chars().enumerate() {
                    if is_newline_char(c) {
                        result += &text[..pos];
                        return Some(result);
                    }
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
            let contents = self.get_span_contents(&piece);
            for (pos, c) in contents.chars().enumerate() {
                if is_newline_char(c) {
                    current_line += 1;
                    if current_line == line {
                        eprintln!("IS CURRENT LINE");
                        return Some(self.get_line_content_until_next_linebreak(index, pos));
                    }
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
            for (pos, c) in text.chars().enumerate() {
                if is_newline_char(c) {
                    result += &text[..pos];
                    return result;
                }
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
        for (i, piece) in self.table.iter().enumerate() {
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
            lines: vec![],
            table: vec![
                Span {
                    buffer: BufferType::Add,
                    start: 0,
                    len: 6,
                    end: 6,
                },
                Span {
                    buffer: BufferType::Original,
                    start: 0,
                    len: 5,
                    end: 5,
                },
                Span {
                    buffer: BufferType::Add,
                    start: 17,
                    len: 6,
                    end: 23,
                },
                Span {
                    buffer: BufferType::Original,
                    start: 5,
                    len: 9,
                    end: 14,
                },
            ],
        };

        let expected = "Lorem ipsum dolor sit amet";
        let actual = buffer.text();
        assert_eq!(expected, actual);
    }

    #[test]
    fn append() {
        let mut buffer = TextBuffer::new(Some("Lorem ipsum dolor"));
        buffer.append(" sit");
        buffer.append(" amet");

        let expected = "Lorem ipsum dolor sit amet";
        let actual = buffer.text();
        assert_eq!(expected, actual);
    }

    #[test]
    fn insert() {
        let mut buffer = TextBuffer::new(Some("This is  text"));
        buffer.insert(8, "some");

        let expected = "This is some text";
        let actual = buffer.text();
        assert_eq!(expected, actual);
    }

    #[test]
    fn delete() {
        let mut buffer = TextBuffer::new(Some("ipsum sit amet"));
        buffer.insert(0, "Lorem ");
        buffer.insert(11, "deletedtext");
        buffer.insert(11, " dolor");
        buffer.delete(17, 28);

        let expected = "Lorem ipsum dolor sit amet";
        let actual = buffer.text();

        assert_eq!(expected, actual);
    }

    #[test]
    fn get_line_contents_single() {
        let buffer = TextBuffer::new(Some(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        ));

        let expected = Some(String::from(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        ));
        let actual = buffer.get_line_content(1);
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_line_contents_multiple() {
        let mut buffer = TextBuffer::new(Some("Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nPraesent ultricies lacus ut molestie dapibus."));
        buffer.append("\nNam diam lorem, efficitur nec mauris eget, ultrices molestie mi.");
        buffer.append("\nSed varius magna quis maximus mattis.");

        let expected = Some(String::from(
            "Praesent ultricies lacus ut molestie dapibus.",
        ));
        let actual = buffer.get_line_content(2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_line_contents_multiple_spans() {
        let mut buffer = TextBuffer::new(Some(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.\n",
        ));
        buffer.append("Praesent ultricies lacus ut molestie dapibus.\nNam diam lorem, e");
        buffer.append("fficitur nec mauris eget, ultrices molestie mi.\nSed varius magna quis maximus mattis.");

        let expected = Some(String::from(
            "Nam diam lorem, efficitur nec mauris eget, ultrices molestie mi.",
        ));
        let actual = buffer.get_line_content(3);
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_line_contents_invalid() {
        let mut buffer = TextBuffer::new(Some("Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nPraesent ultricies lacus ut molestie dapibus."));
        buffer.append("\nNam diam lorem, efficitur nec mauris eget, ultrices molestie mi.");
        buffer.append("\nSed varius magna quis maximus mattis.");

        let expected = None;
        let actual = buffer.get_line_content(5);
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_line_count_empty() {
        let buffer = TextBuffer::new(None);
        assert_eq!(1, buffer.get_line_count());
    }

    #[test]
    fn get_line_count_single() {
        let buffer = TextBuffer::new(Some(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        ));
        assert_eq!(1, buffer.get_line_count());
    }

    #[test]
    fn get_line_count_multiple() {
        let mut buffer = TextBuffer::new(Some("Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nPraesent ultricies lacus ut molestie dapibus."));
        buffer.append("\nNam diam lorem, efficitur nec mauris eget, ultrices molestie mi.\nSed varius magna quis maximus mattis.");
        assert_eq!(4, buffer.get_line_count());
    }
}
