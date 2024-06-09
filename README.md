# Text Editor

A simple terminal text editor written in Rust.

# Crates

## text-buffer

An implementation of a [Piece table](https://en.wikipedia.org/wiki/Piece_table) written in Rust, a data structure used for representing text documents.

```rust
let mut buffer = TextBuffer::new(Some(String::from("Lorem ipsum dolor")));
buffer.append("amet");
buffer.insert(17, " sit ");

let document = buffer.text();
```

Fetching individual lines is also supported:

```rust
let mut buffer = TextBuffer::new(Some(String::from("Line 1.")));
buffer.append("\nLine 2.");

let line = buffer.get_line_content(1);
```
