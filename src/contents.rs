use crate::{Indicator, Keys, Word};
use smartstring::alias::String;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;

pub enum NewContents<'a> {
    Old(&'a [u8]),
    New(Vec<u8>),
}

pub struct Contents {
    pub(crate) contents: Vec<u8>,
    pub(crate) origin: PathBuf,
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

impl<'a> Contents {
    pub fn get_str_from_result(result: &[NewContents]) -> String {
        let mut f_result = String::new();

        for r in result.iter() {
            match r {
                NewContents::Old(slice) => f_result.push_str(std::str::from_utf8(slice).unwrap()),
                NewContents::New(slice) => f_result.push_str(std::str::from_utf8(slice).unwrap()),
            }
        }

        f_result
    }

    pub fn write_to_target(result: &[NewContents], mut target: fs::File) {
        for r in result.iter() {
            match r {
                NewContents::Old(slice) => target.write_all(slice).unwrap(),
                NewContents::New(slice) => target.write_all(slice).unwrap(),
            }
        }
    }
}

impl crate::Parse for Contents {
    fn find_indicator(slice: &[u8], from: usize, indicator: &Indicator) -> Option<usize> {
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
        start_indicator: &Indicator,
        end_indicator: &Indicator,
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
            return Ok((0, vec![NewContents::Old(&self.contents[..])]));
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
                        result.push(NewContents::Old(&self.contents[last_i..some_start]));

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
                            Ok(r) => result.push(NewContents::New(r.as_bytes().to_vec())),
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

        result.push(NewContents::Old(&self.contents[last_i..]));

        Ok((sum, result))
    }
}
