mod parser;
mod token;

use anyhow::{anyhow, ensure};
use std::{
    collections::{hash_map::Entry, HashMap},
    ops::{Deref, DerefMut},
    path::Path,
};
use tera::{Map, Value};
use token::{Logos, Variant};

use crate::{values::token::Tokens, warn};

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Values {
    pub value_map: ValueMap,
    pub type_map: TypeMap,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct ValueMap(HashMap<String, Value>);

#[derive(Debug, PartialEq, Eq, Default)]
pub struct TypeMap(HashMap<String, Type>);

#[derive(Debug, PartialEq, Eq, Clone)]
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
    #[must_use]
    pub fn is_equivalent(&self, other: &Type) -> bool {
        if &Type::Any == self {
            true
        } else {
            self == other
        }
    }

    #[must_use]
    pub fn is_equivalent_or_empty(&self, other: &Type) -> bool {
        self.is_equivalent(other) || other == &Type::Unknown
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

impl Type {
    fn from(value: &Value, decl_type: &Type) -> Self {
        let res = match value {
            Value::Null => decl_type.clone(),
            Value::Bool(_) => Type::Bool,
            Value::Number(_) => Type::Number,
            Value::String(_) => Type::String,
            Value::Array(a) => Type::Array(Box::new(match a.as_slice() {
                [] => {
                    if let Type::Array(decl_a) = decl_type {
                        decl_a.deref().clone()
                    } else {
                        Type::Unknown
                    }
                }
                [first] => Type::from(
                    first,
                    if let Type::Array(decl) = decl_type {
                        decl
                    } else {
                        &Type::Unknown
                    },
                ),
                [first, ..] => {
                    let inner_decl = if let Type::Array(decl) = decl_type {
                        decl
                    } else {
                        &Type::Unknown
                    };
                    for e in a {
                        if !Type::from(e, inner_decl).is_equivalent(&Type::from(first, inner_decl))
                        {
                            warn!("Not all values in array have the same value. Treating as an Array [ Any ] for {}", value);
                            return Type::Array(Box::new(Type::Any));
                        }
                    }
                    Type::from(first, inner_decl)
                }
            })),
            Value::Object(o) => {
                let mut fields = HashMap::new();
                let inner_decl = if let Type::Object(decl) = decl_type {
                    Some(decl)
                } else {
                    None
                };

                for (k, v) in o {
                    fields.insert(
                        k.to_owned(),
                        Type::from(
                            v,
                            inner_decl.and_then(|m| m.get(k)).unwrap_or(&Type::Unknown),
                        ),
                    );
                }
                Type::Object(fields)
            }
        };

        crate::trace!(
            "Type::from with_hint={}: {value} has type {res}",
            if matches!(decl_type, Type::Unknown) {
                "no"
            } else {
                "yes"
            }
        );

        res
    }
}

impl Values {
    #[must_use]
    pub fn stash(mut self, other: Self) -> Self {
        for (k, v) in other.value_map.0 {
            self.value_map
                .insert(k, v)
                .iter()
                .for_each(|v| warn!("Overriding value with {:?}", v));
        }

        for (k, v) in other.type_map.0 {
            if let Entry::Vacant(e) = self.type_map.entry(k.clone()) {
                e.insert(v);
            } else {
                warn!("Using previously defined data type for value {}", k);
            }
        }

        self
    }

    pub fn verify_types(&self) -> anyhow::Result<()> {
        let mut res = Ok(());

        for (k, v) in self.value_map.iter() {
            let decl_type = self.type_map.get(k).expect("Missing value");
            let val_type = Type::from(v, decl_type);

            crate::trace!("Decl type of '{k}' is {decl_type}");
            crate::trace!("Real type of '{k}' is {val_type}");

            if !decl_type.is_equivalent_or_empty(&val_type) {
                crate::error!(
                    "The value of '{k}' does not match with the declared type\n    Value: {v}\n    Decl type: {decl_type}\n    Real type: {val_type}",
                    v = format!("{v:#}").replace('\n', "\n    ")
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

impl Values {
    pub fn from_str(s: &str, path: &Path) -> std::result::Result<Self, anyhow::Error> {
        let mut tokens: Tokens<'_> = Tokens::new(s, format!("{}", path.display()));
        let mut lexer = Variant::lexer(s);

        while let Some(token) = lexer.next() {
            let token = if let Err(()) = token {
                crate::error!(tokens.error_current_span("Error reading token"));
                continue;
            } else {
                token.expect("Already matched error")
            };

            if let Variant::Comment(text) = token {
                crate::trace!("Skipping comment '{text}'",);
                continue;
            }
            tokens.span.push(lexer.span());
            tokens.token.push(token);
        }

        let (value_map, type_map) = parser::parse_config(&mut tokens)?;

        ensure! {
            tokens.is_empty(),
            anyhow!(tokens.error_current_span(format!("Invalid syntax on config, expected to finish parsing everything but remains: {:?}", tokens.tokens())))
        };

        Ok(Values {
            value_map,
            type_map,
        })
    }
}
