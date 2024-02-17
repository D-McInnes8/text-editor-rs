use log::warn;

#[derive(Debug)]
pub struct TextBuffer {
    original: Vec<char>,
    add: Vec<char>,
    table: Vec<Node>,
    lines: Vec<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct Node {
    buffer: BufferType,
    start: usize,
    length: usize,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BufferType {
    Original,
    Add,
}

impl TextBuffer {
    pub fn new(text: Option<&str>) -> TextBuffer {
        let mut buffer = TextBuffer {
            original: text.unwrap_or("").chars().collect(),
            add: Vec::with_capacity(1000),
            table: Vec::with_capacity(500),
            lines: Vec::with_capacity(10),
        };

        buffer.table.push(Node {
            buffer: BufferType::Original,
            start: 0,
            length: text.map_or(0, |x| x.len()),
        });

        buffer
    }

    pub fn append(&mut self, text: &str) {
        let pos = self.add.len();
        self.add.append(&mut text.chars().collect::<Vec<char>>());
        self.table.push(Node {
            buffer: BufferType::Add,
            start: pos,
            length: text.len(),
        });
    }

    fn add_to_buffer(&mut self, text: &str) -> usize {
        let pos = self.add.len();
        self.add.append(&mut text.chars().collect::<Vec<char>>());
        pos
    }

    pub fn insert(&mut self, pos: usize, text: &str) {
        if let Some((piece, piece_index, pos_in_document)) = &self.get_piece_at_position(pos) {
            eprintln!("pos: {}, pos in document: {}", pos, pos_in_document);
            let diff = pos - pos_in_document;

            let p1_start = piece.start;
            let p1_length = pos_in_document + pos;

            let p2_start = 0;
            let p2_length = text.len();

            let p3_start = p1_start + p1_length;
            let p3_length = piece.length - diff;

            let append_pos = self.add.len();
            let mut chars = text.chars().collect::<Vec<char>>();
            //self.add.append(&mut vec![]);
            //self.add.append(&mut text.chars().collect::<Vec<char>>());

            let append_pos = self.add_to_buffer(text);

            let p1 = Node {
                buffer: piece.buffer.to_owned(),
                start: p1_start,
                length: p1_length,
            };
            let p2 = Node {
                buffer: BufferType::Add,
                start: append_pos,
                length: p2_length,
            };
            let p3 = Node {
                buffer: piece.buffer.to_owned(),
                start: p3_start,
                length: p3_length,
            };

            self.table[*piece_index].start = p1_start;
            self.table[*piece_index].length = p1_length;
            self.table.insert(piece_index + 1, p3);
            self.table.insert(piece_index + 1, p2);
        } else {
            warn!("Position {} is too large", pos);
        }
    }

    pub fn delete(&mut self, pos: usize, length: usize) {}

    pub fn text(&self) -> String {
        let mut text = String::new();

        for row in &self.table {
            match row.buffer {
                BufferType::Original => {
                    let c = &self.original[row.start..(row.start + row.length)];
                    text += c.iter().collect::<String>().as_str();
                }
                BufferType::Add => {
                    let c = &self.add[row.start..(row.start + row.length)];
                    text += c.iter().collect::<String>().as_str();
                }
            }
        }

        text
    }

    fn get_piece_at_position(&self, pos: usize) -> Option<(Node, usize, usize)> {
        let mut current_pos = 0;

        for (i, piece) in self.table.iter().enumerate() {
            current_pos += piece.length;
            if current_pos >= pos {
                return Some((piece.clone(), i, current_pos - piece.length));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_text() {
        let buffer = TextBuffer {
            original: vec![
                'i', 'p', 's', 'u', 'm', ' ', 's', 'i', 't', ' ', 'a', 'm', 'e', 't',
            ],
            lines: vec![],
            add: vec![
                'L', 'o', 'r', 'e', 'm', ' ', 'd', 'e', 'l', 'e', 't', 'e', 'd', 't', 'e', 'x',
                't', ' ', 'd', 'o', 'l', 'o', 'r',
            ],
            table: vec![
                Node {
                    buffer: BufferType::Add,
                    start: 0,
                    length: 6,
                },
                Node {
                    buffer: BufferType::Original,
                    start: 0,
                    length: 5,
                },
                Node {
                    buffer: BufferType::Add,
                    start: 17,
                    length: 6,
                },
                Node {
                    buffer: BufferType::Original,
                    start: 5,
                    length: 9,
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
    fn delete() {}
}
