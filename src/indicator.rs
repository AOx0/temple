use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Indicator<'a>(&'a str);

impl AsRef<str> for Indicator<'_> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'a> Indicator<'a> {
    #[must_use]
    pub fn new(s: &'a str) -> Self {
        Indicator(s)
    }

    #[must_use]
    pub fn find_in(&self, slice: &str, from: usize) -> Option<usize> {
        if slice.is_empty() || slice.len() < from {
            return None;
        };

        slice[from..].find(self.as_ref())
    }

    #[must_use]
    pub fn first_char(&self) -> Option<char> {
        self.as_ref().chars().next()
    }

    #[must_use]
    pub fn size(&self) -> usize {
        self.as_ref().len()
    }
}

impl Display for Indicator<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
