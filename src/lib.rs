use crate::NewContents::{New, Old};
use smartstring::alias::String;
use std::fmt::{Display, Formatter};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub struct Keys {
    pub list: Vec<(String, String)>,
}

impl Keys {
    pub fn add(&mut self, mut other: Keys) {
        self.list.append(&mut other.list);
    }

    pub fn get_match(&self, key: &str, file: &Path) -> Result<&str, String> {
        for i in 0..self.list.len() {
            if self.list[i].0 == key {
                return Ok(&self.list[i].1);
            }
        }

        Err(format!(
            "No value found for key \"{0}\" in file {1}.\nSet it:\n\
         \t1. In .temple_conf as {0}=value;\n\
         \t2. In .temple/template/.temple as {0}=value\n\
         \t3. As argument:  `temple new template new_project {0}=value`",
            key,
            file.display()
        )
        .into())
    }
}

impl From<&str> for Keys {
    fn from(string: &str) -> Keys {
        let mut keys = Keys { list: vec![] };
        let no_space = string.replace('\n', "");
        let empty_string = String::from_str("").unwrap();
        for statement in no_space.split(',') {
            let statement: Vec<&str> = statement.split('=').collect();
            let to_push: (String, String) = (
                statement.get(0).unwrap_or(&"").deref().into(),
                statement.get(1).unwrap_or(&"").deref().into(),
            );

            if to_push.0 == empty_string || to_push.1 == empty_string {
                continue;
            } else {
                keys.list.push(to_push);
            }
        }

        keys
    }
}

pub struct Contents {
    contents: Vec<u8>,
    origin: PathBuf,
}

impl Contents {
    pub fn from_file(path: PathBuf) -> Result<Contents, &'static str> {
        let mut contents = vec![];
        let file = OpenOptions::new().read(true).open(&path);

        if let Ok(mut file) = file {
            if file.read_to_end(&mut contents).is_ok() {
                Ok(Contents {
                    contents,
                    origin: path,
                })
            } else {
                Err("Failed to read contents")
            }
        } else {
            Err("Failed to open file")
        }
    }
}

impl From<&str> for Contents {
    fn from(s: &str) -> Self {
        let contents = s.as_bytes().to_vec();
        Contents {
            contents,
            origin: PathBuf::from("None. Contents from &str"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Indicator(u8, u8, u8, bool);

impl Indicator {
    pub fn from(string: &str, is_start: bool) -> Result<Indicator, &str> {
        if string.len() != 3 {
            Err("Len must be 3")
        } else {
            let bytes = string.as_bytes();
            Ok(Indicator(bytes[0], bytes[1], bytes[2], is_start))
        }
    }
}

pub enum NewContents<'a> {
    Old(&'a [u8]),
    New(Vec<u8>),
}

pub trait Parse {
    fn find_indicator(slice: &[u8], from: usize, indicator: Indicator) -> Option<usize>;
    fn replace(
        &mut self,
        start_indicator: Indicator,
        end_indicator: Indicator,
        keys: &Keys,
    ) -> Result<(usize, Vec<NewContents>), String>;
}

impl<'a> Contents {
    pub fn get_str_from_result(result: &[NewContents]) -> String {
        let mut f_result = String::new();

        for r in result.iter() {
            match r {
                Old(slice) => f_result.push_str(std::str::from_utf8(slice).unwrap()),
                New(slice) => f_result.push_str(std::str::from_utf8(slice).unwrap()),
            }
        }

        f_result
    }

    pub fn write_to_target(result: &[NewContents], mut target: std::fs::File) {
        for r in result.iter() {
            match r {
                Old(slice) => target.write_all(slice).unwrap(),
                New(slice) => target.write_all(slice).unwrap(),
            }
        }
    }
}

impl Parse for Contents {
    fn find_indicator(slice: &[u8], from: usize, indicator: Indicator) -> Option<usize> {
        if slice.is_empty() || slice.len() < 6 {
            return None;
        };
        for i in from..slice.len() - if indicator.3 { 3 } else { 0 } {
            let byte = slice[i];
            if byte == indicator.0 && slice[i + 1] == indicator.1 && slice[i + 2] == indicator.2 {
                return Some(i);
            }
        }

        None
    }

    fn replace(
        &mut self,
        start_indicator: Indicator,
        end_indicator: Indicator,
        keys: &Keys,
    ) -> Result<(usize, Vec<NewContents>), String> {
        let mut result: Vec<NewContents> = Vec::with_capacity(self.contents.len());
        let mut i: usize = 0;
        let mut sum: usize = 0;
        let mut word = Word::new();
        let mut last_i: usize = 0;

        if !(Self::find_indicator(&self.contents, i, start_indicator).is_some()
            && Self::find_indicator(&self.contents, i, end_indicator).is_some())
        {
            return Ok((0, vec![Old(&self.contents[..])]));
        }

        loop {
            if i >= self.contents.as_slice().len() {
                break;
            }
            if self.contents[i] == start_indicator.0 {
                if let Some(mut some_start) =
                    Self::find_indicator(&self.contents, i, start_indicator)
                {
                    if let Some(some_end) = Self::find_indicator(&self.contents, i, end_indicator) {
                        result.push(Old(&self.contents[last_i..some_start]));

                        some_start += 3;

                        word.set(
                            &self.contents.as_slice()[some_start..some_end],
                            some_end - some_start,
                        );

                        let replacement = keys.get_match(
                            std::str::from_utf8(&word.contents[0..word.size]).unwrap(),
                            &self.origin,
                        );

                        match replacement {
                            Ok(r) => result.push(New(r.as_bytes().to_vec())),
                            Err(e) => return Err(e),
                        }

                        sum += 1;
                        i = some_end + 2;
                        last_i = some_end + 3;
                    }
                }
            }
            i += 1;
        }

        result.push(Old(&self.contents[last_i..]));

        Ok((sum, result))
    }
}

#[derive(Clone, Copy)]
struct Word {
    contents: [u8; 100],
    size: usize,
}

impl Display for Word {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {})",
            String::from_str(std::str::from_utf8(&self.contents[0..self.size]).unwrap()).unwrap(),
            self.size
        )
    }
}

impl Word {
    fn new() -> Word {
        Word {
            contents: [0u8; 100],
            size: 0usize,
        }
    }

    #[allow(unused)]
    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.contents[0..self.size]).unwrap()
    }

    fn set(&mut self, slice: &[u8], size: usize) {
        for (i, &byte) in slice.iter().enumerate() {
            self.contents[i] = byte;
        }

        self.size = size;
    }
}
