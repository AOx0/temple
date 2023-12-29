use anyhow::{anyhow, Result};
use logos::{Logos, Span};
use owo_colors::OwoColorize;
use std::{borrow::Cow, fmt::Write, path::Path, sync::OnceLock};

use crate::{delimit::Delimiters, error, values::Values};

pub struct ContentsLexer<'i> {
    pub in_delimiter: bool,
    pub indicators: Delimiters<'i>,
    pub origin: &'i Path,
    pub state: logos::Lexer<'i, Type<'i>>,
    pub content: &'i str,
    pub next: Option<(Result<Type<'i>, anyhow::Error>, Span)>,
    pub returned_raw: bool,
    pub returned_close: bool,
}

static DELIMITERS: OnceLock<(String, String)> = OnceLock::new();

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
#[logos(
    error = (),
    skip r"[ \t\n\f]+"
)]
pub enum Type<'i> {
    #[regex("[+-]?[0-9]*[.][0-9]*", |lex| format!("{num}0", num = lex.slice()).parse().ok())]
    FNumber(f64),

    #[regex("[+-][1-9][0-9]*", |lex| lex.slice().parse().ok())]
    SNumber(isize),

    #[regex("0", |_| 0)]
    #[regex("[1-9][0-9]*", |lex| lex.slice().parse().ok())]
    UNumber(usize),

    #[regex(r#""[^"]*""#, |lex| lex.slice().trim_matches('"'))]
    #[regex(r#"'[^']*'"#, |lex| lex.slice().trim_matches('\''))]
    String(&'i str),

    #[regex(r#"(?i:false)"#, |_| false)]
    #[regex(r#"(?i:true)"#, |_| true)]
    Bool(bool),

    #[token("]")]
    SqClose,

    #[token("[")]
    SqOpen,

    #[regex("(?i:if)")]
    KwIf,
    #[regex("(?i:for)")]
    KwFor,
    #[regex("(?i:in)")]
    KwIn,
    #[token("==")]
    Eq,
    #[regex("(?i:[a-z][_a-z0-9]*)", priority = 1)]
    Ident(&'i str),
    DelimitOpen,
    DelimitClose,
    #[regex("#[^\n]*", |lex| lex.slice())]
    Comment(&'i str),
    Raw(&'i str),

    #[regex(r##"[^ \t\n\f][^ \t\n\f]"##, delimiter, priority = 0)]
    PotentialDelim(DelimiterType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelimiterType {
    DelimitOpen,
    DelimitClose,
}

fn delimiter<'i>(lex: &mut logos::Lexer<'i, Type<'i>>) -> logos::FilterResult<DelimiterType, ()> {
    let (ref start, ref end) = DELIMITERS.get().expect("Set on init");
    if lex.slice().starts_with(&start[..2]) && lex.remainder().starts_with(&start[2..]) {
        lex.bump(start.len() - 2);
        logos::FilterResult::Emit(DelimiterType::DelimitOpen)
    } else if lex.slice().starts_with(&end[..2]) && lex.remainder().starts_with(&end[2..]) {
        lex.bump(end.len() - 2);
        logos::FilterResult::Emit(DelimiterType::DelimitClose)
    } else {
        logos::FilterResult::Error(())
    }
}

impl<'i> ContentsLexer<'i> {
    pub fn new(s: &'i str, path: &'i Path, config: &'i Values) -> anyhow::Result<Self> {
        let indicators: Delimiters<'_> = config
            .value_map
            .get("temple_delimiters")
            .ok_or(anyhow!(
                "Delimiters must be set with the identifier 'temple_delimiters'"
            ))?
            .try_into()?;

        DELIMITERS
            .set((
                indicators.delimiters().0.to_owned(),
                indicators.delimiters().1.to_owned(),
            ))
            .map_err(|e| anyhow!("Failed setting OnceLock: {e:?}"))?;

        Ok(ContentsLexer {
            next: None,
            content: s,
            in_delimiter: false,
            indicators,
            state: Type::lexer(s),
            origin: path,
            returned_raw: false,
            returned_close: false,
        })
    }
}

impl<'i> std::ops::Deref for ContentsLexer<'i> {
    type Target = logos::Lexer<'i, Type<'i>>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<'i> std::ops::DerefMut for ContentsLexer<'i> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

pub struct Replaced<'a>(Vec<Cow<'a, str>>);

impl Replaced<'_> {
    pub fn write_to_file(&self, mut target: impl std::io::Write) -> Result<()> {
        for r in self.get_iter() {
            match r {
                Cow::Owned(slice) => target.write_all(slice.as_bytes())?,
                Cow::Borrowed(slice) => target.write_all(slice.as_bytes())?,
            }
        }

        Ok(())
    }

    pub fn get_iter(&self) -> std::slice::Iter<'_, Cow<'_, str>> {
        self.0.iter()
    }

    pub fn extend_str(&self, t: &mut String) -> Result<()> {
        t.clear();
        for r in self.get_iter() {
            match r {
                Cow::Owned(slice) => t.write_str(slice)?,
                Cow::Borrowed(slice) => t.write_str(slice)?,
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn get_string(self) -> String {
        self.0.concat()
    }
}

pub struct Location(Span, usize);

impl From<(Span, usize)> for Location {
    fn from((span, line): (Span, usize)) -> Self {
        Location(span, line)
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}-{}", self.1, self.0.start, self.0.end)
    }
}

struct Underlined(usize);

impl std::fmt::Display for Underlined {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            "^".repeat(self.0)
                .if_supports_color(owo_colors::Stream::Stdout, |s| {
                    s.style(owo_colors::Style::new().bold().yellow())
                }),
        )
    }
}

impl ContentsLexer<'_> {
    fn get_line(&self, line: usize) -> &str {
        self.content.lines().nth(line - 1).unwrap_or_default()
    }

    pub fn span(&self) -> Span {
        if self.returned_raw {
            let mut span = self.state.span();
            span.start += self.indicators.delimiters().1.len();

            span
        } else {
            self.state.span()
        }
    }

    pub fn slice(&self) -> &str {
        if self.returned_raw {
            &self.state.slice()[self.indicators.delimiters().1.len()..]
        } else {
            self.state.slice()
        }
    }

    fn error_at(&self, location: Location, msg: impl Into<String>) -> String {
        format!(
            "{msg}\n   {arrow}{path}:{line}:{start}\n{empty_pipe}\n{bline} {contents}\n{empty_pipe} {underline}",
            msg = msg.into(),
            arrow = "--> ".if_supports_color(owo_colors::Stream::Stdout, |s| s
                .style(owo_colors::Style::new().bold().blue())),
            empty_pipe = format!("{: >3} |", "")
                .if_supports_color(owo_colors::Stream::Stdout, |s| s
                .style(owo_colors::Style::new().bold().blue())),
            bline = format!("{line: >3} |", line = location.1)
                .if_supports_color(owo_colors::Stream::Stdout, |s| s
                .style(owo_colors::Style::new().bold().blue())),
            path = self.origin.display(),
            line = location.1,
            start = location.0.start,
            contents = self.get_line(location.1),
            underline = Underlined(self.get_line(location.1).len())
        )
    }

    #[must_use]
    pub fn get_location(&self, span: Span) -> Location {
        let mut line = 1;
        let mut col = 1;
        for i in 0..span.start {
            if self
                .content
                .chars()
                .nth(i)
                .expect("Logos returns valid spans always")
                == '\n'
            {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (col..col + (span.end - span.start), line).into()
    }
}

impl<'i> Iterator for ContentsLexer<'i> {
    type Item = Result<Type<'i>, anyhow::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.returned_raw = false;

        if self.state.remainder().is_empty() {
            return None;
        }

        if self.in_delimiter {
            let next = self
                .state
                .next()
                .as_mut()
                .map(|v| v.map_err(|()| anyhow!("")));

            self.in_delimiter = !matches!(
                next,
                Some(Ok(Type::PotentialDelim(DelimiterType::DelimitClose)))
            );
            self.returned_close = !self.in_delimiter;

            next
        } else if let Some(n) = self.indicators.find_start(self.remainder(), 0) {
            if self.indicators.find_end(self.remainder(), n).is_none() {
                let mut span = self.state.span();

                span.start += n;
                span.end += self.indicators.0.len();

                return Some(Err(anyhow!(self.error_at(
                    self.get_location(span),
                    format!("Unclosed delimiter {}", self.indicators.0 .0),
                ))));
            }

            let raw = &self.remainder()[..n];
            self.in_delimiter = true;

            if self.returned_close {
                self.returned_raw = true;
                self.returned_close = false;
            }

            self.bump(raw.len());

            Some(Ok(Type::Raw(raw)))
        } else {
            self.returned_raw = true;

            let rem = self.remainder();
            self.bump(rem.len());

            Some(Ok(Type::Raw(rem)))
        }
    }
}
