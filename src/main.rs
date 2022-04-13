use clap::{Parser, Subcommand};
use fs_extra::dir::create_all;
use smartstring::alias::String;
use std::env::current_dir;
use std::fs;
use std::fs::{create_dir_all, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use temple_parse::*;

#[derive(Parser)]
#[clap()]
struct Args {
    #[clap(subcommand)]
    pub(crate) command: Commands,
}

macro_rules! r_keys_str {
    ($contents: expr, $keys: expr) => {
        $contents.replace(
            Indicator::from("{{ ", true).unwrap(),
            Indicator::from(" }}", false).unwrap(),
            $keys,
        )
    };
}

macro_rules! gen_keys {
    ($path: expr) => {{
        let mut file = OpenOptions::new().read(true).open($path).unwrap();
        let mut file_contents: Vec<u8> = Vec::new();
        file.read_to_end(&mut file_contents).unwrap();
        Keys::from_string(std::str::from_utf8(&file_contents).unwrap())
    }};
}

#[derive(Subcommand, Debug)]
enum Commands {
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

fn render_dirs(dir: &Path, target: PathBuf, keys: &Keys, dip: bool) -> Result<(), String> {
    if dir.is_dir() {
        // create_dir(&target.join(r_keys_str!(dir.file_name().unwrap().to_str().unwrap()).as_str())).unwrap();
        let mut contents = Contents::from(dir.file_name().unwrap().to_str().unwrap());
        let dir_name = r_keys_str!(contents, keys);

        if let Err(e) = dir_name {
            return Err(e);
        }

        let dir_name = Contents::get_str_from_result(&dir_name.unwrap().1);

        /* println!(
            "Creating dir {} in {}",
            dir.display(),
            target.display()
        ); */

        create_dir_all(if !dip { target.parent().unwrap().join(dir_name.as_str()) } else { target.clone() }).unwrap();

        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                let mut contents = Contents::from(path.file_name().unwrap().to_str().unwrap());
                let replacement = r_keys_str!(contents, keys);

                if let Err(e) = replacement {
                    return Err(e);
                }

                let replacement = Contents::get_str_from_result(&replacement.unwrap().1);

                render_dirs(&path, target.join(replacement.as_str()), keys, false)?;
            } else {
                let mut contents = Contents::from(path.file_name().unwrap().to_str().unwrap());
                let replacement = r_keys_str!(contents, keys);

                if let Err(e) = replacement {
                    return Err(e);
                }

                let replacement = Contents::get_str_from_result(&replacement.unwrap().1);

                let new = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(target.join(replacement.as_str()))
                    .unwrap();

                let mut contents =
                    Contents::from_file(path.parent().unwrap().join(path.file_name().unwrap()))
                        .unwrap();
                
                let replacement = contents
                .replace(
                    Indicator::from("{{ ", true).unwrap(),
                    Indicator::from(" }}", false).unwrap(),
                    keys,
                );

                let result = match replacement {
                    Ok(o) => {o},
                    Err(e) => return Err(e),
                };

                Contents::write_to_target(&result.1, new);

                /* println!(
                    "Rendering {} in {}",
                    path.parent()
                        .unwrap()
                        .join(path.file_name().unwrap())
                        .display(),
                    target
                        .join(
                            r_keys_str!(path.file_name().unwrap().to_str().unwrap(), keys).as_str()
                        )
                        .display()
                ) */
            }
        }
    }
    Ok(())
}

fn main() {
    let args = Args::parse();

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
                let keys_project_config = gen_keys!(template.join(".temple"));
                let keys_project_user = Keys::from_string(cli_keys.join(" ").as_str());
                let mut project_keys = Keys::from_string(format!("project={}", &project).as_str());

                project_keys.add(keys_project_user);
                project_keys.add(keys_project_config);
                project_keys.add(gen_keys!(config));

                // println!("{:?}", project_keys.list);

                let mut contents = Contents::from("{{ project }}");
                let dir_name = r_keys_str!(contents, &project_keys);

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
