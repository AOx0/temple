use fs_extra::dir::create_all;
use lazy_static::lazy_static;
use smartstring::alias::String;
use std::env::current_dir;
use std::fs;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;

mod clap_s;
mod contents;
mod indicator;
mod keys;
mod word;

pub use clap_s::*;
pub use contents::*;
pub use indicator::*;
pub use keys::*;
pub use word::*;

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
    static ref END: Indicator = Indicator::from(" }}", false).unwrap();
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

pub trait Parse {
    fn find_indicator(slice: &[u8], from: usize, indicator: &Indicator) -> Option<usize>;
    fn replace(
        &mut self,
        start_indicator: &Indicator,
        end_indicator: &Indicator,
        keys: &Keys,
    ) -> Result<(usize, Vec<NewContents>), String>;
}
