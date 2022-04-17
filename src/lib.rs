use crate::NewContents::{New, Old};
pub use clap::{Parser, Subcommand};
use fs_extra::dir::create_all;
use lazy_static::lazy_static;
use smartstring::alias::String;
use std::env::current_dir;
use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::{create_dir_all, OpenOptions};
use std::io::{Read, Write};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::str::FromStr;

#[derive(Parser)]
#[clap(version)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    New {
        /// Name of the template
        name: String,

        /// Name of the project
        project: String,

        /// Custom defined keys from terminal
        #[clap(default_value = "")]
        cli_keys: Vec<String>,
    },
    List,
    Init,
}

pub fn app(args: Args) {
    let home = directories::UserDirs::new()
        .unwrap()
        .home_dir()
        .join(".temple");
    let config = directories::UserDirs::new()
        .unwrap()
        .home_dir()
        .join(".temple_conf");

    if (!home.exists() || !config.exists()) && !matches!(args.command, Commands::Init) {
        eprintln!(
            "Error: No \"~/.temple\" or \".temple_conf\".\n    Run `temple init` to create them"
        );
        exit(1);
    }

    match args.command {
        Commands::New {
            name,
            project,
            cli_keys,
        } => {
            let template = home.join(name.as_str());

            if template.is_dir() && template.join(".temple").exists() {
                let keys_project_config = Keys::from_file_contents(&template.join(".temple"));
                let keys_project_user = Keys::from(cli_keys.join(" ").as_str());
                let mut project_keys = Keys::from(format!("project={}", &project).as_str());

                project_keys.add(keys_project_user);
                project_keys.add(keys_project_config);
                project_keys.add(Keys::from_file_contents(&config));

                // println!("{:?}", project_keys.list);

                let mut contents = Contents::from("{{ project }}");
                let dir_name = contents.replace(&START, &END, &project_keys);

                if let Err(e) = dir_name {
                    println!("Error: {}", e);
                    exit(1);
                }

                let dir_name = Contents::get_str_from_result(&dir_name.unwrap().1);
                // let target = current_dir().unwrap().join(dir_name.as_str());

                // println!("{}", target.display());

                if let Err(e) = render_dirs(
                    &template,
                    current_dir().unwrap().join(dir_name.as_str()),
                    &project_keys,
                    true,
                ) {
                    println!("Error: {}", e);
                    fs_extra::dir::remove(current_dir().unwrap().join(dir_name.as_str())).unwrap();
                    exit(1);
                }
            } else {
                println!("Error: Template does not exist");
            }
        }
        Commands::List => {
            let contents = home.read_dir().unwrap();
            let mut available: Vec<String> = vec![];

            for c in contents {
                let c = c.unwrap();

                if c.file_type().unwrap().is_dir() && c.path().join(".temple").exists() {
                    available.push(c.file_name().as_os_str().to_str().unwrap().into())
                }
            }

            if available.is_empty() {
                println!("No available templates. To add templates add them in ~/.temple.")
            } else {
                println!("Available templates: ");
                available.iter().for_each(|a| println!("   * {}", a));
            }
        }
        Commands::Init => {
            create_all(&home, true).unwrap();
            let mut conf = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(config)
                .unwrap();

            let default_config = "\
            name=Your name;\n\
            github=your_github_user;\n"
                .as_bytes();

            conf.write_all(default_config).unwrap();

            println!("Created ~/.temple_conf file and ~/.temple dir");
        }
    }
}

lazy_static! {
    static ref START: Indicator = Indicator::from("{{ ", true).unwrap();
    static ref END: Indicator = Indicator::from("{{ ", true).unwrap();
}

fn render_dirs(dir: &Path, target: PathBuf, keys: &Keys, dip: bool) -> Result<(), String> {
    if dir.is_dir() {
        let mut contents = Contents::from(dir.file_name().unwrap().to_str().unwrap());
        let dir_name = contents.replace(&START, &END, keys);

        if let Err(e) = dir_name {
            return Err(e);
        }

        let dir_name = Contents::get_str_from_result(&dir_name.unwrap().1);

        create_dir_all(if !dip {
            target.parent().unwrap().join(dir_name.as_str())
        } else {
            target.clone()
        })
        .unwrap();

        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                let mut contents = Contents::from(path.file_name().unwrap().to_str().unwrap());
                let replacement = contents.replace(&START, &END, keys);

                if let Err(e) = replacement {
                    return Err(e);
                }

                let replacement = Contents::get_str_from_result(&replacement.unwrap().1);

                render_dirs(&path, target.join(replacement.as_str()), keys, false)?;
            } else {
                if dip && path.file_name().unwrap().to_str().unwrap() == ".temple" {
                    continue;
                }

                let mut contents = Contents::from(path.file_name().unwrap().to_str().unwrap());
                let replacement = contents.replace(&START, &END, keys);

                if let Err(e) = replacement {
                    return Err(e);
                }

                let replacement = Contents::get_str_from_result(&replacement.unwrap().1);

                let new = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(target.clone().join(replacement.as_str()))
                    .unwrap();

                let mut contents =
                    Contents::from_file(path.parent().unwrap().join(path.file_name().unwrap()))
                        .unwrap();

                let replacement = contents.replace(&START, &END, &*keys);

                let result = match replacement {
                    Ok(o) => o,
                    Err(e) => return Err(e),
                };

                Contents::write_to_target(&result.1, new);
            }
        }
    }
    Ok(())
}

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

    pub fn from_file_contents(path: &Path) -> Keys {
        let mut file = OpenOptions::new().read(true).open(path).unwrap();
        let mut file_contents: Vec<u8> = Vec::new();
        file.read_to_end(&mut file_contents).unwrap();
        Keys::from(std::str::from_utf8(&file_contents).unwrap())
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
    fn find_indicator(slice: &[u8], from: usize, indicator: &Indicator) -> Option<usize>;
    fn replace(
        &mut self,
        start_indicator: &Indicator,
        end_indicator: &Indicator,
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

    pub fn write_to_target(result: &[NewContents], mut target: fs::File) {
        for r in result.iter() {
            match r {
                Old(slice) => target.write_all(slice).unwrap(),
                New(slice) => target.write_all(slice).unwrap(),
            }
        }
    }
}

impl Parse for Contents {
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
