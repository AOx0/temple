pub mod contents;
mod indicator;
pub mod indicators;
pub mod keys;
mod word;

pub use contents::Contents;
pub use indicators::Indicators;
pub use keys::Keys;
pub use smartstring::alias::String;

#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn basic_parse() {
        let mut contents = Contents::from("lmao {{ jaja }}");
        let indicators = Indicators::new("{{ ", " }}").unwrap();
        let keys = Keys::from("jaja=perro");
        let replace = contents.replace(&indicators, &keys);

        let r = if let Ok(res) = replace {
            match res.0 {
                666 => String::from("No changes. No keys"),
                _ => Contents::get_str_from_result(&res.1),
            }
        } else {
            String::from("Invalid chars or data")
        };

        println!("{r}");
        assert_eq!(r, "lmao perro");
    }
}
