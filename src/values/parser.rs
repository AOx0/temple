use std::collections::HashMap;

use anyhow::{anyhow, bail, ensure, Context, Result};
use tera::{Map, Number, Value};

use crate::{
    ferror,
    values::{Type, TypeMap, ValueMap},
    warn,
};

use super::token::{Tokens, Variant};

pub fn try_value_from(tokens: &mut Tokens<'_>) -> Result<Value, anyhow::Error> {
    match tokens.tokens() {
        &[Variant::String(s), ..] => {
            let s = s.to_string();
            tokens.step();
            Ok(Value::String(s))
        }
        &[Variant::UNumber(v), ..] => {
            tokens.step();
            Ok(Value::Number(v.into()))
        }
        &[Variant::SNumber(v), ..] => {
            tokens.step();
            Ok(Value::Number(v.into()))
        }
        &[Variant::FNumber(v), ..] => {
            tokens.step();
            Ok(Value::Number(
                Number::from_f64(v).context("Invalid float value")?,
            ))
        }
        [Variant::SqOpen, ..] => parse_list(tokens),
        [Variant::CyOpen, ..] => parse_object(tokens),
        _ => bail!(ferror!(
            "{}",
            tokens.error_current_span(format!(
                "Found unexpected token {:?} while trying to parse value",
                tokens.try_first().map(|t| t.token)
            ))
        )),
    }
}

macro_rules! berr {
    ($($arg:tt)+) => {
        bail!(crate::ferror!("{}", $($arg)*))
    }
}

macro_rules! aerr {
    ($($arg:tt)+) => {
        anyhow!(crate::ferror!("{}", $($arg)*))
    }
}

pub fn try_type_from(tokens: &mut Tokens<'_>) -> Result<Type, anyhow::Error> {
    match tokens.tokens() {
        [Variant::KwAny, ..] => {
            tokens.step();
            Ok(Type::Any)
        }
        [Variant::KwNumber, ..] => {
            tokens.step();
            Ok(Type::Number)
        }
        [Variant::KwString, ..] => {
            tokens.step();
            Ok(Type::String)
        }
        [Variant::KwBool, ..] => {
            tokens.step();
            Ok(Type::Bool)
        }
        [Variant::KwArray, ..] => parse_array_type(tokens.skiping(1)),
        [Variant::KwObject, ..] => parse_object_type(tokens.skiping(1)),
        [Variant::SqOpen, ..] => parse_array_type(tokens),
        [Variant::CyOpen, ..] => parse_object_type(tokens),
        _ => berr!(tokens.error_current_span(format!(
            "Found unexpected token {:?} while trying to parse data type",
            tokens.try_first().map(|t| t.token)
        ))),
    }
}

pub fn parse_object_type(tokens: &mut Tokens<'_>) -> Result<Type, anyhow::Error> {
    if matches!(tokens.peek().token, Variant::CyOpen) {
        tokens.step();

        if let &[Variant::CyClose, ..] = tokens.tokens() {
            tokens.step();
            Ok(Type::Object(HashMap::new()))
        } else {
            let mut res = HashMap::new();

            while matches!(tokens.tokens(), [Variant::Ident(_), Variant::EqD, ..]) {
                let loc = tokens.current_location();
                let ident = tokens.get_ident().expect("Just matched");
                let value = try_type_from(tokens.skiping(2))?;

                res.insert(ident.to_string(), value).is_some().then(|| {
                    warn!(tokens.error_at(
                        loc,
                        format!("Ident `{ident}` is already defined, overriding")
                    ));
                });

                if let [Variant::Comma, ..] = tokens.tokens() {
                    tokens.step();
                }
            }

            ensure!(
                tokens.peek().token == &Variant::CyClose,
                aerr!(tokens.error_current_span(format!(
                    "Expected closing '}}' in Object type declaration but found {:?}",
                    tokens.peek().token
                )))
            );

            tokens.step();
            Ok(Type::Object(res))
        }
    } else {
        Err(aerr!(tokens.error_current_span(
            "Token is not an object type declaration",
        )))
    }
}

pub fn parse_array_type(tokens: &mut Tokens<'_>) -> Result<Type, anyhow::Error> {
    if let [Variant::SqOpen, ..] = tokens.tokens() {
        tokens.step();

        let typ = try_type_from(tokens)?;

        ensure!(
            tokens.peek().token == &Variant::SqClose,
            aerr!(tokens
                .error_current_span("Expected ']' after type {typ} in Array type declaration"))
        );

        tokens.step();
        Ok(Type::Array(Box::new(typ)))
    } else {
        Err(aerr!(tokens.error_current_span(
            "Expected '[' before type Array type declaration",
        )))
    }
}

pub fn parse_config(tokens: &mut Tokens<'_>) -> Result<(ValueMap, TypeMap), anyhow::Error> {
    let mut values = ValueMap::default();
    let mut types = TypeMap::default();

    while !tokens.is_empty() {
        match tokens.tokens() {
            [Variant::Ident(_), Variant::Eq | Variant::EqD, ..] => {
                let loc = tokens.current_location();
                let ident = tokens.get_ident().expect("Just matched it");
                tokens.step();

                // Parse optional data type
                let typ = if matches!(tokens.tokens(), [Variant::EqD, token, ..] if token.is_type_decl())
                {
                    let typ = try_type_from(tokens.skiping(1))?;
                    types.insert(ident.to_string(), typ);

                    Some(types.get(ident.as_str()).expect("just pushed the value"))
                } else if let [Variant::EqD, token, ..] = tokens.tokens() {
                    berr!(tokens.error_current_span(format!(
                        "Unexpected token {token} ({token:?}) following ':' in type declaration"
                    )))
                } else {
                    types.insert(ident.to_string(), Type::Any);
                    None
                };

                if let [Variant::Eq, ..] = tokens.tokens() {
                    let value = try_value_from(tokens.skiping(1))?;

                    values.insert(ident.to_string(), value).is_some().then(|| {
                        warn!(tokens.error_at(
                            loc,
                            format!("Ident `{ident}` is already defined, overriding")
                        ));
                    });
                } else {
                    values
                        .insert(ident.to_string(), Value::Null)
                        .is_some()
                        .then(|| {
                            warn!(tokens.error_at(
                                loc,
                                format!("Ident `{ident}` is already defined, overriding")
                            ));
                        });

                    if typ.is_none() {
                        warn!(tokens
                            .error_current_span("We advise to state a data type for empty values"));
                    }
                }

                if let [Variant::Comma | Variant::Semicolon, ..] = tokens.tokens() {
                    tokens.step();
                }
            }
            &[token, ..] => {
                berr!(tokens.error_current_span(format!("Found unexpected token {token:?}")))
            }
            _ => {
                berr!(tokens.error_current_span("Expected token, found nothing"),)
            }
        }
    }

    ensure! {
        tokens.is_empty(),
        aerr!(
            tokens.error_current_span(
                format!("Invalid syntax on config, expected to finish parsing everything but remains: {:?}",
                    tokens.tokens()
                )
            )
        ),
    };

    Ok((values, types))
}

pub fn parse_object(tokens: &mut Tokens<'_>) -> Result<Value, anyhow::Error> {
    if let [Variant::CyOpen, ..] = tokens.tokens() {
        tokens.step();

        if let &[Variant::CyClose, ..] = tokens.tokens() {
            tokens.step();
            Ok(Value::Object(Map::new()))
        } else {
            let mut res = Map::new();

            while let &[Variant::Ident(_), Variant::Eq | Variant::EqD, ..] = tokens.tokens() {
                let loc = tokens.current_location();
                let ident = tokens.get_ident().expect("Just matched");
                let value = try_value_from(tokens.skiping(2))?;

                res.insert(ident.to_string(), value).is_some().then(|| {
                    warn!(tokens.error_at(
                        loc,
                        format!("Ident `{ident}` is already defined, overriding")
                    ));
                });

                if let [Variant::Comma, ..] = tokens.tokens() {
                    tokens.step();
                }
            }

            ensure!(
                tokens.peek().token == &Variant::CyClose,
                aerr!(tokens.error_current_span(format!(
                    "Expected closing '}}' in Object declaration but found {:?}",
                    tokens.peek().token
                )))
            );

            tokens.step();

            Ok(Value::Object(res))
        }
    } else {
        Err(aerr!(
            tokens.error_current_span("Token is not an object declaration")
        ))
    }
}

fn parse_list(tokens: &mut Tokens<'_>) -> Result<Value, anyhow::Error> {
    if let &[Variant::SqOpen, ..] = tokens.tokens() {
        tokens.step();

        if !tokens.tokens().contains(&Variant::SqClose) {
            berr!(tokens.error_current_span("Array not closed"))
        }

        if let &[Variant::SqClose, ..] = tokens.tokens() {
            tokens.step();
            Ok(Value::Array(Vec::new()))
        } else {
            let mut list = Vec::new();

            while let [next, ..] = tokens.tokens() {
                match (next.is_expr_decl(), tokens.tokens()) {
                    (true, _) => {
                        let value = try_value_from(tokens)?;
                        list.push(value);
                    }
                    (false, [Variant::Comma, Variant::Comma, ..]) => {
                        berr!(tokens.error_current_span("There must be a value between commas"))
                    }
                    (false, [Variant::SqClose, ..]) => {
                        tokens.step();
                        break;
                    }
                    (false, [Variant::Comma, ..]) => tokens.step(),
                    (false, [token, ..]) => {
                        berr!(tokens.error_current_span(format!(
                            "Invalid token in list declaration: `{token:?}`"
                        )))
                    }
                    (_, []) => unreachable!("We did check the list is not empty"),
                }
            }

            if let Some(Variant::Comma) = tokens.try_first().map(|e| e.token) {
                tokens.step();
            }

            Ok(Value::Array(list))
        }
    } else {
        Err(aerr!(tokens.error_current_span("Token is not a list",)))
    }
}
