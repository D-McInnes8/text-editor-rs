use log::{info, warn};

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
    pos: DocumentPosition,
}

#[derive(Debug, Clone, Copy)]
pub struct DocumentPosition {
    start: u32,
    end: u32,
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
        //self.table.insert(0, Node::add(pos, text.len()));
        self.table
            .insert(0, Span::new(BufferType::Add, pos, text.len()));
    }

    fn add_to_buffer(&mut self, text: &str) -> usize {
        let pos = self.add.len();
        self.add += text;
        pos
    }

    pub fn insert(&mut self, pos: usize, text: &str) {
        info!("Inserting {} at position {}", text, pos);
        //eprintln!();
        //eprintln!("Inserting {} at position {}", text, pos);

        // position is at the start
        if pos == 0 {
            self.prepend(text);
            return;
        }

        // position is at the end
        if pos == self.doc_len() {
            self.append(text);
            return;
        }

        // position is in the middle
        if let Some((piece, piece_index, pos_in_document)) = &self.get_piece_at_position(pos) {
            let pos_in_add_buffer = self.add_to_buffer(text);

            /*eprintln!("{:?}", self);
            eprintln!(
                "piece: {:?}, piece index: {}, doc position: {}",
                piece, piece_index, pos_in_document
            );
            eprintln!(
                "p3 length: {} - {} - {}",
                piece.length, pos, pos_in_document
            );*/

            let piece1 = Span::new(piece.buffer, piece.start, pos - pos_in_document); //pos_in_document + pos);
            let piece2 = Span::new(BufferType::Add, pos_in_add_buffer, text.len());

            /*eprintln!();
            eprintln!("piece1: {:?}", piece1);
            eprintln!("piece2: {:?}", piece2);
            eprintln!("{} - ({} + {})", piece.length, piece1.start, piece1.length);
            eprintln!();*/
            let piece3 = Span::new(
                piece.buffer,
                piece1.start + piece1.len,
                piece.len - (piece1.start + piece1.len),
                //piece.length - pos - pos_in_document,
            );

            self.table[*piece_index] = piece1;
            self.table.insert(piece_index + 1, piece3);
            self.table.insert(piece_index + 1, piece2);
        } else {
            warn!("Position {} is too large", pos);
        }
    }

    pub fn delete(&mut self, start: usize, end: usize) {
        let len = end - start;
        eprintln!("start: {}, end: {}", start, end);
        eprintln!("len: {}", len);
        eprintln!("buffer: {:?}", self);

        // start and end are in the same piece:
        //     1. split the piece into two new pieces.
        // start and end are in different pieces:
        //     1. set new length for start piece.
        //     2. set new start for end piece.
        //     3. remove any pieces between these two pieces.
        let p1 = self.get_piece_at_position(start);
        let p2 = self.get_piece_at_position(end);

        eprintln!(
            "p1: {:?}. text: {}",
            p1,
            self.get_text_for_piece(p1.unwrap().1)
        );
        eprintln!(
            "p2: {:?}. text: {}",
            p2,
            self.get_text_for_piece(p2.unwrap().1)
        );

        match (p1, p2) {
            (Some((p1, i1, d1)), Some((p2, i2, d2))) if i1 == i2 => {
                eprintln!("delete from single piece.");
                let start_relative = start - d1;
                let end_relative = start + len;
                self.split_piece(i1, start_relative, end_relative);
            }
            (Some((p1, i1, d1)), Some((p2, i2, d2))) => {
                eprintln!("delete from multiple pieces.");
                self.delete_multiple(i1, i2);
            }
            (Some((a, b, c)), None) => {}
            _ => {
                eprintln!("none");
            }
        };
    }

    fn split_piece(&mut self, index: usize, start: usize, end: usize) {
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

    fn delete_multiple(&mut self, i1: usize, i2: usize) {
        let p1 = &self.table[i1];
        let p2 = &self.table[i2];

        // Remove and pieces between the two pieces.
        if i2 - i1 > 1 {
            for i in i1 + 1..i2 - 1 {
                eprintln!("index: {}", i);
                //self.table.remove(i);
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

    fn get_piece_at_position(&self, pos: usize) -> Option<(Span, usize, usize)> {
        let mut current_pos = 0;

        for (i, piece) in self.table.iter().enumerate() {
            /*eprintln!(
                "index: {}, piece: {:?}. text: '{}'",
                i,
                piece,
                self.get_text_for_piece(i)
            );*/
            //current_pos += piece.length;
            eprintln!("current pos: {}", current_pos);
            if current_pos + piece.len >= pos {
                //eprintln!("Returning piece");
                eprintln!("piece: {:?}. text: {}", piece, self.get_text_for_piece(i));
                eprintln!("{} >= {}", current_pos, pos);
                return Some((piece.clone(), i, current_pos));
            }

            current_pos += piece.len;
        }

        eprintln!(
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

    fn get_text_for_piece(&self, pos: usize) -> &str {
        let p = &self.table[pos];
        match p.buffer {
            BufferType::Add => &self.add[p.start..(p.start + p.len)],
            BufferType::Original => &self.original[p.start..(p.start + p.len)],
        }
    }
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
}
