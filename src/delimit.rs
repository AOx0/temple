use anyhow::{anyhow, ensure};

pub struct Delimiters<'a>(Delimiter<'a>, Delimiter<'a>);

impl Delimiters<'_> {
    fn start(&self) -> Delimiter<'_> {
        self.0
    }

    fn end(&self) -> Delimiter<'_> {
        self.1
    }

    fn start_size(&self) -> usize {
        self.0.len()
    }

    fn end_size(&self) -> usize {
        self.1.len()
    }

    fn first_open_chars(&self) -> (char, char) {
        (
            self.0.chars().next().expect("Asserted on Self::new"),
            self.0.chars().nth(1).expect("Asserted on Self::new"),
        )
    }

    fn first_close_chars(&self) -> (char, char) {
        (
            self.1.chars().next().expect("Asserted on Self::new"),
            self.1.chars().nth(1).expect("Asserted on Self::new"),
        )
    }
}

impl<'a> Delimiters<'a> {
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

#[derive(Debug, Clone, Copy)]
pub struct Delimiter<'a>(&'a str);

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
