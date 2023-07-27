mod args;
pub mod contents;
mod indicator;
pub mod indicators;
pub mod keys;
mod shared;
mod word;

pub use args::Commands;
pub use args::{Args, Parser};
pub use config_files::ConfigFiles;
pub use config_files::*;
pub use contents::Contents;
pub use contents::*;
use fs_extra::dir::create_all;
pub use indicators::Indicators;
pub use keys::Keys;
pub use shared::*;
pub use smartstring::alias::String;
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

pub fn init_temple_config_files(config_files: ConfigFiles) -> Result<(), String> {
    create_all(&config_files.temple_home, true).unwrap();
    let mut conf = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(config_files.temple_config)
        .unwrap();

    let default_config = "\
            name=Your name,\n\
            github=your_github_user,\n"
        .as_bytes();

    conf.write_all(default_config).unwrap();

    println!("Created ~/.temple_conf file and ~/.temple dir");

    Ok(())
}

fn get_templates_in_path(path: &Path) -> Vec<Template> {
    let contents = path.read_dir().unwrap();
    let mut available = vec![];

    for c in contents {
        let c = c.unwrap();

        if c.file_type().unwrap().is_dir() && c.path().join(".temple").exists() {
            available.push(Template {
                path: c.path(),
                name: c.file_name().as_os_str().to_str().unwrap().into(),
            })
        }
    }

    available
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

        if local.is_some() && global.is_some() {
            if prefer_local {
                local
            } else {
                global
            }
        } else if global.is_some() {
            global
        } else if local.is_some() {
            local
        } else {
            None
        }
    }
}

fn find_local_templates_folder(from: PathBuf, config_files: &ConfigFiles) -> Option<PathBuf> {
    if config_files.temple_home == from {
        return None;
    }

    let mut current = from;
    loop {
        let c = current.join(".temple");
        if c.is_dir() {
            return Some(c);
        }
        current = current.parent()?.into();
    }
}

fn get_available_templates(config_files: &ConfigFiles) -> Result<Templates, String> {
    config_files.exists()?;

    let contents_local = find_local_templates_folder(current_dir().unwrap(), config_files);

    let available = get_templates_in_path(&config_files.temple_home);
    let available_local = if let Some(path) = contents_local {
        if path != config_files.temple_home {
            get_templates_in_path(&path)
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    if available.is_empty() && available_local.is_empty() {
        return Err(
            "No available templates. To add templates add them in ~/.temple for global templates \
or ./.temple for local templates."
                .into(),
        );
    }

    Ok(Templates {
        global: available,
        local: available_local,
    })
}

pub fn list_available_templates(
    config_files: ConfigFiles,
    long: bool,
    path: bool,
) -> Result<(), String> {
    let templates = get_available_templates(&config_files)?;

    let home = directories::UserDirs::new().unwrap();
    let home = home.home_dir();

    if !templates.global.is_empty() {
        if long {
            println!("Available global templates (~/.temple): ");
            templates.global.iter().for_each(|a| {
                let a = format!(
                    "   * {}{}",
                    a.name,
                    path.then_some(format!("\t'{}'", a.path.to_str().unwrap()).into())
                        .unwrap_or(String::new())
                )
                .replace(home.to_str().unwrap(), "~");
                println!("{}", a)
            });
        } else {
            println!(
                "{}",
                templates
                    .global
                    .iter()
                    .map(|a| format!(
                        "{}{}",
                        a.name,
                        path.then_some(format!("\t'{}'", a.path.to_str().unwrap()).into())
                            .unwrap_or(String::new())
                    )
                    .replace(home.to_str().unwrap(), "~"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }

    if !templates.local.is_empty() {
        if long {
            println!(
                "Available local templates ({}/.temple): ",
                current_dir()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace(home.to_str().unwrap(), "~")
            );
            templates.local.iter().for_each(|a| {
                let a = format!(
                    "   * {}{}",
                    a.name,
                    path.then_some(format!("\t'{}'", a.path.to_str().unwrap()).into())
                        .unwrap_or(String::new())
                )
                .replace(home.to_str().unwrap(), "~");
                println!("{}", a)
            });
        } else {
            println!(
                "{}",
                templates
                    .local
                    .iter()
                    .map(|a| format!(
                        "{}{}",
                        a.name,
                        path.then_some(format!("\t'{}'", a.path.to_str().unwrap()).into())
                            .unwrap_or(String::new())
                    )
                    .replace(home.to_str().unwrap(), "~"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }

    Ok(())
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

    let templates = get_available_templates(&config_files)?;

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
