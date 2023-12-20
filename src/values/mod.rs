mod parser;
mod token;

use anyhow::{anyhow, ensure};
use std::ops::{Deref, DerefMut};
use tera::{Map, Value};
use token::{Logos, Token};

#[derive(Debug, PartialEq, Eq)]
pub struct Values(pub Map<String, Value>);

impl Values {
    pub fn stash(&mut self, other: Map<String, Value>) {
        for (k, v) in other {
            self.insert(k, v)
                .iter()
                .for_each(|v| println!("Warn: Overriding value {v:?}"));
        }
    }
}

impl DerefMut for Values {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for Values {
    type Target = Map<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::str::FromStr for Values {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let tokens = Token::lexer(s)
            .collect::<std::result::Result<Vec<Token<'_>>, _>>()
            .map_err(|()| anyhow!("Failed to get tokens"))?;

        if let (Value::Object(config), remain) = parser::parse_object(&tokens, false)? {
            ensure!(
                remain.is_empty(),
                "Error while parsing config, bad syntax, ramains: {remain:?}"
            );

            Ok(Values(config))
        } else {
            unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
    use logos::Logos;
    use tera::Value;

    use super::{parser::try_from, token::Token};

    #[test]
    fn comma_ending_list() {
        let inp = "[1, 2, 3, ]";

        let tokens = Token::lexer(inp)
            .map(std::result::Result::unwrap)
            .collect::<Vec<_>>();

        assert_eq!(
            try_from(tokens.as_slice()).expect("This should never fail"),
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

        let tokens = Token::lexer(inp)
            .map(std::result::Result::unwrap)
            .collect::<Vec<_>>();

        assert_eq!(
            try_from(tokens.as_slice()).expect("This should never fail"),
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

        let tokens = Token::lexer(inp)
            .map(std::result::Result::unwrap)
            .collect::<Vec<_>>();

        assert_eq!(
            try_from(tokens.as_slice())
                .map_err(|e| { e.to_string() })
                .expect_err("This should never fail")
                .as_str(),
            "There must be a value between commas"
        );
    }
}
