mod parser;
mod token;

use anyhow::{anyhow, ensure};
use logos::Span;
use owo_colors::OwoColorize;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    path::Path,
};
use tera::{Map, Value};
use token::{Logos, TokenE};

use crate::values::token::{MessageType, Tokens};

#[derive(Debug, PartialEq, Eq)]
pub struct Values {
    pub value_map: ValueMap,
    pub type_map: TypeMap,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct ValueMap(HashMap<String, Value>);

#[derive(Debug, PartialEq, Eq, Default)]
pub struct TypeMap(HashMap<String, Type>);

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Number,
    String,
    Object(HashMap<String, Type>),
    Array(Box<Type>),
    Bool,
    Unknown,
    Any,
}

impl Type {
    /// Verify if a given value matches the current Type variant
    #[must_use]
    pub fn value_matches(&self, value: &Value) -> bool {
        if let Type::Any = self {
            true
        } else {
            &Type::from(value) == self
        }
    }

    #[must_use]
    pub fn is_equivalent(&self, other: &Type) -> bool {
        if &Type::Any == self || &Type::Any == other {
            true
        } else {
            self == other
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Number => write!(f, "Number"),
            Type::String => write!(f, "String"),
            Type::Object(fields) => {
                write!(f, "Object {{ ")?;

                for (i, (k, v)) in fields.iter().enumerate() {
                    write!(f, "{k}: {v}")?;
                    if i != fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, " }}")
            }
            Type::Array(t) => write!(f, "Array [ {t} ]"),
            Type::Bool => write!(f, "Bool"),
            Type::Unknown => write!(f, "Unknown"),
            Type::Any => write!(f, "Any"),
        }
    }
}

impl From<&Value> for Type {
    fn from(value: &Value) -> Self {
        match value {
            Value::Null => Type::Any,
            Value::Bool(_) => Type::Bool,
            Value::Number(_) => Type::Number,
            Value::String(_) => Type::String,
            Value::Array(a) => Type::Array(Box::new(match a.as_slice() {
                [] => Type::Any,
                [first] => Type::from(first),
                [first, ..] => {
                    for e in a {
                        assert!(Type::from(e).is_equivalent(&Type::from(first)));
                    }
                    Type::from(first)
                }
            })),
            Value::Object(o) => {
                let mut fields = HashMap::new();
                for (k, v) in o {
                    fields.insert(k.to_owned(), Type::from(v));
                }
                Type::Object(fields)
            }
        }
    }
}

impl Values {
    pub fn stash(&mut self, other: Map<String, Value>) {
        for (k, v) in other {
            //TODO: Stash data type
            self.value_map
                .insert(k, v)
                .iter()
                .for_each(|v| println!("Warn: Overriding value {v:?}"));
        }
    }

    pub fn verify_types(&self) -> anyhow::Result<()> {
        let mut res = Ok(());

        for (k, v) in self.value_map.iter() {
            let decl_type = self.type_map.get(k).expect("Missing value");
            let val_type = Type::from(v);

            if !decl_type.is_equivalent(&val_type) {
                eprintln!(
                    "{}: Data type for value {k:?} of type {decl_type:?} does not conform to the defined value {v:?} of type {val_type:?}",
                    "error".if_supports_color(owo_colors::Stream::Stdout, |s| s
                        .style(owo_colors::Style::new().bold().red()))
                );

                res = Err(anyhow!("Invalid configuration values/types"));
            }
        }

        res
    }
}

impl DerefMut for ValueMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for ValueMap {
    type Target = HashMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TypeMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for TypeMap {
    type Target = HashMap<String, Type>;

    fn deref(&self) -> &Self::Target {
        &self.0
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

fn line_col(inp: &str, span: Span) -> Location {
    let mut line = 1;
    let mut col = 1;
    for i in 0..span.start {
        if inp
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

impl Values {
    pub fn from_str(s: &str, path: &Path) -> std::result::Result<Self, anyhow::Error> {
        let mut tokens: Tokens<'_> = Tokens::new(s, format!("{}", path.display()));
        let mut lexer = TokenE::lexer(s);

        while let Some(token) = lexer.next() {
            let token = if let Err(()) = token {
                eprintln!(
                    "{}",
                    tokens.error_current_span(MessageType::Error, "Error reading token")
                );
                continue;
            } else {
                token.expect("Already matched error")
            };

            /*
            if let TokenE::Unknow(matched) = token {
                eprintln!(
                    "{}",
                    tokens.error_current_span(
                        MessageType::Error,
                        format!("Matched unknown token {matched}")
                    )
                );
                continue;
            } else
            */
            if let TokenE::Comment(text) = token {
                if std::env::var("TEMPLE_INFO").is_ok() || std::env::var("TEMPLE_TRACE").is_ok() {
                    println!(
                        "{}: Skipping comment '{text}'",
                        "tracing".if_supports_color(owo_colors::Stream::Stdout, |s| s
                            .style(owo_colors::Style::new().bold().bright_black()))
                    );
                }
                continue;
            }
            tokens.span.push(lexer.span());
            tokens.token.push(token);
        }

        let (value_map, type_map) = parser::parse_config(&mut tokens)?;

        ensure! {
            tokens.is_empty(),
            anyhow!(tokens.error_current_span(MessageType::Error, format!("Invalid syntax on config, expected to finish parsing everything but remains: {:?}", tokens.tokens())))
        };

        Ok(Values {
            value_map,
            type_map,
        })
    }
}

#[cfg(testasd)]
mod tests {
    use logos::Logos;
    use tera::Value;

    use super::{parser::try_value_from, token::TokenE};

    #[test]
    fn comma_ending_list() {
        let inp = "[1, 2, 3, ]";

        let tokens = TokenE::lexer(inp)
            .map(std::result::Result::unwrap)
            .collect::<Vec<_>>();

        assert_eq!(
            try_value_from(tokens.as_slice()).expect("This should never fail"),
            (
                Value::Array(vec![
                    Value::Number(1.into()),
                    Value::Number(2.into()),
                    Value::Number(3.into())
                ]),
                [].as_slice()
            )
        );
    }

    #[test]
    fn non_comma_ending_list() {
        let inp = "[1, 2, 3 ]";

        let tokens = TokenE::lexer(inp)
            .map(std::result::Result::unwrap)
            .collect::<Vec<_>>();

        assert_eq!(
            try_value_from(tokens.as_slice()).expect("This should never fail"),
            (
                Value::Array(vec![
                    Value::Number(1.into()),
                    Value::Number(2.into()),
                    Value::Number(3.into())
                ]),
                [].as_slice()
            )
        );
    }

    #[test]
    fn invalid_list() {
        let inp = "[1,,]";

        let tokens = TokenE::lexer(inp)
            .map(std::result::Result::unwrap)
            .collect::<Vec<_>>();

        assert_eq!(
            try_value_from(tokens.as_slice())
                .map_err(|e| { e.to_string() })
                .expect_err("This should never fail")
                .as_str(),
            "There must be a value between commas"
        );
    }
}
