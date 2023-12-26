use std::collections::HashMap;

use anyhow::{anyhow, bail, ensure, Context, Result};
use tera::{Map, Number, Value};

use crate::values::{Type, TypeMap, ValueMap};

use super::token::{TokenE, Tokens};

pub fn try_value_from(tokens: &mut Tokens<'_>) -> Result<Value, anyhow::Error> {
    match tokens
        .try_first()
        .context("Premature ending, expetced Value")?
        .token
    {
        TokenE::String(s) => {
            let s = s.to_string();
            tokens.step();
            Ok(Value::String(s))
        }
        TokenE::UNumber(v) => {
            tokens.step();
            Ok(Value::Number(v.into()))
        }
        TokenE::SNumber(v) => {
            tokens.step();
            Ok(Value::Number(v.into()))
        }
        TokenE::FNumber(v) => {
            tokens.step();
            Ok(Value::Number(
                Number::from_f64(v).context("Invalid float value")?,
            ))
        }
        TokenE::SqOpen => parse_list(tokens),
        TokenE::CyOpen => parse_object(tokens),
        _ => bail!(tokens.error_current_span(format!(
            "Found unexpected token {:?} while trying to parse value",
            tokens.try_first().map(|t| t.token)
        )),),
    }
}

pub fn try_type_from(tokens: &mut Tokens<'_>) -> Result<Type, anyhow::Error> {
    match tokens.tokens() {
        [TokenE::KwAny, ..] => {
            tokens.step();
            Ok(Type::Any)
        }
        [TokenE::KwNumber, ..] => {
            tokens.step();
            Ok(Type::Number)
        }
        [TokenE::KwString, ..] => {
            tokens.step();
            Ok(Type::String)
        }
        [TokenE::KwBool, ..] => {
            tokens.step();
            Ok(Type::Bool)
        }
        [TokenE::KwArray, ..] => parse_array_type(tokens.skiping(1)),
        [TokenE::KwObject, ..] => parse_object_type(tokens.skiping(1)),
        [TokenE::SqOpen, ..] => parse_array_type(tokens),
        [TokenE::CyOpen, ..] => parse_object_type(tokens),
        _ => bail!(tokens.error_current_span(format!(
            "Found unexpected token {:?} while trying to parse data type",
            tokens.try_first().map(|t| t.token)
        )),),
    }
}

pub fn parse_object_type(tokens: &mut Tokens<'_>) -> Result<Type, anyhow::Error> {
    if matches!(tokens.peek().token, TokenE::CyOpen) {
        tokens.step();

        if let &[TokenE::CyClose, ..] = tokens.tokens() {
            tokens.step();
            Ok(Type::Object(HashMap::new()))
        } else {
            let mut res = HashMap::new();

            while matches!(tokens.tokens(), [TokenE::Ident(_), TokenE::EqD, ..]) {
                let ident = tokens.get_ident().expect("Just matched");
                let value = try_type_from(tokens.skiping(2))?;

                res.insert(ident.to_string(), value).is_some().then(|| {
                    println!(
                        "{}",
                        tokens.error_current_span(format!(
                            "Warn: Ident `{ident}` is already defined, overriding"
                        )),
                    );
                });

                if let [TokenE::Comma, ..] = tokens.tokens() {
                    tokens.step();
                }
            }

            ensure!(
                tokens.peek().token == &TokenE::CyClose,
                anyhow!(tokens.error_current_span(format!(
                    "Expected closing '}}' in Object type declaration but found {:?}",
                    tokens.peek().token
                )),)
            );

            tokens.step();
            Ok(Type::Object(res))
        }
    } else {
        Err(anyhow!(tokens.error_current_span(
            "Token is not an object type declaration",
        ),))
    }
}

pub fn parse_array_type(tokens: &mut Tokens<'_>) -> Result<Type, anyhow::Error> {
    if let [TokenE::SqOpen, ..] = tokens.tokens() {
        tokens.step();

        let typ = try_type_from(tokens)?;

        ensure!(
            tokens.peek().token == &TokenE::SqClose,
            anyhow!(tokens
                .error_current_span("Expected ']' after type {typ} in Array type declaration"))
        );

        tokens.step();
        Ok(Type::Array(Box::new(typ)))
    } else {
        Err(anyhow!(tokens.error_current_span(
            "Expected '[' before type Array type declaration",
        ),))
    }
}

pub fn parse_config(tokens: &mut Tokens<'_>) -> Result<(ValueMap, TypeMap), anyhow::Error> {
    let mut values = ValueMap::default();
    let mut types = TypeMap::default();

    while !tokens.is_empty() {
        match tokens.tokens() {
            [TokenE::Ident(_), TokenE::Eq | TokenE::EqD, ..] => {
                let ident = tokens.get_ident().expect("Just matched it");
                tokens.step();

                // Parse optional data type
                let typ = if matches!(tokens.tokens(), [TokenE::EqD, token, ..] if token.is_type_decl())
                {
                    tokens.step();
                    let typ = try_type_from(tokens)?;
                    types.insert(ident.to_string(), typ);

                    Some(types.get(ident.as_str()).expect("just pushed the value"))
                } else {
                    types.insert(ident.to_string(), Type::Any);
                    None
                };

                if let [TokenE::Eq, ..] = tokens.tokens() {
                    let value = try_value_from(tokens.skiping(1))?;

                    values.insert(ident.to_string(), value).is_some().then(|| {
                        println!(
                            "{}",
                            tokens.error_current_span(format!(
                                "Warn: Ident `{ident}` is already defined, overriding",
                            )),
                        );
                    });
                } else {
                    values
                        .insert(ident.to_string(), Value::Null)
                        .is_some()
                        .then(|| {
                            println!(
                                "{}",
                                tokens.error_current_span(format!(
                                    "Warn: Ident `{ident}` is already defined, overriding",
                                )),
                            );
                        });

                    if typ.is_none() {
                        println!(
                            "{}",
                            tokens.error_current_span(format!(
                                "Warn: We advise to state a data type for empty values",
                            )),
                        );
                    }
                }

                if let [TokenE::Comma | TokenE::Semicolon, ..] = tokens.tokens() {
                    tokens.step();
                }
            }
            &[token, ..] => {
                bail!(tokens.error_current_span(format!("Found unexpected token {:?}", token)),)
            }
            _ => {
                bail!(tokens.error_current_span("Expected token, found nothing",),)
            }
        }
    }

    ensure! {
        tokens.is_empty(),
        anyhow!(
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
    if let [TokenE::CyOpen, ..] = tokens.tokens() {
        tokens.step();

        if let &[TokenE::CyClose, ..] = tokens.tokens() {
            tokens.step();
            Ok(Value::Object(Map::new()))
        } else {
            let mut res = Map::new();

            while let &[TokenE::Ident(_), TokenE::Eq | TokenE::EqD, ..] = tokens.tokens() {
                let ident = tokens.get_ident().expect("Just matched");
                let value = try_value_from(tokens.skiping(2))?;

                res.insert(ident.to_string(), value)
                    .is_some()
                    .then(|| println!("Warn: Ident `{ident}` is already defined, overriding"));

                if let [TokenE::Comma, ..] = tokens.tokens() {
                    tokens.step();
                }
            }

            ensure!(
                tokens.peek().token == &TokenE::CyClose,
                anyhow!(tokens.error_current_span(format!(
                    "Expected closing '}}' in Object declaration but found {:?}",
                    tokens.peek().token
                )),)
            );

            tokens.step();

            Ok(Value::Object(res))
        }
    } else {
        Err(anyhow!(
            tokens.error_current_span("Token is not an object declaration",),
        ))
    }
}

fn parse_list(tokens: &mut Tokens<'_>) -> Result<Value, anyhow::Error> {
    if let &[TokenE::SqOpen, ..] = tokens.tokens() {
        tokens.step();

        if !tokens.tokens().contains(&TokenE::SqClose) {
            bail!(tokens.error_current_span("Array not closed",),)
        }

        if let &[TokenE::SqClose, ..] = tokens.tokens() {
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
                    (false, [TokenE::Comma, TokenE::Comma, ..]) => {
                        bail!(tokens.error_current_span("There must be a value between commas",),)
                    }
                    (false, [TokenE::SqClose, ..]) => {
                        tokens.step();
                        break;
                    }
                    (false, [TokenE::Comma, ..]) => tokens.step(),
                    (false, [token, ..]) => {
                        bail!(tokens
                            .error_current_span(format!("Invalid token in list: `{token:?}`",)),)
                    }
                    _ => unreachable!("We did check the list is not empty"),
                }
            }

            if let Some(TokenE::Comma) = tokens.try_first().map(|e| e.token) {
                tokens.step();
            }

            Ok(Value::Array(list))
        }
    } else {
        Err(anyhow!(tokens.error_current_span("Token is not a list",),))
    }
}
