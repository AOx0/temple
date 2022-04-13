use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::borrow::Borrow;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use temple_parse::{Contents, Indicator, Keys, Parse};

/// Test with multiple values
pub fn replacements_found(c: &mut Criterion) {
    let mut group = c.benchmark_group("replacements_found");
    for contents in [
        black_box("{{ jaja }}}"),
        black_box("{{{ ok }}} {{{ ok }}} {{{ ok }}}"),
        black_box("{{{ ok }}} {{{ ok }}} {{{{{ ok }}}}} {{{"),
    ]
    .iter()
    {
        group.throughput(Throughput::Bytes(contents.as_bytes().len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(contents),
            contents,
            |b, &contents| {
                b.iter(|| {
                    let mut contents = Contents::from(black_box(contents));
                    let start = black_box(Indicator::from("{{{", true)).unwrap();
                    let end = black_box(Indicator::from("}}}", false)).unwrap();
                    let replace = contents.replace(start, end);

                    let r = if let Ok(res) = replace {
                        match res.0 {
                            666 => "No changes. No keys".to_string(),
                            _ => Contents::get_str_from_result(&res.1).to_string(),
                        }
                    } else {
                        "Invalid chars or data".to_string()
                    };

                    println!("{r}");
                });
            },
        );
    }
    group.finish();
}

/// Test with multiple values
pub fn replacements_found_file(c: &mut Criterion) {
    let mut group = c.benchmark_group("replacements_found_file");
    for contents in [
        black_box("/Users/alejandro/.pi_templates/madlang-miso/src/Main.hs"),
        black_box("/Users/alejandro/.pi_templates/madlang-miso/README.md"),
        black_box("/Users/alejandro/.pi_templates/madlang-miso/{{ project }}.cabal"),
    ]
    .iter()
    {
        // group.throughput(Throughput::Bytes(contents.as_bytes().len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(contents),
            contents,
            |b, &contents| {
                b.iter(|| {
                    let mut contents =
                        Contents::from_file(black_box(contents.parse().unwrap())).unwrap();
                    let start = black_box(Indicator::from("{{ ", true)).unwrap();
                    let end = black_box(Indicator::from(" }}", false)).unwrap();
                    let replace = contents.replace(start, end);

                    let r = if let Ok(res) = replace {
                        match res.0 {
                            666 => "No changes. No keys".to_string(),
                            _ => Contents::get_str_from_result(&res.1).to_string(),
                        }
                    } else {
                        "Invalid chars or data".to_string()
                    };

                    println!("{r}");
                });
            },
        );
    }
    group.finish();
}

pub fn keys_from_string(c: &mut Criterion) {
    c.bench_function("String conversion to keys", |b| {
        b.iter(|| {
            let keys = Keys::from_string(black_box(
                "\
    name=Perro;\n\
    nombre=Daniel Alejandro\n\
    \
    ",
            ));

            keys.list.iter().for_each(|e| println!("{:?}", e));
        })
    });
}

pub fn replace_and_write(c: &mut Criterion) {
    c.bench_function("String conversion to keys and write of single file", |b| {
        b.iter(|| {
            let mut contents = Contents::from_file(black_box(PathBuf::from(
                "/Users/alejandro/.pi_templates/madlang-miso/src/Main.hs",
            )))
            .unwrap();
            let start = black_box(Indicator::from("{{ ", true)).unwrap();
            let end = black_box(Indicator::from(" }}", false)).unwrap();
            let replace = contents.replace(start, end).unwrap();

            Contents::write_to_target(
                &replace.1,
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .append(false)
                    .open("/Users/alejandro/CLionProjects/temple/temple_parse/benches/out.txt")
                    .unwrap(),
            )
        })
    });
}

pub fn replace_and_write2(c: &mut Criterion) {
    c.bench_function(
        "String conversion to keys and write of single file 2",
        |b| {
            b.iter(|| {
                let mut contents = Contents::from_file(black_box(PathBuf::from(
                    "/Users/alejandro/.pi_templates/madlang-miso/{{ project }}.cabal",
                )))
                .unwrap();
                let start = black_box(Indicator::from("{{ ", true)).unwrap();
                let end = black_box(Indicator::from(" }}", false)).unwrap();
                let replace = contents.replace(start, end).unwrap();

                Contents::write_to_target(
                    &replace.1,
                    OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .append(false)
                        .open("/Users/alejandro/CLionProjects/temple/temple_parse/benches/out.txt")
                        .unwrap(),
                )
            })
        },
    );
}

criterion_group!(
    benches,
    replacements_found,
    keys_from_string,
    replacements_found_file,
    replace_and_write,
    replace_and_write2
);
criterion_main!(benches);
