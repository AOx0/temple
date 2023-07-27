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
        let (mut i, mut last_i) = (0usize, 0usize);

        if !(indicators.find_in(&self.contents, i, true).is_some()
            && indicators.find_in(&self.contents, i, false).is_some())
        {
            return Ok(Replaced(vec![Cow::Borrowed(&self.contents[..])]));
        }

        while let (true, Some(mut start)) = (
            i < self.contents.len(),
            indicators.find_in(&self.contents, i, true),
        ) {
            if let Some(end) = indicators.find_in(&self.contents, i, false) {
                result.push(Cow::Borrowed(&self.contents[last_i..start]));

                start += indicators.start_size();

                let replacement = keys.get_match(&self.contents[start..end]);

                match replacement {
                    Some(r) => {
                        println!("Sustitución a {} de {}", r, &self.contents[start..end]);
                        result.push(Cow::Borrowed(r))
                    }
                    None => {
                        return Err(format!(
                            "No key {} found in {}",
                            &self.contents[start..end],
                            self.origin.display()
                        ))
                    }
                }

                i = end + indicators.end_size() - 1;
                last_i = end + indicators.end_size();
            }
            i += 1;
        }

        result.push(Cow::Borrowed(&self.contents[last_i..]));

        Ok(Replaced(result))
    }
}
