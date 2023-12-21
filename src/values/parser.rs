use std::collections::HashMap;

use anyhow::{anyhow, bail, ensure, Context, Result};
use tera::{Map, Number, Value};

use crate::values::{Type, TypeMap, ValueMap};

use super::token::Token;

pub fn try_value_from<'a>(
    tokens: &'a [Token<'a>],
) -> Result<(Value, &'a [Token<'a>]), anyhow::Error> {
    match tokens {
        &[Token::String(s), ..] => Ok((Value::String(s.to_string()), &tokens[1..])),
        &[Token::UNumber(v), ..] => Ok((Value::Number(v.into()), &tokens[1..])),
        &[Token::SNumber(v), ..] => Ok((Value::Number(v.into()), &tokens[1..])),
        &[Token::FNumber(v), ..] => Ok((
            Value::Number(Number::from_f64(v).context("Invalid float value")?),
            &tokens[1..],
        )),
        [Token::SqOpen, ..] => parse_list(tokens),
        [Token::CyOpen, ..] => parse_object(tokens),
        _ => unimplemented!(),
    }
}

pub fn try_type_from<'a>(
    tokens: &'a [Token<'a>],
) -> Result<(Type, &'a [Token<'a>]), anyhow::Error> {
    match tokens {
        [Token::KwAny, ..] => Ok((Type::Any, &tokens[1..])),
        [Token::KwNumber, ..] => Ok((Type::Number, &tokens[1..])),
        [Token::KwString, ..] => Ok((Type::String, &tokens[1..])),
        [Token::KwBool, ..] => Ok((Type::Bool, &tokens[1..])),
        [Token::KwArray, ..] => parse_array_type(tokens),
        [Token::KwObject, ..] => parse_object_type(tokens),
        _ => unimplemented!(),
    }
}

pub fn parse_object_type<'a>(
    token: &'a [Token<'a>],
) -> Result<(Type, &'a [Token<'a>]), anyhow::Error> {
    if let [Token::KwObject, Token::CyOpen, ..] = token {
        let mut remain = &token[2..];

        if let &[Token::CyClose, ..] = remain {
            Ok((Type::Object(HashMap::new()), &remain[1..]))
        } else {
            let mut res = HashMap::new();

            while let &[Token::Ident(ident), Token::EqD, ..] = remain {
                remain = &remain[2..];

                let (value, r) = try_type_from(remain)?;

                res.insert(ident.to_string(), value)
                    .is_some()
                    .then(|| println!("Warn: Ident `{ident}` is already defined, overriding"));

                remain = r;

                if let [Token::Comma, ..] = remain {
                    remain = &remain[1..];
                }
            }

            ensure! {
                matches!(remain, [Token::CyClose, ..]),
                "Warn: Invalid syntax on object declaration, remains: {remain:?}"
            };

            Ok((Type::Object(res), &remain[1..]))
        }
    } else {
        Err(anyhow!("Token is not an object type declaration"))
    }
}

pub fn parse_array_type<'a>(
    token: &'a [Token<'a>],
) -> Result<(Type, &'a [Token<'a>]), anyhow::Error> {
    if let [Token::KwArray, Token::SqOpen, ..] = token {
        let mut remains = &token[2..];

        let (typ, rem) = try_type_from(remains)?;

        remains = rem;

        ensure!(
            remains.last().context("Invalid syntax")? == &Token::SqClose,
            "Invalid syntax"
        );

        Ok((typ, &remains[1..]))
    } else {
        Err(anyhow!("Token is not an array type declaration"))
    }
}

pub fn parse_config<'a>(
    token: &'a [Token<'a>],
) -> Result<((ValueMap, TypeMap), &'a [Token<'a>]), anyhow::Error> {
    let mut remain = token;
    let mut values = ValueMap::default();
    let mut types = TypeMap::default();

    while !remain.is_empty() {
        match remain {
            &[Token::Ident(ident), Token::Eq | Token::EqD, ..] => {
                remain = &remain[1..];

                // Parse optional data type
                let typ = if matches!(remain, [Token::EqD, token, ..] if token.is_type_decl()) {
                    let (typ, r) = try_type_from(&remain[1..])?;
                    types.insert(ident.to_string(), typ);
                    remain = r;
                    Some(types.get(ident).expect("just pushed the value"))
                } else {
                    types.insert(ident.to_string(), Type::Any);
                    None
                };

                if let [Token::Eq, ..] = remain {
                    let (value, r) = try_value_from(&remain[1..])?;

                    values
                        .insert(ident.to_string(), value)
                        .is_some()
                        .then(|| println!("Warn: Ident `{ident}` is already defined, overriding"));

                    remain = r;
                } else {
                    values
                        .insert(ident.to_string(), Value::Null)
                        .is_some()
                        .then(|| println!("Warn: Ident `{ident}` is already defined, overriding"));

                    if typ.is_none() {
                        println!("Warn: We advise to state a data type for empty values");
                    }
                }

                if let [Token::Comma | Token::Semicolon, ..] = remain {
                    remain = &remain[1..];
                }
            }
            _ => {
                bail!("Invalid syntax, remains: {remain:?}")
            }
        }
    }

    ensure! {
        remain.is_empty(),
        "Warn: Invalid syntax on config, remains: {remain:?}"
    };

    Ok(((values, types), &remain))
}

pub fn parse_object<'a>(token: &'a [Token<'a>]) -> Result<(Value, &'a [Token<'a>]), anyhow::Error> {
    if let [Token::CyOpen, ..] = token {
        let mut remain = &token[1..];

        if let &[Token::CyClose, ..] = remain {
            Ok((Value::Object(Map::new()), &remain[1..]))
        } else {
            let mut res = Map::new();

            while let &[Token::Ident(ident), Token::Eq | Token::EqD, ..] = remain {
                remain = &remain[2..];

                let (value, r) = try_value_from(remain)?;

                res.insert(ident.to_string(), value)
                    .is_some()
                    .then(|| println!("Warn: Ident `{ident}` is already defined, overriding"));

                remain = r;

                if let [Token::Comma, ..] = remain {
                    remain = &remain[1..];
                }
            }

            ensure! {
                matches!(remain, [Token::CyClose, ..]),
                "Warn: Invalid syntax on object declaration, remains: {remain:?}"
            };

            Ok((Value::Object(res), &remain[1..]))
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
                match (next.is_expr_decl(), rem) {
                    (true, _) => {
                        let (value, r) = try_value_from(rem)?;
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
