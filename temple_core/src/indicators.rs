use crate::indicator::{Indicator, KeyIndicator};

#[derive(Clone, Debug)]
pub struct Indicators {
    pub start: Indicator,
    pub end: Indicator,
}

impl Indicators {
    pub fn new(start: &str, end: &str) -> Result<Self, &'static str> {
        let start = Indicator::from_str(start, true)?;
        let end = Indicator::from_str(end, false)?;

        Ok(Indicators { start, end })
    }

    pub fn find_in(&self, contents: &[u8], from: usize, start: bool) -> std::option::Option<usize> {
        if start {
            self.start.find_in(contents, from)
        } else {
            self.end.find_in(contents, from)
        }
    }

    pub fn start_char(&self) -> u8 {
        self.start.first_char()
    }

    pub fn start_size(&self) -> usize {
        self.start.size()
    }

    pub fn end_size(&self) -> usize {
        self.end.size()
    }
}
