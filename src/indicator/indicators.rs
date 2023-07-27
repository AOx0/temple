pub trait KeyIndicator
where
    Self: Sized,
{
    fn from_str(string: &str, is_start: bool) -> Result<Self, &'static str>;
    fn find_in(&self, slice: &[u8], from: usize) -> Option<usize>;
    fn first_char(&self) -> u8;
    fn size(&self) -> usize;
}

#[derive(Clone, Debug, PartialEq)]
pub struct IndicatorN(pub(crate) Vec<u8>, pub(crate) bool);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Indicator3D(pub(crate) u8, pub(crate) u8, pub(crate) u8, pub(crate) bool);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Indicator2D(pub(crate) u8, pub(crate) u8, pub(crate) bool);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Indicator1D(pub(crate) u8, pub(crate) bool);

macro_rules! bytes_or_err {
    ($string: expr, $len: tt) => {
        if $string.len() != $len {
            Err(concat!("Len must be ", $len))
        } else {
            Ok($string.as_bytes())
        }
    };

    ($string: expr) => {
        if $string.len() <= 0 {
            Err("Len must be more than 0")
        } else {
            Ok($string.as_bytes())
        }
    };
}

impl KeyIndicator for IndicatorN {
    fn from_str(string: &str, is_start: bool) -> Result<Self, &'static str> {
        let bytes = bytes_or_err!(string)?;
        Ok(IndicatorN(Vec::from(bytes), is_start))
    }

    fn find_in(&self, slice: &[u8], from: usize) -> Option<usize> {
        if slice.is_empty() || (from..slice.len()).count() < (self.0.len()) {
            return None;
        };

        for i in from..(slice.len() - if self.1 { self.0.len() } else { 0 }) {
            if (i..slice.len()).count() < (self.0.len()) {
                return None;
            };

            let equal = {
                let mut equal = true;
                for offset in 0..self.0.len() {
                    if slice[i + offset] != self.0[offset] {
                        equal = false;
                    }
                }
                equal
            };

            if equal {
                return Some(i);
            }
        }

        None
    }

    fn first_char(&self) -> u8 {
        self.0[0]
    }

    fn size(&self) -> usize {
        self.0.len()
    }
}

impl KeyIndicator for Indicator3D {
    fn from_str(string: &str, is_start: bool) -> Result<Self, &'static str> {
        let bytes = bytes_or_err!(string, 3)?;
        Ok(Indicator3D(bytes[0], bytes[1], bytes[2], is_start))
    }

    fn find_in(&self, slice: &[u8], from: usize) -> Option<usize> {
        if slice.is_empty() || slice.len() < 6 {
            return None;
        };
        for i in from..slice.len() - if self.3 { 3 } else { 0 } {
            if slice[i] == self.0 && slice[i + 1] == self.1 && slice[i + 2] == self.2 {
                return Some(i);
            }
        }
        None
    }

    fn first_char(&self) -> u8 {
        self.0
    }

    fn size(&self) -> usize {
        3usize
    }
}

impl KeyIndicator for Indicator2D {
    fn from_str(string: &str, is_start: bool) -> Result<Self, &'static str> {
        let bytes = bytes_or_err!(string, 2)?;
        Ok(Indicator2D(bytes[0], bytes[1], is_start))
    }

    fn find_in(&self, slice: &[u8], from: usize) -> Option<usize> {
        if slice.is_empty() || slice.len() < 4 {
            return None;
        };
        for i in from..slice.len() - if self.2 { 2 } else { 0 } {
            if slice[i] == self.0 && slice[i + 1] == self.1 {
                return Some(i);
            }
        }

        None
    }

    fn first_char(&self) -> u8 {
        self.0
    }

    fn size(&self) -> usize {
        2usize
    }
}

impl KeyIndicator for Indicator1D {
    fn from_str(string: &str, is_start: bool) -> Result<Self, &'static str> {
        let bytes = bytes_or_err!(string, 1)?;
        Ok(Indicator1D(bytes[0], is_start))
    }

    fn find_in(&self, slice: &[u8], from: usize) -> Option<usize> {
        if slice.is_empty() || slice.len() < 2 {
            return None;
        };

        #[allow(clippy::needless_range_loop)]
        for i in from..slice.len() - if self.1 { 1 } else { 0 } {
            if slice[i] == self.0 {
                return Some(i);
            }
        }

        None
    }

    fn first_char(&self) -> u8 {
        self.0
    }

    fn size(&self) -> usize {
        1usize
    }
}
