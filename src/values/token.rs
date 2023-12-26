pub use logos::Logos;
use logos::Span;
use owo_colors::OwoColorize;

use crate::values::{get_line, line_col};

#[derive(Debug, Default)]
pub struct Tokens<'i> {
    pub inp: &'i str,
    pub path: String,
    pub span: Vec<Span>,
    pub token: Vec<TokenE<'i>>,
    pub cursor: usize,
}

pub struct Token<'i> {
    pub token: TokenE<'i>,
    pub span: Span,
}

pub struct Peek<'re, 'i> {
    pub token: &'re TokenE<'i>,
    pub span: &'re Span,
}

#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    Error,
    Warning,
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::Error => write!(
                f,
                "{}",
                "error".if_supports_color(owo_colors::Stream::Stdout, |s| s
                    .style(owo_colors::Style::new().bold().red()))
            ),
            MessageType::Warning => write!(
                f,
                "{}",
                "warning".if_supports_color(owo_colors::Stream::Stdout, |s| s
                    .style(owo_colors::Style::new().bold().yellow()))
            ),
        }
    }
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

impl Tokens<'_> {
    #[allow(clippy::field_reassign_with_default)]
    pub fn new(inp: &str, path: impl Into<String>) -> Tokens<'_> {
        let mut res = Tokens::default();
        res.inp = inp;
        res.path = path.into();
        res
    }

    pub fn error_current_span(&self, typ: MessageType, msg: impl Into<String>) -> String {
        if self.is_empty() {
            String::new()
        } else {
            let location = line_col(self.inp, self.span[self.cursor].clone());
            format!(
                "{typ}: {msg}\n  {arrow}{path}:{line}:{start}\n{empty_pipe}\n{bline} {contents}\n{empty_pipe} {underline}\n{empty_pipe}",
                typ = typ,
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
        if std::env::var("TEMPLE_INFO").is_ok() {
            for token in &self.token[self.cursor..self.cursor + steps] {
                println!(
                    "{}: Shift {token} ({token:?})",
                    "info".if_supports_color(owo_colors::Stream::Stdout, |s| s
                        .style(owo_colors::Style::new().bold().green()))
                );
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
        if let &TokenE::Ident(s) = self.peek().token {
            Some(s.to_string())
        } else {
            None
        }
    }

    pub fn tokens(&self) -> &[TokenE<'_>] {
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
pub enum TokenE<'i> {
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

impl std::fmt::Display for TokenE<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenE::FNumber(num) => write!(f, "{num}"),
            TokenE::SNumber(num) => write!(f, "{num}"),
            TokenE::UNumber(num) => write!(f, "{num}"),
            TokenE::String(str) => write!(f, "{str:?}"),
            TokenE::Bool(bool) => write!(f, "{bool}"),
            TokenE::SqClose => write!(f, "']'"),
            TokenE::SqOpen => write!(f, "'['"),
            TokenE::CyClose => write!(f, "'}}'"),
            TokenE::CyOpen => write!(f, "'{{'"),
            TokenE::Comma => write!(f, "','"),
            TokenE::Semicolon => write!(f, "';'"),
            TokenE::Eq => write!(f, "'='"),
            TokenE::EqD => write!(f, "':'"),
            TokenE::KwNumber => write!(f, "Number"),
            TokenE::KwString => write!(f, "String"),
            TokenE::KwBool => write!(f, "Bool"),
            TokenE::KwObject => write!(f, "Object"),
            TokenE::KwArray => write!(f, "Array"),
            TokenE::KwAny => write!(f, "Any"),
            TokenE::Ident(ident) => write!(f, "{ident}"),
            // TokenE::Unknow(matched) => write!(f, "{matched}"),
            TokenE::Comment(text) => write!(f, "# {text}"),
        }
    }
}

impl TokenE<'_> {
    pub fn is_expr_decl(&self) -> bool {
        matches!(
            self,
            TokenE::String(_)
                | TokenE::UNumber(_)
                | TokenE::SNumber(_)
                | TokenE::SqOpen
                | TokenE::CyOpen
        )
    }

    pub fn is_type_decl(&self) -> bool {
        matches!(
            self,
            TokenE::KwNumber
                | TokenE::KwString
                | TokenE::KwArray
                | TokenE::KwObject
                | TokenE::KwBool
                | TokenE::KwAny
                | TokenE::SqOpen
                | TokenE::CyOpen
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Logos, TokenE};

    #[test]
    fn tokenize() {
        use TokenE::*;

        let inp = "edades = [ 1, 2, 3, ]\nnombre = \"Daniel\"edad = 2";

        let tokens = TokenE::lexer(inp);
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
