use criterion::{criterion_group, criterion_main, Criterion};
use text_buffer::TextBuffer;

fn insert_to_empty_document(c: &mut Criterion) {
    let mut buffer = setup_new_doc();

    c.bench_function("insert sentence at the start of an empty document", |b| {
        b.iter(|| buffer.insert(0, "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Praesent ultricies lacus ut molestie dapibus."))
    });
}

fn insert_to_existing_document(c: &mut Criterion) {
    let mut buffer = setup_existing_doc();

    c.bench_function("insert sentence in the middle of an existing document", |b| {
        b.iter(|| buffer.insert(0, "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Praesent ultricies lacus ut molestie dapibus."))
    });
}

fn delete_from_document(c: &mut Criterion) {
    let mut buffer = setup_existing_doc();

    c.bench_function("delete from existing document", |b| {
        b.iter(|| buffer.delete(100, 1200))
    });
}

fn get_line_content_near_start_of_document(c: &mut Criterion) {
    let buffer = setup_existing_doc();

    c.bench_function("get line content near start of document", |b| {
        b.iter(|| _ = buffer.get_line_content(5))
    });
}

fn get_line_content_near_end_of_document(c: &mut Criterion) {
    let buffer = setup_existing_doc();

    c.bench_function("get line content near end of document", |b| {
        b.iter(|| _ = buffer.get_line_content(50000))
    });
}

fn setup_existing_doc() -> TextBuffer {
    let ipsum_path = concat!(env!("CARGO_MANIFEST_DIR"), "/benches/ipsum");
    let text = std::fs::read_to_string(ipsum_path).expect("Unable to find file.");
    TextBuffer::new(Some(text))
}

fn setup_new_doc() -> TextBuffer {
    TextBuffer::new(None)
}

criterion_group!(
    benches,
    insert_to_empty_document,
    insert_to_existing_document,
    delete_from_document,
    get_line_content_near_start_of_document,
    get_line_content_near_end_of_document
);
criterion_main!(benches);
