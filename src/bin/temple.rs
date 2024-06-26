use anyhow::{anyhow, bail, ensure, Context, Result};
use clap::Parser;
use inquire::validator::Validation;
use std::{
    borrow::Cow,
    env::{current_dir, current_exe},
    fs::OpenOptions,
    io::{Read, Write},
    ops::Not,
    path::PathBuf,
    process::ExitCode,
    str::FromStr,
};
use temple::{
    args::{Args, Commands, InitOpt},
    config::{Prefer, TempleDirs},
    error, info,
    replacer::ContentsLexer,
    trace,
    values::{Type, Values},
    warn,
};
use walkdir::WalkDir;

fn templ_path(path: &std::path::Path) -> PathBuf {
    if path.join("config.tpl").exists() {
        path.join("config.tpl")
    } else {
        path.join("config.temple")
    }
}

fn name_is_valid(name: &str) -> Result<()> {
    (name.is_ascii() && !name.contains(':'))
        .then_some(())
        .ok_or(anyhow!("Invalid name: '{name}'"))
}

fn parse_values_from_path(path: &std::path::Path, buff: &mut String) -> Result<Values> {
    buff.clear();

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|err| anyhow!("Error opening file {path}: {err}", path = path.display()))?;

    file.read_to_string(buff)
        .map_err(|err| anyhow!("Error reading file {path}: {err}", path = path.display()))?;

    Values::from_str(buff, path).map_err(|err| {
        eprintln!("{err:?}");
        anyhow!("Failed to parse values from {}", path.display())
    })
}

fn parse_values_from_str(str: &str, desc: &str) -> Result<Values> {
    Values::from_str(str, current_exe().unwrap().as_path()).map_err(|err| {
        eprintln!("{err:?}");
        anyhow!("Failed to parse values from {desc}")
    })
}

fn app(args: &Args) -> Result<()> {
    let temple_dirs = TempleDirs::default_paths()
        .map_err(|e| anyhow!("Failed getting default directories: {e}"))?;

    trace!("Global config: {}", temple_dirs.global_config().display());
    trace!(
        "Local config: {}",
        temple_dirs
            .local_config()
            .map(|e| e.display())
            .unwrap_or(std::path::PathBuf::from("None").display())
    );

    if !matches!(args.command, Commands::Init { sub } if sub == InitOpt::Global) {
        let exists = temple_dirs.global_config().exists() && temple_dirs.global_config().is_dir();
        trace!("Checking if global configuration exists: {exists}",);
        ensure!(
            exists,
            anyhow!(
                r#"There was an error with the global config dir: {0}
Are you sure it exists, its a dir, and it contains a config.tpl?
If this is your first temple execution you can create a new global config with the command:

    temple init global
"#,
                temple_dirs.global_config().display(),
            )
        );
    }

    match args.command {
        Commands::List { .. } => temple_dirs.display_available_templates(&args.command),
        Commands::Info { ref template_name } => {
            let is_global = !template_name.starts_with("local:");
            let name = template_name.trim_start_matches("local:");

            name_is_valid(name).map_err(|err| anyhow!("Error with name: {err}"))?;

            let prefers = if !is_global {
                Prefer::Local
            } else {
                Prefer::Global
            };

            let mut buffer = String::new();

            let templates = temple_dirs
                .get_available_templates()
                .map_err(|err| anyhow!("Failed to get templates: {err}"))?;

            let template = templates
                .get_named(name, &prefers)
                .ok_or(anyhow!("Template '{name}' does not exist"))?;

            let path = templ_path(&template.0);
            let config = parse_values_from_path(&path, &mut buffer)
                .map_err(|err| anyhow!("Error while parsing config: {err}"))?;

            println!(
                "Name: {name}\nPath: {path}\nConfig: {config}\nConfig values: {conf:#?}",
                path = template.0.display(),
                config = path.display(),
                conf = config
            );

            Ok(())
        }
        Commands::Init { sub } => match sub {
            InitOpt::Global => {
                temple_dirs
                    .create_global_dir()
                    .map_err(|e| anyhow!("Failed creating global dir: {e}"))?;
                temple_dirs
                    .create_global_config()
                    .map_err(|e| anyhow!("Failed creating config file: {e}"))?
                    .map(|mut f| {
                        info!("Writing default configuration to global configuration file");

                        writeln!(
                            f,
                            r#"temple_delimiters: {{ open: String, close: String }} = {{
    open: "{{{{",
    close: "}}}}"
}}"#
                        )
                        .map_err(|e| e.into())
                    })
                    .unwrap_or(Ok(()))
            }
            InitOpt::Local { not_hidden } => {
                let current_dir = std::env::current_dir()
                    .context("Failed to get current dir")?
                    .join(if not_hidden { "temple" } else { ".temple" });
                temple_dirs.create_config_dir(&current_dir)?;
                temple_dirs.create_config_file(&current_dir).map(|_| ())
            }
        },
        Commands::Create { ref template_name } => {
            let is_global = !template_name.starts_with("local:");
            let name = template_name.trim_start_matches("local:");

            name_is_valid(name)?;

            let path = if is_global {
                temple_dirs.global_config()
            } else if let Some(path) = temple_dirs.local_config() {
                path
            } else {
                bail!("Tried to create a new local template but there is no local temple folder");
            }
            .join(name);

            ensure!(
                !path.exists(),
                "A {} template with the name '{}' already exists",
                if is_global { "global" } else { "local" },
                name
            );

            // To avoid the user doing unwanted operations we prompt for confirmation with
            // the path visible to the user
            if !is_global && !confirm_creation(&path)? {
                return Ok(());
            }

            info!("Creating new template directory: {}", path.display());

            std::fs::create_dir_all(&path)
                .map_err(|err| anyhow!("Failed creating template directory: {err}"))?;

            let config = path.join("config.tpl");

            info!(
                "Creating empty template configuration at {}",
                config.display()
            );

            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&config)
                .map_err(|err| anyhow!("Failed creating configuration file: {err}"))?;

            Ok(())
        }
        Commands::Remove { ref template_name } => {
            let is_local = template_name.starts_with("local:");
            let name = template_name.trim_start_matches("local:");

            name_is_valid(name)?;

            let path = if !is_local {
                temple_dirs.global_config()
            } else if let Some(path) = temple_dirs.local_config() {
                path
            } else {
                bail!("Tried removing local template but there is no local template directory");
            }
            .join(name);

            ensure!(
                path.exists(),
                "A {} template with the name '{}' does not exists",
                if !is_local { "global" } else { "local" },
                name
            );

            if !confirm_remove(&path)? {
                return Ok(());
            }

            info!("Removing template '{name}' at {}", path.display());

            std::fs::remove_dir_all(&path).map_err(|err| anyhow!("Failed removing dir: {err}"))?;

            Ok(())
        }
        Commands::Deinit { sub } => match sub {
            temple::args::DeinitOpt::Global => {
                let path = temple_dirs.global_config();

                if confirm_remove(path)? {
                    info!(
                        "Removing temple configuration directory: {}",
                        path.display()
                    );
                    std::fs::remove_dir_all(temple_dirs.global_config()).map_err(|e| anyhow!("{e}"))
                } else {
                    Ok(())
                }
            }
            temple::args::DeinitOpt::Local => {
                if let Some(path) = temple_dirs.local_config() {
                    if confirm_remove(path)? {
                        info!(
                            "Removing temple configuration directory: {}",
                            path.display()
                        );
                        std::fs::remove_dir_all(path).map_err(|e| anyhow!("{e}"))
                    } else {
                        Ok(())
                    }
                } else {
                    anyhow::bail!("No local config")
                }
            }
        },
        Commands::DebugConfig { ref paths } => {
            let mut result_value = Values::default();

            for path in paths {
                info!("Reading: {}", path.display());

                let contents = std::fs::read_to_string(path)?;

                let values = Values::from_str(&contents, path).map_err(|err| {
                    eprintln!("{err:?}");
                    anyhow!("Failed to parse values from {}", path.display())
                })?;

                info!("{}:\n{:#?}", path.display(), values);

                values.verify_types().map_err(|err| {
                    error!(err);
                    anyhow!("Invalid types")
                })?;

                result_value = result_value.stash(values);
                result_value.verify_types().map_err(|err| {
                    error!(err);
                    anyhow!("Invalid types")
                })?;
            }

            info!("End result:\n{:#?}", result_value);
            result_value.verify_types().map_err(|err| {
                error!(err);
                anyhow!("Invalid types")
            })
        }
        Commands::New {
            ref template_name,
            ref project_name,
            mut in_place,
            ref cli_keys,
            ref overwrite,
            ..
        } => {
            let templates = temple_dirs
                .get_available_templates()
                .map_err(|err| anyhow!("Failed to get templates: {err}"))?;

            let name = template_name.trim_start_matches("local:");
            name_is_valid(name)?;
            name_is_valid(project_name)?;

            let prefers = if template_name.starts_with("local:") {
                Prefer::Local
            } else {
                Prefer::Global
            };

            let mut buff = String::new();

            let template = templates
                .get_named(name, &prefers)
                .ok_or(anyhow!("Template '{name}' does not exist"))?;

            let global_config =
                parse_values_from_path(&templ_path(temple_dirs.global_config()), &mut buff)
                    .map_err(|err| {
                        anyhow!(
                            "Error while parsing global config at {}: {err}",
                            templ_path(temple_dirs.global_config()).display()
                        )
                    })?;
            let local_config = temple_dirs
                .local_config()
                .map(templ_path)
                .map(|local| parse_values_from_path(&local, &mut buff))
                .transpose()?
                .unwrap_or_default();
            let template_config = parse_values_from_path(&templ_path(&template.0), &mut buff)
                .map_err(|err| {
                    anyhow!(
                        "Error while parsing config at {}: {err}",
                        templ_path(&template.0).display()
                    )
                })?;
            let cli_config = parse_values_from_str(&cli_keys.join(" "), "Args")
                .map_err(|err| anyhow!("Error while parsing config from str: {err}"))?;

            let mut config = global_config
                .stash(local_config)
                .stash(template_config)
                .stash(cli_config);

            for (name, value) in config.value_map.iter_mut() {
                if value.is_null() {
                    let dtype = config.type_map.get_mut(name).expect("We know it exists");
                    match dtype {
                        Type::Array(_)
                        | Type::Object(_)
                        | Type::Any
                        | Type::Number => {
                            let input = ask_any(name, &format!("{dtype}"), dtype.clone())?;
                            let input_value = Values::parse_value(&input, "")
                                .expect("Infallible, checked inside the function");

                            let val_type = Type::from_value(&input_value, dtype);
                            if val_type.is_equivalent(dtype) {
                                *value = input_value;
                                *dtype = val_type;
                            } else {
                                bail!("Error, value not valid for key {name:?} with type {dtype}\n");
                            }
                        },
                        Type::String => {
                            *value = tera::Value::String(ask_string(name)?)
                        },
                        Type::Bool => *value = tera::Value::Bool(ask_bool(&format!("Set bool value of {name:?} to `true`?"))?),
                        Type::Unknown => bail!(
                            "Keys with unknown data type and no value assigned are not supported: {name:?}\n"
                        ),
                    }
                }
            }
            trace!("Final config: {:?}", config.value_map);
            trace!("Working with template {:?}", template);

            let current_dir =
                current_dir().map_err(|err| anyhow!("Failed getting current dir: {err}"))?;

            if matches!(config.value_map.get("temple_in_place"), Some(v) if v.is_boolean()) {
                let conf_in_place = config
                    .value_map
                    .get("temple_in_place")
                    .unwrap()
                    .as_bool()
                    .unwrap();

                in_place = conf_in_place || in_place;
            }

            let current_dir = if in_place {
                current_dir
            } else {
                current_dir.join(project_name)
            };

            // Insert template and project name
            {
                config.value_map.insert(
                    "temple_template_name".to_string(),
                    tera::Value::String(template_name.to_string()),
                );

                config.value_map.insert(
                    "temple_project_name".to_string(),
                    tera::Value::String(project_name.to_string()),
                );

                config.value_map.insert(
                    "temple_render_path".to_string(),
                    tera::Value::String(current_dir.as_os_str().to_string_lossy().to_string()),
                );
            }

            let mut overwrite_targets = None;

            let walker = WalkDir::new(&template.0).into_iter();
            for entry in walker.filter_entry(|e| {
                let name = e.file_name().to_str().unwrap_or_default();
                !(name.ends_with(".temple") || name.ends_with(".tpl"))
            }) {
                if let Ok(entry) = entry.map_err(|err| warn!("Error with path: {}", err)) {
                    let target = entry
                        .path()
                        .strip_prefix(&template.0)
                        .map_err(|err| anyhow!("Failed stripping prefix: {err}"))?;

                    if target == PathBuf::from_str("").expect("Infallible") {
                        trace!(
                            "Rendering: Skipping empty target, presumably the root file. Path {}",
                            entry.path().display()
                        );
                        continue;
                    }

                    let target = current_dir.join(target);

                    trace!(
                        "Rendering: Render of {} into {}",
                        entry.path().display(),
                        target.display()
                    );

                    if entry.file_type().is_dir() {
                        continue;
                    }

                    let mut origin =
                        OpenOptions::new()
                            .read(true)
                            .open(entry.path())
                            .map_err(|err| {
                                anyhow!("Error with origin path {}: {err}", entry.path().display())
                            })?;

                    let target = render_path(&target, &config)?;

                    // Set the overwrite value once
                    overwrite_targets = if target.exists() && overwrite_targets.is_none() {
                        if !overwrite {
                            Some(ask_bool(&format!("The target dir {} already exists. Do you want to overwrite the target files?", target.display()))?)
                        } else {
                            Some(*overwrite)
                        }
                    } else {
                        overwrite_targets
                    };

                    // If overwrite is false skip the render
                    if target.exists() && !overwrite_targets.is_some_and(|v| v) {
                        warn!(
                            "Skipping dir {} because it already exists",
                            target.display()
                        );
                        continue;
                    }

                    // Create parent dirs as needed
                    if let Some(par) = target.parent() {
                        std::fs::create_dir_all(par).map_err(|err| {
                            anyhow!("Error while creating parent of {}: {err}", target.display())
                        })?;
                    }

                    let mut target = OpenOptions::new()
                        .create(true)
                        .truncate(overwrite_targets.unwrap_or_default())
                        .write(true)
                        .open(&target)
                        .map_err(|err| {
                            anyhow!("Error with target path {}: {err}", target.display())
                        })?;

                    buff.clear();
                    origin.read_to_string(&mut buff).map_err(|err| {
                        anyhow!(
                            "Error while reading origin path {}: {err}",
                            entry.path().display()
                        )
                    })?;

                    let contents = buff.as_str();
                    let path = entry.path();
                    let repl = Replaced::from(
                        &collect_tokens(ContentsLexer::new(contents, path, &config)?),
                        &config,
                    )
                    .map_err(|err| {
                        anyhow!(
                            "Error while replacing values from {}: {err:?}",
                            path.display()
                        )
                    })?;

                    for res in repl.contents {
                        target
                            .write_all(res.as_bytes())
                            .map_err(|err| anyhow!("Error writing: {err}"))?;
                    }

                    // info!("{:?}", repl.map(|v| v.contents.join("")));
                };
            }

            info!("Rendered {:?} at {:?}", name, current_dir.display());
            Ok(())
        }
    }
}

fn render_path(render: &std::path::Path, config: &Values) -> Result<PathBuf, anyhow::Error> {
    let contents = render.display().to_string();
    let path = std::path::Path::new("Path");
    let repl = Replaced::from(
        &collect_tokens(ContentsLexer::new(&contents, path, config)?),
        config,
    )
    .map_err(|err| {
        anyhow!(
            "Error while replacing values from path name {}: {err:?}",
            path.display()
        )
    })?;
    let path = repl.contents.join("");
    let path = PathBuf::from_str(&path).map_err(|err| {
        anyhow!(
            "Error computing target path from {}: {err}",
            render.display()
        )
    })?;
    Ok(path)
}

fn collect_tokens(mut contents: ContentsLexer<'_>) -> Vec<temple::replacer::Type<'_>> {
    let mut con = vec![];

    while let Some(token) = contents.next() {
        if let Err(e) = token {
            error!(e);
            break;
        }

        trace!(
            "Lexer: {:?}: {}: {}: {token:?}",
            contents.span(),
            contents.get_location(contents.span()),
            contents.slice(),
        );

        con.push(token.unwrap());
    }

    con
}

#[derive(Debug, Clone)]
#[repr(transparent)]
struct Replaced<'a> {
    contents: Vec<Cow<'a, str>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum ErrorReplace<'i> {
    NoValue(&'i str),
    NoField(&'i str, &'i str),
    ExpectedValue(&'i str),
    UnexpectedObject(&'i str, &'i str),
    UnexpectedField(&'i str, &'i str),
}

impl<'a> Replaced<'a> {
    fn from(
        mut value: &[temple::replacer::Type<'a>],
        values: &'a Values,
    ) -> Result<Self, Vec<ErrorReplace<'a>>> {
        use temple::replacer::Type;

        let mut contents = Vec::new();
        let mut errors = Vec::new();

        while !value.is_empty() {
            match *value {
                [Type::Raw(blob), ..] => {
                    contents.push(Cow::Borrowed(blob));

                    value = &value[1..];
                }
                [Type::Ident(ident), ..] => {
                    if let Some(v) = values.value_map.get(ident) {
                        if let Some(v) = v.as_object().is_none().then_some(v) {
                            if let tera::Value::String(v) = v {
                                contents.push(Cow::Owned(v.to_owned()));
                            } else if v.is_null().not() {
                                contents.push(Cow::Owned(v.to_string()));
                            } else {
                                errors.push(ErrorReplace::NoValue(ident))
                            }
                        } else {
                            errors.push(ErrorReplace::ExpectedValue(ident));
                        }
                    } else {
                        errors.push(ErrorReplace::NoValue(ident));
                    }

                    value = &value[1..];
                }
                [Type::IdentWithField(access), ..] => {
                    let (ident, fields) = access.split_once('.').expect(
                        "The REGEX does guarantee there is at least an identifier and one field",
                    );

                    'a: {
                        if let Some(mut curr) = values.value_map.get(ident) {
                            for field in fields.split('.') {
                                curr = if curr.is_object() {
                                    if let Some(v) = curr.get(field) {
                                        v
                                    } else {
                                        errors.push(ErrorReplace::NoField(access, field));
                                        break;
                                    }
                                } else {
                                    errors.push(ErrorReplace::UnexpectedField(access, field));
                                    break;
                                }
                            }

                            if curr.is_object() {
                                errors.push(ErrorReplace::UnexpectedObject(access, ""));
                                break 'a;
                            }

                            if let tera::Value::String(curr) = curr {
                                contents.push(Cow::Owned(curr.to_owned()));
                            } else if curr.is_null().not() {
                                contents.push(Cow::Owned(curr.to_string()));
                            } else {
                                errors.push(ErrorReplace::NoValue(ident))
                            }
                        } else {
                            errors.push(ErrorReplace::NoValue(ident));
                        }
                    }

                    value = &value[1..];
                }
                _ => {
                    value = &value[1..];
                }
            }
        }

        if errors.is_empty() {
            Ok(Self { contents })
        } else {
            Err(errors)
        }
    }
}

fn confirm_remove(path: &std::path::Path) -> Result<bool> {
    inquire::Confirm::new(&format!("Do you want to remove {}?", path.display()))
        .with_default(false)
        .prompt()
        .map_err(|err| anyhow!(err))
}

fn confirm_creation(path: &std::path::Path) -> Result<bool> {
    inquire::Confirm::new(&format!("Do you want to create {}?", path.display()))
        .with_default(false)
        .prompt()
        .map_err(|err| anyhow!(err))
}

fn ask_string(key: &str) -> Result<String> {
    inquire::prompt_text(format!("Enter a String value for field {key:?}:"))
        .map_err(|err| anyhow!(err))
}

fn ask_any(key: &str, kind: &str, expected_type: Type) -> Result<String> {
    inquire::Text::new(&format!("Enter {kind} value for field {key:?}:"))
        .with_validator(move |a: &str| {
            Ok(if a.is_empty().not() {
                match Values::parse_value(a, "stdin") { 
                    Ok(value) => {
                        let val_type = Type::from_value(&value, &expected_type);
                        if val_type.is_equivalent(&expected_type) {
                            inquire::validator::Validation::Valid
                        } else {
                            Validation::Invalid(format!("Mismatching types. Expected {expected_type} but found {val_type}").into())
                        }
                    },
                    Err(e) => Validation::Invalid(e.to_string().into()),
                }
            } else {
                Validation::Invalid(
                    "Empty values not allowed".to_owned().into(),
                )
            })
        })
        .prompt()
        .map_err(|err| anyhow!(err))
}

fn ask_bool(key: &str) -> Result<bool> {
    inquire::prompt_confirmation(key).map_err(|err| anyhow!(err))
}

fn main() -> ExitCode {
    let args = Args::parse();

    match app(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if !args.no_errors() {
                error!("{e}",);
            }
            ExitCode::FAILURE
        }
    }
}
