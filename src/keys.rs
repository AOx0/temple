use std::{fs::OpenOptions, io::Read, path::Path};

#[derive(Clone, Debug)]
pub struct Keys {
    pub keys: Vec<String>,
    pub values: Vec<String>,
    pub ignore_list: Vec<String>,
}

impl Keys {
    pub fn add(&mut self, mut other: Keys) {
        self.keys.append(&mut other.keys);
        self.values.append(&mut other.values);
    }

    #[must_use]
    pub fn get_match(&self, key: &str) -> Option<&str> {
        self.keys
            .iter()
            .enumerate()
            .find_map(|(i, a)| (a.as_str() == key).then_some(i))
            .map(|i| self.values[i].as_str())
    }

    #[must_use]
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
            keys: vec![],
            values: vec![],
            ignore_list: vec![],
        };
        let no_space = string.replace('\n', "");
        for statement in no_space.split(',') {
            let mut statement = statement.split('=');
            let to_push: (String, String) = (
                (statement.next().unwrap_or("")).into(),
                (statement.next().unwrap_or("")).into(),
            );

            if to_push.0.is_empty() || to_push.1.is_empty() {
                continue;
            }
            keys.keys.push(to_push.0);
            keys.values.push(to_push.1);
        }

        keys
    }
}
