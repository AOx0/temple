use criterion::{criterion_group, criterion_main, Criterion};

pub fn replace_and_write(c: &mut Criterion) {
    c.bench_function("String conversion to keys and write of single file", |_| {});
}

criterion_group!(benches, replace_and_write);
criterion_main!(benches);
