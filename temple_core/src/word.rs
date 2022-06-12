use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

#[derive(Clone, Copy)]
pub struct Word {
    pub(crate) contents: [u8; 300],
    pub(crate) size: usize,
}

impl Display for Word {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {})",
            String::from_str(std::str::from_utf8(&self.contents[0..self.size]).unwrap()).unwrap(),
            self.size
        )
    }
}

impl Word {
    pub(crate) fn new() -> Word {
        Word {
            contents: [0u8; 300],
            size: 0usize,
        }
    }

    #[allow(unused)]
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.contents[0..self.size]).unwrap()
    }

    pub(crate) fn set(&mut self, slice: &[u8], size: usize) {
        for (i, &byte) in slice.iter().enumerate() {
            self.contents[i] = byte;
        }

        self.size = size;
    }
}
