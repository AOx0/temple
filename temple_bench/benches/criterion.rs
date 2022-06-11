use criterion::{black_box, criterion_group, criterion_main, Criterion};
use temple_core::*;

pub fn replace_and_write(c: &mut Criterion) {
    c.bench_function("String conversion to keys and write of single file", |b| {
        b.iter(|| {
            let mut contents = Contents::from(black_box("lmao {{ jaja }}"));
            let ind = black_box(Indicators::new("{{ ", " }}")).unwrap();
            let keys = Keys::from("jaja=perro,");
            let replace = contents.replace(&ind, &keys);

            let r = if let Ok(res) = replace {
                match res.0 {
                    666 => String::from("No changes. No keys"),
                    _ => Contents::get_str_from_result(&res.1),
                }
            } else {
                String::from("Invalid chars or data")
            };

            println!("{r}");
        })
    });
}

criterion_group!(benches, replace_and_write);
criterion_main!(benches);
