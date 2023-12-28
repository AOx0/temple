use anyhow::{anyhow, ensure};

pub struct Delimiters<'a>(pub Delimiter<'a>, pub Delimiter<'a>);

impl Delimiters<'_> {
    #[must_use]
    pub fn delimiters(&self) -> (&str, &str) {
        (self.0 .0, self.1 .0)
    }

    #[must_use]
    pub fn sizes(&self) -> (usize, usize) {
        (self.0.len(), self.1.len())
    }

    #[must_use]
    pub fn first_open_chars(&self) -> (char, char) {
        (
            self.0.chars().next().expect("Asserted on Self::new"),
            self.0.chars().nth(1).expect("Asserted on Self::new"),
        )
    }

    #[must_use]
    pub fn first_close_chars(&self) -> (char, char) {
        (
            self.1.chars().next().expect("Asserted on Self::new"),
            self.1.chars().nth(1).expect("Asserted on Self::new"),
        )
    }

    #[must_use]
    pub fn find_start(&self, contents: &str, from: usize) -> Option<usize> {
        self.0.find_in(contents, from)
    }

    #[must_use]
    pub fn find_end(&self, contents: &str, from: usize) -> Option<usize> {
        self.1.find_in(contents, from)
    }
}

impl<'a> Delimiters<'a> {
    // TODO: Remove dead_code
    #[allow(dead_code)]
    fn new(open: &'a str, close: &'a str) -> anyhow::Result<Self> {
        ensure!(
            open.len() >= 2,
            anyhow!("Open delimiter {open} does not have a len of 2 or more")
        );
        ensure!(
            close.len() >= 2,
            anyhow!("Close delimiter {close} does not have a len of 2 or more")
        );
        ensure!(
            open.is_ascii(),
            anyhow!("Open delimiters can only contain ASCII characters")
        );
        ensure!(
            close.is_ascii(),
            anyhow!("Close delimiters can only contain ASCII characters")
        );

        Ok(Self(open.into(), close.into()))
    }
}

impl<'a> TryFrom<&'a tera::Value> for Delimiters<'a> {
    type Error = anyhow::Error;

    fn try_from(value: &'a tera::Value) -> anyhow::Result<Self> {
        let open = value
            .get("open")
            .ok_or(anyhow!("No open delimiter specified"))?
            .as_str()
            .ok_or(anyhow!("Invalid value as open delimiter"))?;

        let close = value
            .get("close")
            .ok_or(anyhow!("No close delimiter specified"))?
            .as_str()
            .ok_or(anyhow!("Invalid value as close delimiter"))?;

        ensure! {
            open.trim() != close.trim(),
            anyhow!("Both delimiters can't be the same")
        };

        Delimiters::new(open, close)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Delimiter<'a>(pub &'a str);

impl Delimiter<'_> {
    #[must_use]
    pub fn find_in(&self, slice: &str, from: usize) -> Option<usize> {
        if slice.is_empty() || slice.len() < from {
            return None;
        };

        slice[from..].find(self.0)
    }
}

impl<'a> From<&'a str> for Delimiter<'a> {
    fn from(value: &'a str) -> Self {
        Delimiter(value)
    }
}

impl std::ops::Deref for Delimiter<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
