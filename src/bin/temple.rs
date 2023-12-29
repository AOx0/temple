use anyhow::{anyhow, bail, ensure, Context, Result};
use clap::Parser;
use std::{path::PathBuf, process::ExitCode};
use temple::{
    args::{Args, Commands, InitOpt},
    config::{Prefer, TempleDirs},
    error, info,
    replacer::ContentsLexer,
    trace,
    values::Values,
};

fn name_is_valid(name: &str) -> Result<()> {
    (name.is_ascii() && !name.contains(':'))
        .then_some(())
        .ok_or(anyhow!("Invalid name: '{name}'"))
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
        Commands::Init { sub } => match sub {
            InitOpt::Global => {
                temple_dirs
                    .create_global_dir()
                    .map_err(|e| anyhow!("Failed creating global dir: {e}"))?;
                temple_dirs
                    .create_global_config()
                    .map_err(|e| anyhow!("Failed creating config file: {e}"))?
                    .map(|mut f| {
                        use std::io::Write;

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
            if !is_global && !confirm_creation(&path) {
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

            if !confirm_remove(&path) {
                return Ok(());
            }

            info!("Removing template '{name}' at {}", path.display());

            std::fs::remove_dir_all(&path).map_err(|err| anyhow!("Failed removing dir: {err}"))?;

            Ok(())
        }
        Commands::Deinit { sub } => match sub {
            temple::args::DeinitOpt::Global => {
                let path = temple_dirs.global_config();

                if confirm_remove(path) {
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
                    if confirm_remove(path) {
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
            ref template_name, ..
        } => {
            let templates = temple_dirs
                .get_available_templates()
                .map_err(|err| anyhow!("Failed to get templates: {err}"))?;

            let get_config_path = |path: &std::path::Path| {
                if path.join("config.tpl").exists() {
                    path.join("config.tpl")
                } else {
                    path.join("config.temple")
                }
            };

            let global_config = get_config_path(temple_dirs.global_config());
            let local_config = temple_dirs.local_config().map(get_config_path);

            let global_config_str = std::fs::read_to_string(&global_config)?;
            let local_config_str = local_config.as_ref().map(std::fs::read_to_string);

            let global_config =
                Values::from_str(&global_config_str, &global_config).map_err(|err| {
                    eprintln!("{err:?}");
                    anyhow!("Failed to parse values from {}", global_config.display())
                })?;

            let local_config =
                if let (Some(path), Some(contents)) = (local_config, local_config_str) {
                    let contents = contents?;
                    Values::from_str(&contents, &path).map_err(|err| {
                        eprintln!("{err:?}");
                        anyhow!("Failed to parse values from {}", path.display())
                    })?
                } else {
                    Values::default()
                };

            let config = global_config.stash(local_config);

            let prefers = if template_name.starts_with("local:") {
                Prefer::Local
            } else {
                Prefer::Global
            };

            let name = template_name.trim_start_matches("local:");
            name_is_valid(name)?;

            let template = templates
                .get_named(name, &prefers)
                .ok_or(anyhow!("Template '{name}' does not exist"))?;

            trace!("Working with template {:?}", template);

            // let contents = " Hola ma llamo {{ name }} y {{ if xp == 4 }}soy nuevo{{ else }}soy experimentado{{}} en esto";
            let contents = " Hola ma llamo {{ if name }} mas texto {{}} {{";

            let path = PathBuf::default();
            let mut contents = ContentsLexer::new(contents, &path, &config)?;

            while let Some(token) = contents.next() {
                if let Err(e) = token {
                    error!(e);
                    break;
                }

                println!(
                    "{:?}: {}: {}: {token:?}",
                    contents.span(),
                    contents.get_location(contents.span()),
                    contents.slice(),
                )
            }

            Ok(())
        }
    }
}

fn confirm_remove(path: &std::path::Path) -> bool {
    let ans = inquire::Confirm::new(&format!("Do you want to remove {}?", path.display()))
        .with_default(false)
        .prompt();

    ans.unwrap()
}

fn confirm_creation(path: &std::path::Path) -> bool {
    let ans = inquire::Confirm::new(&format!("Do you want to create {}?", path.display()))
        .with_default(false)
        .prompt();

    ans.unwrap()
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
