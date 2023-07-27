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

    #[must_use] pub fn find_start(&self, contents: &str, from: usize) -> Option<usize> {
        self.start.find_in(contents, from)
    }

    #[must_use] pub fn find_end(&self, contents: &str, from: usize) -> Option<usize> {
        self.end.find_in(contents, from)
    }

    #[must_use] pub fn start_char(&self) -> Option<char> {
        self.start.first_char()
    }

    #[must_use] pub fn start_size(&self) -> usize {
        self.start.size()
    }

    #[must_use] pub fn end_size(&self) -> usize {
        self.end.size()
    }
}
