use std::str::FromStr;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use temple::*;

pub fn replace_and_write(c: &mut Criterion) {
    c.bench_function("String conversion to keys and write of single file", |b| {
        b.iter(|| {
            let mut contents = Contents::from_str("lmao {{ jaja }}").unwrap();
            let indicators = Indicators::new("{{ ", " }}").unwrap();
            let keys = Keys::from("jaja=perro");
            let replace = contents.replace(indicators, &keys);

            let r = if let Ok(res) = replace {
                res.get_string()
            } else {
                String::from("Invalid chars or data")
            };

            println!("{r}");
            assert_eq!(r, "lmao perro");
        })
    });
}

criterion_group!(benches, replace_and_write);
criterion_main!(benches);
