use smartstring::alias::String;
use std::fs::OpenOptions;
use std::io::Read;
use std::ops::Deref;
use std::path::Path;
use std::str::FromStr;

pub struct Keys {
    pub list: Vec<(String, String)>,
    pub ignore_list: Vec<String>,
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

    pub fn from_file_contents(path: &Path) -> Keys {
        let mut file = OpenOptions::new().read(true).open(path).unwrap();
        let mut file_contents: Vec<u8> = Vec::new();
        file.read_to_end(&mut file_contents).unwrap();
        Keys::from(std::str::from_utf8(&file_contents).unwrap())
    }
}

impl From<&str> for Keys {
    fn from(string: &str) -> Keys {
        let mut keys = Keys {
            list: vec![],
            ignore_list: vec![],
        };
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
