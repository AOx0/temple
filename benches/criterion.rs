use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tem_p::{Contents, Indicator, Keys, Parse};

pub fn replace_and_write(c: &mut Criterion) {
    c.bench_function("String conversion to keys and write of single file", |b| {
        b.iter(|| {
            let mut contents = Contents::from(black_box("lmao {{ jaja }}"));
            let start = black_box(Indicator::from("{{ ", true)).unwrap();
            let end = black_box(Indicator::from(" }}", false)).unwrap();
            let keys = Keys::from("jaja=perro,");
            let replace = contents.replace(&start, &end, &keys);

            let r = if let Ok(res) = replace {
                match res.0 {
                    666 => "No changes. No keys".to_string(),
                    _ => Contents::get_str_from_result(&res.1).to_string(),
                }
            } else {
                "Invalid chars or data".to_string()
            };

            println!("{r}");
        })
    });
}

criterion_group!(benches, replace_and_write);
criterion_main!(benches);
