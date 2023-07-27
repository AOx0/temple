use crate::indicator::Indicator;

#[derive(Clone, Debug, Copy)]
pub struct Indicators<'a> {
    pub start: Indicator<'a>,
    pub end: Indicator<'a>,
}

impl<'a> Indicators<'a> {
    pub fn new(start: &'a str, end: &'a str) -> Result<Self, &'static str> {
        let start = Indicator::new(start);
        let end = Indicator::new(end);

        Ok(Indicators { start, end })
    }

    pub fn find_in(&self, contents: &str, from: usize, start: bool) -> std::option::Option<usize> {
        if start {
            self.start.find_in(contents, from)
        } else {
            self.end.find_in(contents, from)
        }
    }

    pub fn start_char(&self) -> Option<char> {
        self.start.first_char()
    }

    pub fn start_size(&self) -> usize {
        self.start.size()
    }

    pub fn end_size(&self) -> usize {
        self.end.size()
    }
}
