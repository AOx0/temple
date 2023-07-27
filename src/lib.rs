mod args;
pub mod contents;
mod indicator;
pub mod indicators;
pub mod keys;
mod shared;
mod word;

pub use args::Commands;
pub use args::{Args, Parser};
pub use config_files::*;
pub use contents::Contents;
pub use contents::*;
use fs_extra::dir::create_all;
pub use indicators::Indicators;
pub use keys::Keys;
pub use shared::*;
pub use smartstring::alias::String;
use std::env;
use std::{
    cell::RefCell,
    env::current_dir,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    rc::Rc,
};

mod config_files;
mod renderer;

pub struct ConfigFiles {
    pub temple_home: PathBuf,
    pub temple_config: PathBuf,
}

impl ConfigFiles {
    pub fn get() -> Self {
        let user_dirs = directories::UserDirs::new().unwrap();

        let (is_home, base_path) = if let Ok(config_dir) = env::var("XDG_CONFIG_DIRS") {
            (false, config_dir.into())
        } else if user_dirs.home_dir().join(".config").exists() {
            (false, user_dirs.home_dir().join(".config"))
        } else {
            (true, user_dirs.home_dir().to_owned())
        };

        let temple_home = base_path.join(if is_home { ".temple" } else { "temple" });
        let temple_config = temple_home.join("temple.conf");
        ConfigFiles {
            temple_home,
            temple_config,
        }
    }

    pub fn exists(&self) -> Result<(), String> {
        let home = directories::UserDirs::new().unwrap();
        if !self.temple_home.exists() || !self.temple_home.exists() {
            Err(format!(
                "Error: No '{}' and '{}'.\n    Run `temple init` to create them",
                self.temple_home.display(),
                self.temple_config.display()
            )
            .replace(home.home_dir().to_str().unwrap(), "~")
            .into())
        } else {
            Ok(())
        }
    }

    pub fn init_temple_config_files(&self) -> Result<(), String> {
        create_all(&self.temple_home, true).unwrap();
        let home = directories::UserDirs::new().unwrap();
        let mut conf = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.temple_config)
            .unwrap();

        let default_config = "\
                name=Your name,\n\
                github=your_github_user,\n"
            .as_bytes();

        conf.write_all(default_config).unwrap();

        println!(
            "{}",
            format!(
                "Created temple config files:\n    '{}'\n    '{}'",
                self.temple_home.display(),
                self.temple_config.display()
            )
            .replace(home.home_dir().to_str().unwrap(), "~")
        );

        Ok(())
    }

    fn get_available_templates(&self) -> Result<Templates, String> {
        self.exists()?;

        let contents_local = find_local_templates_folder(current_dir().unwrap(), self);

        let available_global = get_templates_in_path(&self.temple_home);
        let available_local = contents_local
            .and_then(|path| (path != self.temple_home).then(|| get_templates_in_path(&path)))
            .unwrap_or_default();

        if available_global.is_empty() && available_local.is_empty() {
            return Err(format!(
                "No available templates. To add templates add them in {} for global templates or create a .temple/ directory for local templates.",
                self.temple_home.display()
            )
            .into());
        }

        Ok(Templates {
            global: available_global,
            local: available_local,
        })
    }

    pub fn list_available_templates(&self, long: bool, path: bool) -> Result<(), String> {
        let templates = self.get_available_templates()?;

        let home = directories::UserDirs::new().unwrap();
        let home = home.home_dir();

        let iter = [templates.global, templates.local];

        for (i, iter) in iter.iter().enumerate() {
            (iter.len() != 0).then(|| {
                long.then(|| {
                    println!(
                        "Available {} templates (def at '{}'): ",
                        (i == 0).then_some("global").unwrap_or("local"),
                        (i == 0)
                            .then(|| { self.temple_home.as_path() })
                            .unwrap_or_else(|| { iter.first().unwrap().path.parent().unwrap() })
                            .to_str()
                            .unwrap()
                            .replace(home.to_str().unwrap(), "~")
                    );
                });

                print!(
                    "{}",
                    iter.iter()
                        .map(|a| format!(
                            "{dotchr}{tename}{tepath}{spacer}",
                            dotchr = long.then_some("    ").unwrap_or(""),
                            tename = (long || (!long && i == 0))
                                .then(|| a.name.clone())
                                .unwrap_or_else(|| format!("local:{}", a.name).into()),
                            tepath = path
                                .then_some(format!("\t'{}'", a.path.to_str().unwrap()).into())
                                .unwrap_or(String::new()),
                            spacer = long.then_some("\n").unwrap_or(" ")
                        )
                        .replace(home.to_str().unwrap(), "~"))
                        .collect::<Vec<_>>()
                        .join(""),
                );
                (long || (!long && i != 0)).then(|| println!());
            });
        }

        Ok(())
    }
}

fn get_templates_in_path(path: &Path) -> Vec<Template> {
    path.read_dir()
        .unwrap()
        .map(|c| c.unwrap())
        .filter_map(|c| {
            (c.file_type().unwrap().is_dir() && c.path().join(".temple").exists()).then(|| {
                Template {
                    path: c.path(),
                    name: c.file_name().as_os_str().to_str().unwrap().into(),
                }
            })
        })
        .collect::<Vec<_>>()
}

#[derive(Clone)]
struct Template {
    pub path: PathBuf,
    pub name: String,
}

struct Templates {
    pub global: Vec<Template>,
    pub local: Vec<Template>,
}

impl Templates {
    pub fn get_named(&self, name: &str, prefer_local: bool) -> Option<&Template> {
        let local = self.local.iter().find(|&t| t.name == name);
        let global = self.global.iter().find(|&t| t.name == name);

        match (local, global) {
            (Some(local), Some(global)) => Some(prefer_local.then(|| local).unwrap_or(global)),
            (None, Some(global)) => Some(global),
            (Some(local), None) => Some(local),
            _ => None,
        }
    }
}

fn find_local_templates_folder(from: PathBuf, config_files: &ConfigFiles) -> Option<PathBuf> {
    if config_files.temple_home == from {
        return None;
    }

    let mut current = from.as_path();
    while let Some(parent) = current.parent() {
        let c = current.join(".temple");
        if c.is_dir() {
            return Some(c);
        } else if c.is_file() {
            println!("Warning: Found {} which is not a directory", c.display());
        }
        current = parent;
    }

    None
}

pub fn create_project_from_template(
    template_name: &str,
    project_name: &str,
    cli_keys: Vec<std::string::String>,
    config_files: ConfigFiles,
    prefer_local: bool,
    place_in_place: bool,
    overwrite: bool,
) -> Result<(), String> {
    config_files.exists()?;

    let templates = (&config_files).get_available_templates()?;

    let config = config_files.temple_config;
    let handles = Rc::new(RefCell::new(vec![]));

    let template = templates.get_named(template_name, prefer_local);

    let template: &Path = if let Some(template) = template {
        if template.path.join(".temple").is_file() {
            &template.path
        } else {
            return Err("Error: Template does not exist".into());
        }
    } else {
        return Err("Error: Template does not exist".into());
    };

    let keys_project_config = Keys::from_file_contents(&template.join(".temple"));
    let keys_project_user = Keys::from(cli_keys.join(" ").as_str());
    let mut project_keys = Keys::from(format!("project={}", &project_name).as_str());

    project_keys.add(keys_project_user);
    project_keys.add(keys_project_config);
    project_keys.add(Keys::from_file_contents(&config));

    let start = project_keys
        .get_match("start_indicator", None)
        .unwrap_or("{{ ");
    let end = project_keys
        .get_match("end_indicator", None)
        .unwrap_or(" }}");

    let indicators = &Indicators::new(start, end).unwrap();

    let target = current_dir().unwrap();

    let target = if place_in_place {
        target
    } else {
        target.join(project_name)
    };

    if place_in_place {
        renderer::render_recursive(
            handles.clone(),
            template,
            target.clone(),
            &project_keys,
            true,
            indicators,
            true,
            overwrite,
            place_in_place,
        )?;
    } else if target.exists() && !overwrite {
        return Err(format!(
            "Error: directory {} already exists",
            target.file_name().unwrap().to_str().unwrap()
        )
        .into());
    }

    if let Err(e) = renderer::render_recursive(
        handles.clone(),
        template,
        target,
        &project_keys,
        true,
        indicators,
        false,
        overwrite,
        place_in_place,
    ) {
        fs_extra::dir::remove(current_dir().unwrap().join(project_name)).unwrap();
        return Err(e);
    }

    let handlers = Rc::try_unwrap(handles)
        .expect("I hereby claim that my_ref is exclusively owned")
        .into_inner();

    for handler in handlers {
        let res = handler.join();
        if let Err(error) = res {
            return Err(format!("Error: {:?}", error).into());
        } else if let Ok(Err(error)) = res {
            return Err(error);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn basic_parse() {
        let mut contents = Contents::from("lmao {{ jaja }}");
        let indicators = Indicators::new("{{ ", " }}").unwrap();
        let keys = Keys::from("jaja=perro");
        let replace = contents.replace(&indicators, &keys);

        let r = if let Ok(res) = replace {
            match res.0 {
                666 => String::from("No changes. No keys"),
                _ => Contents::get_str_from_result(&res.1),
            }
        } else {
            String::from("Invalid chars or data")
        };

        println!("{r}");
        assert_eq!(r, "lmao perro");
    }

    #[test]
    fn custom_key_parse() {
        let mut contents = Contents::from("lmao [[[jaja]]]");
        let indicators = Indicators::new("[[[", "]]]").unwrap();
        let keys = Keys::from("jaja=perro");
        let replace = contents.replace(&indicators, &keys);

        let r = if let Ok(res) = replace {
            match res.0 {
                666 => String::from("No changes. No keys"),
                _ => Contents::get_str_from_result(&res.1),
            }
        } else {
            String::from("Invalid chars or data")
        };

        println!("{r}");
        assert_eq!(r, "lmao perro");
    }
}
