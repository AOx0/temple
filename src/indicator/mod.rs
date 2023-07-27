use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Indicator<'a> {
    contents: &'a str,
}

impl AsRef<str> for Indicator<'_> {
    fn as_ref(&self) -> &str {
        self.contents
    }
}

impl<'a> Indicator<'a> {
    pub fn new(string: &'a str) -> Self {
        Indicator { contents: string }
    }

    pub fn find_in(&self, slice: &str, from: usize) -> Option<usize> {
        if slice.is_empty() || slice.len() - from < self.as_ref().len() {
            return None;
        };

        slice[from..]
            .as_bytes()
            .windows(self.as_ref().len())
            .enumerate()
            .find_map(|(i, x)| (x == self.as_ref().as_bytes()).then_some(i))
    }

    pub fn first_char(&self) -> Option<char> {
        self.as_ref().chars().next()
    }

    pub fn size(&self) -> usize {
        self.as_ref().len()
    }
}

impl Display for Indicator<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
