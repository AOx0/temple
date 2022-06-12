use crate::{word::Word, Indicators, Keys};
use smartstring::alias::String;
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

pub struct Contents {
    pub(crate) contents: Vec<u8>,
    pub(crate) origin: PathBuf,
}

pub enum NewContents<'a> {
    Old(&'a [u8]),
    New(Vec<u8>),
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
    pub fn from_file(path: PathBuf) -> Result<Contents, String> {
        let mut contents = vec![];
        let file = OpenOptions::new().read(true).open(&path);

        if let Ok(mut file) = file {
            if file.read_to_end(&mut contents).is_ok() {
                Ok(Contents {
                    contents,
                    origin: path,
                })
            } else {
                Err("Failed to read contents".into())
            }
        } else {
            Err("Failed to open file".into())
        }
    }

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
                NewContents::New(slice) => target.write_all(slice).unwrap(),
                NewContents::Old(slice) => target.write_all(slice).unwrap(),
            }
        }
    }
}

impl Contents {
    pub fn replace(
        &mut self,
        indicators: &Indicators,
        keys: &Keys,
    ) -> Result<(usize, Vec<NewContents>), String> {
        let mut result: Vec<NewContents> = Vec::with_capacity(self.contents.len());
        let mut i: usize = 0;
        let mut sum: usize = 0;
        let mut word = Word::new();
        let mut last_i: usize = 0;

        if !(indicators.find_in(&self.contents, i, true).is_some()
            && indicators.find_in(&self.contents, i, false).is_some())
        {
            return Ok((0, vec![NewContents::Old(&self.contents[..])]));
        }

        loop {
            if i >= self.contents.as_slice().len() {
                break;
            }

            if self.contents[i] == indicators.start_char() {
                if let Some(mut start) = indicators.find_in(&self.contents, i, true) {
                    if let Some(end) = indicators.find_in(&self.contents, i, false) {
                        result.push(NewContents::Old(&self.contents[last_i..start]));

                        start += indicators.start_size();

                        word.set(&self.contents.as_slice()[start..end], end - start);

                        let replacement = keys.get_match(
                            std::str::from_utf8(&word.contents[0..word.size]).unwrap(),
                            Some(&self.origin),
                        );

                        match replacement {
                            Ok(r) => result.push(NewContents::New(r.as_bytes().to_vec())),
                            Err(e) => return Err(e),
                        }

                        sum += 1;
                        i = end + indicators.end_size() - 1;
                        last_i = end + indicators.end_size();
                    }
                }
            }
            i += 1;
        }

        result.push(NewContents::Old(&self.contents[last_i..]));

        Ok((sum, result))
    }
}
