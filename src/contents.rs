use crate::{Indicators, Keys};
use std::{
    borrow::Cow,
    fmt::Write,
    fs::OpenOptions,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

pub struct Contents {
    pub(crate) contents: String,
    pub(crate) origin: PathBuf,
}

impl FromStr for Contents {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Contents {
            contents: s.to_owned(),
            origin: PathBuf::new(),
        })
    }
}

pub struct Replaced<'a>(Vec<Cow<'a, str>>);

impl Replaced<'_> {
    pub fn write_to_file(&self, mut target: impl std::io::Write) {
        for r in self.get_iter() {
            match r {
                Cow::Owned(slice) => target.write_all(slice.as_bytes()).unwrap(),
                Cow::Borrowed(slice) => target.write_all(slice.as_bytes()).unwrap(),
            }
        }
    }

    pub fn get_iter(&self) -> std::slice::Iter<'_, Cow<'_, str>> {
        self.0.iter()
    }

    pub fn extend_str(&self, t: &mut String) {
        t.clear();
        for r in self.get_iter() {
            match r {
                Cow::Owned(slice) => t.write_str(slice).unwrap(),
                Cow::Borrowed(slice) => t.write_str(slice).unwrap(),
            }
        }
    }

    pub fn get_string(self) -> String {
        self.0.concat()
    }
}

impl Contents {
    pub fn from_file(path: &Path) -> Result<Contents, String> {
        OpenOptions::new()
            .read(true)
            .open(path)
            .map(|mut f| {
                let mut contents = String::new();
                f.read_to_string(&mut contents)
                    .is_ok()
                    .then_some(Contents {
                        contents,
                        origin: path.to_path_buf(),
                    })
                    .ok_or(format!("Failed to read from file {}", path.display()))
            })
            .map_err(|err| format!("{}", err))?
    }
}

impl Contents {
    pub fn replace<'a, 'b>(
        &'a mut self,
        indicators: Indicators<'b>,
        keys: &'a Keys,
    ) -> Result<Replaced<'a>, String> {
        let mut result: Vec<Cow<'a, str>> = Vec::with_capacity(self.contents.len());
        let mut last_i = 0;

        if !(indicators.find_start(&self.contents, 0).is_some()
            && indicators.find_end(&self.contents, 0).is_some())
        {
            return Ok(Replaced(vec![Cow::Borrowed(&self.contents[..])]));
        }

        while let Some(mut start) = indicators.find_start(&self.contents, last_i) {
            start = last_i + start;
            if let Some(end) = indicators.find_end(&self.contents, start) {
                result.push(Cow::Borrowed(&self.contents[last_i..start]));

                let key = &self.contents[start + indicators.start_size()..start + end].trim();

                let replacement = keys.get_match(key);

                match replacement {
                    Some(r) => result.push(Cow::Borrowed(r)),
                    None => return Err(format!("No key {key} found in {}", self.origin.display())),
                }

                last_i = start + end + indicators.end_size();
            }
        }

        result.push(Cow::Borrowed(&self.contents[last_i..]));

        Ok(Replaced(result))
    }
}
