use std::{fs::OpenOptions, io::Read, path::Path, str::FromStr};

#[derive(Clone, Debug)]
pub struct Keys {
    pub list: Vec<(String, String)>,
    pub ignore_list: Vec<String>,
}

impl Keys {
    pub fn add(&mut self, mut other: Keys) {
        self.list.append(&mut other.list);
    }

    pub fn get_match(&self, key: &str) -> Option<&str> {
        self.list.iter().find(|a| a.0 == key).map(|a| a.1.as_str())
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
            let mut statement = statement.split('=');
            let to_push: (String, String) = (
                (statement.next().unwrap_or("")).into(),
                (statement.next().unwrap_or("")).into(),
            );

            if to_push.0 == empty_string || to_push.1 == empty_string {
                continue;
            }
            keys.list.push(to_push);
        }

        keys
    }
}
