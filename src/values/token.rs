pub use logos::Logos;
use logos::Span;

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

pub struct TokenPeek<'re, 'i> {
    pub token: &'re TokenE<'i>,
    pub span: &'re Span,
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
        if self.is_empty() {
            String::new()
        } else {
            let location = line_col(self.inp, self.span[self.cursor].clone());
            format!(
                "{msg}\n    {path}:{line}:{start} {contents}",
                msg = msg.into(),
                path = self.path,
                line = location.1,
                start = location.0.start,
                contents = get_line(self.inp, location.1)
            )
        }
    }

    pub fn steps(&mut self, steps: usize) {
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

    pub fn try_span(&self) -> Option<Span> {
        (!self.is_empty()).then(|| self.span[self.cursor].clone())
    }

    pub fn peek(&self) -> TokenPeek<'_, '_> {
        TokenPeek {
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
