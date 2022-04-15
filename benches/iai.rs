use iai::black_box;
use tem_p::{Contents, Indicator, Keys, Parse};

fn test1() {
    let mut contents = Contents::from(black_box("lmao {{ jaja }}"));
    let start = black_box(Indicator::from("{{ ", true)).unwrap();
    let end = black_box(Indicator::from(" }}", false)).unwrap();
    let keys = Keys::from_string("jaja=perro,");
    let replace = contents.replace(start, end, &keys);

    let r = if let Ok(res) = replace {
        match res.0 {
            666 => "No changes. No keys".to_string(),
            _ => Contents::get_str_from_result(&res.1).to_string(),
        }
    } else {
        "Invalid chars or data".to_string()
    };

    println!("{r}");
}

iai::main!(test1);
