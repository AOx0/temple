mod token;
use anyhow::{anyhow, bail, ensure, Result};
use std::collections::HashMap;
use tera::{Number, Value};
use token::{Logos, Token};

#[derive(Debug, PartialEq, Eq)]
pub struct Config(pub HashMap<String, Expr>);

impl std::str::FromStr for Config {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let tokens = Token::lexer(s)
            .collect::<std::result::Result<Vec<Token<'_>>, _>>()
            .map_err(|()| anyhow!("Failed to get tokens"))?;

        if let (Expr::Object(config), remain) = parse_object(&tokens, false)? {
            ensure!(
                remain.is_empty(),
                "Error while parsing config, bad syntax, ramains: {remain:?}"
            );

            Ok(config)
        } else {
            unreachable!()
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Array(Vec<Expr>),
    Object(Config),
    UNumber(usize),
    SNumber(isize),
    String(String),
    Bool(bool),
}

impl Expr {
    #[must_use]
    pub fn to_value(self) -> Value {
        match self {
            Expr::Array(a) => Value::Array(a.into_iter().map(Self::to_value).collect::<Vec<_>>()),
            Expr::Object(Config(o)) => {
                let kvs = o.into_iter().collect::<Vec<_>>();
                let map = kvs.into_iter().map(|(k, v)| (k, v.to_value()));
                Value::Object(map.collect::<tera::Map<_, _>>())
            }
            Expr::UNumber(n) => Value::Number(Number::from(n)),
            Expr::SNumber(n) => Value::Number(Number::from(n)),
            Expr::String(s) => Value::String(s),
            Expr::Bool(b) => Value::Bool(b),
        }
    }

    fn try_from<'a>(value: &'a [Token<'a>]) -> Result<(Self, &'a [Token<'a>]), anyhow::Error> {
        match value {
            [Token::String(_), ..] => parse_string(value),
            [Token::UNumber(_), ..] => parse_uint(value),
            [Token::SNumber(_), ..] => parse_sint(value),
            [Token::SqOpen, ..] => parse_list(value),
            [Token::CyOpen, ..] => parse_object(value, true),
            _ => unimplemented!(),
        }
    }

    fn is_parseable(token: Token<'_>) -> bool {
        matches!(
            token,
            Token::String(_)
                | Token::UNumber(_)
                | Token::SNumber(_)
                | Token::SqOpen
                | Token::CyOpen
        )
    }
}

fn parse_object<'a>(
    token: &'a [Token<'a>],
    with_curly: bool,
) -> Result<(Expr, &'a [Token<'a>]), anyhow::Error> {
    if matches!(token, [Token::CyOpen, ..]) || !with_curly {
        let mut remain = if matches!(token, [Token::CyOpen, ..]) && with_curly {
            &token[1..]
        } else {
            token
        };

        if let &[Token::CyClose, ..] = remain {
            Ok((Expr::Object(Config(HashMap::new())), &remain[1..]))
        } else {
            let mut res = HashMap::new();

            while let &[Token::Ident(ident), Token::Eq | Token::EqD, ..] = remain {
                remain = &remain[2..];

                let (expr, r) = Expr::try_from(remain)?;

                res.insert(ident.to_string(), expr)
                    .is_some()
                    .then(|| println!("Warn: Ident `{ident}` is already defined, overriding"));

                remain = r;

                if let [Token::Comma, ..] = remain {
                    remain = &remain[1..];
                }
            }

            ensure! {
                matches!(remain, [Token::CyClose, ..]) || !with_curly,
                "Warn: Invalid syntax on onbject declaration, remains: {remain:?}"
            };

            if matches!(remain, [Token::CyClose, ..]) && with_curly {
                Ok((Expr::Object(Config(res)), &remain[1..]))
            } else {
                Ok((Expr::Object(Config(res)), remain))
            }
        }
    } else {
        Err(anyhow!("Token is not an object declaration"))
    }
}

fn parse_list<'a>(token: &'a [Token<'a>]) -> Result<(Expr, &'a [Token<'a>]), anyhow::Error> {
    if let &[Token::SqOpen, ..] = token {
        let mut rem = &token[1..];

        if !rem.contains(&Token::SqClose) {
            bail!("Open list is not closed")
        }

        if let &[Token::SqClose, ..] = rem {
            Ok((Expr::Array(Vec::new()), &rem[1..]))
        } else {
            let mut list = Vec::new();

            while let [next, ..] = rem {
                match (Expr::is_parseable(*next), rem) {
                    (true, _) => {
                        let (expr, r) = Expr::try_from(rem)?;
                        list.push(expr);
                        rem = r;
                    }
                    (false, [Token::Comma, Token::Comma, ..]) => {
                        bail!("There must be a value between commas")
                    }
                    (false, [Token::SqClose, ..]) => {
                        rem = &rem[1..];
                        break;
                    }
                    (false, [Token::Comma, ..]) => rem = &rem[1..],
                    (false, [token, ..]) => {
                        bail!("Invalid token in list: `{token:?}`")
                    }
                    _ => unreachable!("We did check the list is not empty"),
                }
            }

            if let Some(Token::Comma) = rem.first() {
                rem = &rem[1..];
            }

            Ok((Expr::Array(list), rem))
        }
    } else {
        Err(anyhow!("Token is not a list"))
    }
}

fn parse_string<'a>(token: &'a [Token<'a>]) -> Result<(Expr, &'a [Token<'a>]), anyhow::Error> {
    if let &[Token::String(s), ..] = token {
        Ok((Expr::String(s.to_owned()), &token[1..]))
    } else {
        Err(anyhow!("Token is not a string"))
    }
}

fn parse_sint<'a>(token: &'a [Token<'a>]) -> Result<(Expr, &'a [Token<'a>])> {
    if let &[Token::SNumber(num), ..] = token {
        Ok((Expr::SNumber(num), &token[1..]))
    } else {
        Err(anyhow!("Token is not a signed number"))
    }
}

fn parse_uint<'a>(token: &'a [Token<'a>]) -> Result<(Expr, &'a [Token<'a>])> {
    if let &[Token::UNumber(num), ..] = token {
        Ok((Expr::UNumber(num), &token[1..]))
    } else {
        Err(anyhow!("Token is not a unsigned number"))
    }
}

#[cfg(test)]
mod tests {
    use logos::Logos;

    use super::{token::Token, Expr};

    #[test]
    fn comma_ending_list() {
        let inp = "[1, 2, 3, ]";

        let tokens = Token::lexer(inp)
            .map(std::result::Result::unwrap)
            .collect::<Vec<_>>();

        assert_eq!(
            Expr::try_from(tokens.as_slice()).expect("This should never fail"),
            (
                Expr::Array(vec![Expr::UNumber(1), Expr::UNumber(2), Expr::UNumber(3)]),
                [].as_slice()
            )
        );
    }

    #[test]
    fn non_comma_ending_list() {
        let inp = "[1, 2, 3 ]";

        let tokens = Token::lexer(inp)
            .map(std::result::Result::unwrap)
            .collect::<Vec<_>>();

        assert_eq!(
            Expr::try_from(tokens.as_slice()).expect("This should never fail"),
            (
                Expr::Array(vec![Expr::UNumber(1), Expr::UNumber(2), Expr::UNumber(3)]),
                [].as_slice()
            )
        );
    }

    #[test]
    fn invalid_list() {
        let inp = "[1,,]";

        let tokens = Token::lexer(inp)
            .map(std::result::Result::unwrap)
            .collect::<Vec<_>>();

        assert_eq!(
            Expr::try_from(tokens.as_slice())
                .map_err(|e| { e.to_string() })
                .expect_err("This should never fail")
                .as_str(),
            "There must be a value between commas"
        );
    }
}
