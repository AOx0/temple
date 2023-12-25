pub use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token<'i> {
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

    #[regex("(?i:[a-z][a-z0-9]*)")]
    Ident(&'i str),
}

impl Token<'_> {
    pub fn is_expr_decl(&self) -> bool {
        matches!(
            self,
            Token::String(_)
                | Token::UNumber(_)
                | Token::SNumber(_)
                | Token::SqOpen
                | Token::CyOpen
        )
    }

    pub fn is_type_decl(&self) -> bool {
        matches!(
            self,
            Token::KwNumber
                | Token::KwString
                | Token::KwArray
                | Token::KwObject
                | Token::KwBool
                | Token::KwAny
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Logos, Token};

    #[test]
    fn tokenize() {
        use Token::*;

        let inp = "edades = [ 1, 2, 3, ]\nnombre = \"Daniel\"edad = 2";

        let tokens = Token::lexer(inp);
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
