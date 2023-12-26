pub use logos::Logos;
use logos::Span;
use owo_colors::OwoColorize;

#[derive(Debug, Default)]
pub struct Tokens<'i> {
    pub inp: &'i str,
    pub path: String,
    pub span: Vec<Span>,
    pub token: Vec<Variant<'i>>,
    pub cursor: usize,
}

pub struct Token<'i> {
    pub token: Variant<'i>,
    pub span: Span,
}

pub struct Peek<'re, 'i> {
    pub token: &'re Variant<'i>,
    pub span: &'re Span,
}

struct Underlined(Span);

impl std::fmt::Display for Underlined {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let span = self.0.clone();

        for _ in 0..(span.start - 1) {
            write!(f, " ")?;
        }

        write!(
            f,
            "{}",
            "^".repeat(span.end - span.start)
                .if_supports_color(owo_colors::Stream::Stdout, |s| {
                    s.style(owo_colors::Style::new().bold().yellow())
                }),
        )
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

fn get_line(inp: &str, line: usize) -> &str {
    inp.lines().nth(line - 1).unwrap_or_default()
}

impl Tokens<'_> {
    #[allow(clippy::field_reassign_with_default)]
    pub fn new(inp: &str, path: impl Into<String>) -> Tokens<'_> {
        let mut res = Tokens::default();
        res.inp = inp;
        res.path = path.into();
        res
    }

    pub fn error_current_span(&self, msg: impl Into<String>) -> String {
        self.error_at(self.current_location(), msg)
    }

    pub fn current_location(&self) -> Location {
        self.location(self.cursor)
    }

    pub fn location(&self, cursor: usize) -> Location {
        let span = self.span[cursor].clone();
        let mut line = 1;
        let mut col = 1;
        for i in 0..span.start {
            if self
                .inp
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

    #[allow(clippy::needless_pass_by_value)]
    pub fn error_at(&self, location: Location, msg: impl Into<String>) -> String {
        if self.is_empty() {
            String::new()
        } else {
            format!(
                "{msg}\n   {arrow}{path}:{line}:{start}\n{empty_pipe}\n{bline} {contents}\n{empty_pipe} {underline}\n{empty_pipe}",
                msg = msg.into(),
                arrow = "--> ".if_supports_color(owo_colors::Stream::Stdout, |s| s
                    .style(owo_colors::Style::new().bold().blue())),
                empty_pipe = format!("{: >3} |", "")
                    .if_supports_color(owo_colors::Stream::Stdout, |s| s
                    .style(owo_colors::Style::new().bold().blue())),
                bline = format!("{line: >3} |", line = location.1)
                    .if_supports_color(owo_colors::Stream::Stdout, |s| s
                    .style(owo_colors::Style::new().bold().blue())),
                path = self.path,
                line = location.1,
                start = location.0.start,
                contents = get_line(self.inp, location.1),
                underline = Underlined(location.0.clone())
            )
        }
    }

    pub fn steps(&mut self, steps: usize) {
        if std::env::var("TEMPLE_TRACE").is_ok() {
            for token in &self.token[self.cursor..self.cursor + steps] {
                crate::trace!("Shift {token} ({token:?})",);
            }
        }
        self.cursor += steps;
    }

    pub fn step(&mut self) {
        self.steps(1);
    }

    pub fn skiping(&mut self, steps: usize) -> &mut Self {
        self.steps(steps);
        self
    }

    pub fn get_ident(&self) -> Option<String> {
        if let &Variant::Ident(s) = self.peek().token {
            Some(s.to_string())
        } else {
            None
        }
    }

    pub fn tokens(&self) -> &[Variant<'_>] {
        &self.token[self.cursor..]
    }

    pub fn try_first(&self) -> Option<Token<'_>> {
        (!self.is_empty()).then(|| Token {
            token: self.token[self.cursor],
            span: self.span[self.cursor].clone(),
        })
    }

    pub fn peek(&self) -> Peek<'_, '_> {
        Peek {
            token: &self.token[self.cursor],
            span: &self.span[self.cursor],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cursor == self.span.len()
    }
}

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Variant<'i> {
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

    #[token("}")]
    CyClose,

    #[token("{")]
    CyOpen,

    #[token(",")]
    Comma,

    #[token(";")]
    Semicolon,

    #[token("=")]
    Eq,

    #[token(":")]
    EqD,

    #[regex(r#"(?i:number)"#)]
    KwNumber,

    #[regex(r#"(?i:string)"#)]
    KwString,

    #[regex(r#"(?i:bool)"#)]
    KwBool,

    #[regex(r#"(?i:object)"#)]
    KwObject,

    #[regex(r#"(?i:array)"#)]
    KwArray,

    #[regex(r#"(?i:any)"#)]
    KwAny,

    #[regex("(?i:[a-z][_a-z0-9]*)")]
    Ident(&'i str),

    #[regex("#[^\n]*", |lex| lex.slice())]
    Comment(&'i str),
    // #[regex(".*", |lex| lex.slice())]
    // Unknow(&'i str),
}

impl std::fmt::Display for Variant<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variant::FNumber(num) => write!(f, "{num}"),
            Variant::SNumber(num) => write!(f, "{num}"),
            Variant::UNumber(num) => write!(f, "{num}"),
            Variant::String(str) => write!(f, "{str:?}"),
            Variant::Bool(bool) => write!(f, "{bool}"),
            Variant::SqClose => write!(f, "']'"),
            Variant::SqOpen => write!(f, "'['"),
            Variant::CyClose => write!(f, "'}}'"),
            Variant::CyOpen => write!(f, "'{{'"),
            Variant::Comma => write!(f, "','"),
            Variant::Semicolon => write!(f, "';'"),
            Variant::Eq => write!(f, "'='"),
            Variant::EqD => write!(f, "':'"),
            Variant::KwNumber => write!(f, "Number"),
            Variant::KwString => write!(f, "String"),
            Variant::KwBool => write!(f, "Bool"),
            Variant::KwObject => write!(f, "Object"),
            Variant::KwArray => write!(f, "Array"),
            Variant::KwAny => write!(f, "Any"),
            Variant::Ident(ident) => write!(f, "{ident}"),
            // TokenE::Unknow(matched) => write!(f, "{matched}"),
            Variant::Comment(text) => write!(f, "# {text}"),
        }
    }
}

impl Variant<'_> {
    pub fn is_expr_decl(&self) -> bool {
        matches!(
            self,
            Variant::String(_)
                | Variant::UNumber(_)
                | Variant::SNumber(_)
                | Variant::SqOpen
                | Variant::CyOpen
        )
    }

    pub fn is_type_decl(&self) -> bool {
        matches!(
            self,
            Variant::KwNumber
                | Variant::KwString
                | Variant::KwArray
                | Variant::KwObject
                | Variant::KwBool
                | Variant::KwAny
                | Variant::SqOpen
                | Variant::CyOpen
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Logos, Variant};

    #[test]
    fn tokenize() {
        use Variant::*;

        let inp = "edades = [ 1, 2, 3, ]\nnombre = \"Daniel\"edad = 2";

        let tokens = Variant::lexer(inp);
        let tokens = tokens
            .into_iter()
            .map(std::result::Result::unwrap)
            .collect::<Vec<_>>();

        assert_eq!(
            tokens.as_slice(),
            &[
                Ident("edades"),
                Eq,
                SqOpen,
                UNumber(1),
                Comma,
                UNumber(2),
                Comma,
                UNumber(3),
                Comma,
                SqClose,
                Ident("nombre"),
                Eq,
                String("Daniel"),
                Ident("edad"),
                Eq,
                UNumber(2)
            ]
        );
    }
}
