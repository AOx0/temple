use anyhow::{anyhow, bail, ensure, Context, Result};
use tera::{Map, Number, Value};

use super::token::Token;

pub fn try_from<'a>(tokens: &'a [Token<'a>]) -> Result<(Value, &'a [Token<'a>]), anyhow::Error> {
    match tokens {
        &[Token::String(s), ..] => Ok((Value::String(s.to_string()), &tokens[1..])),
        &[Token::UNumber(v), ..] => Ok((Value::Number(v.into()), &tokens[1..])),
        &[Token::SNumber(v), ..] => Ok((Value::Number(v.into()), &tokens[1..])),
        &[Token::FNumber(v), ..] => Ok((
            Value::Number(Number::from_f64(v).context("Invalid float value")?),
            &tokens[1..],
        )),
        [Token::SqOpen, ..] => parse_list(tokens),
        [Token::CyOpen, ..] => parse_object(tokens, true),
        _ => unimplemented!(),
    }
}

fn is_parseable(token: Token<'_>) -> bool {
    matches!(
        token,
        Token::String(_) | Token::UNumber(_) | Token::SNumber(_) | Token::SqOpen | Token::CyOpen
    )
}

pub fn parse_object<'a>(
    token: &'a [Token<'a>],
    with_curly: bool,
) -> Result<(Value, &'a [Token<'a>]), anyhow::Error> {
    if matches!(token, [Token::CyOpen, ..]) || !with_curly {
        let mut remain = if matches!(token, [Token::CyOpen, ..]) && with_curly {
            &token[1..]
        } else {
            token
        };

        if let &[Token::CyClose, ..] = remain {
            Ok((Value::Object(Map::new()), &remain[1..]))
        } else {
            let mut res = Map::new();

            while let &[Token::Ident(ident), Token::Eq | Token::EqD, ..] = remain {
                remain = &remain[2..];

                let (value, r) = try_from(remain)?;

                res.insert(ident.to_string(), value)
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
                Ok((Value::Object(res), &remain[1..]))
            } else {
                Ok((Value::Object(res), remain))
            }
        }
    } else {
        Err(anyhow!("Token is not an object declaration"))
    }
}

fn parse_list<'a>(token: &'a [Token<'a>]) -> Result<(Value, &'a [Token<'a>]), anyhow::Error> {
    if let &[Token::SqOpen, ..] = token {
        let mut rem = &token[1..];

        if !rem.contains(&Token::SqClose) {
            bail!("Open list is not closed")
        }

        if let &[Token::SqClose, ..] = rem {
            Ok((Value::Array(Vec::new()), &rem[1..]))
        } else {
            let mut list = Vec::new();

            while let [next, ..] = rem {
                match (is_parseable(*next), rem) {
                    (true, _) => {
                        let (value, r) = try_from(rem)?;
                        list.push(value);
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

            Ok((Value::Array(list), rem))
        }
    } else {
        Err(anyhow!("Token is not a list"))
    }
}
