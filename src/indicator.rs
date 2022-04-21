#[derive(Clone, Copy)]
pub struct Indicator(pub(crate) u8, pub(crate) u8, pub(crate) u8, pub(crate) bool);

impl Indicator {
    pub fn from(string: &str, is_start: bool) -> Result<Indicator, &str> {
        if string.len() != 3 {
            Err("Len must be 3")
        } else {
            let bytes = string.as_bytes();
            Ok(Indicator(bytes[0], bytes[1], bytes[2], is_start))
        }
    }
}
