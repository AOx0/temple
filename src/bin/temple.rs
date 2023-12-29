use anyhow::{anyhow, ensure, Context, Result};
use clap::Parser;
use std::{path::PathBuf, process::ExitCode};
use temple::{
    args::{Args, Commands, InitOpt},
    config::TempleDirs,
    error, info,
    replacer::ContentsLexer,
    trace,
    values::Values,
};

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
        ensure!(
            temple_dirs.global_config().is_dir(),
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
        Commands::New { .. } => {
            let globals = TempleDirs::get_templates_in_dir(temple_dirs.global_config())?;
            let locals = if let Some(l) = temple_dirs
                .local_config()
                .as_ref()
                .map(|c| TempleDirs::get_templates_in_dir(c))
            {
                l?
            } else {
                Vec::default()
            };

            let get_config_path = |path: &std::path::Path| {
                if path.join("config.tpl").exists() {
                    path.join("config.tpl")
                } else {
                    path.join("config.temple")
                }
            };

            let global_config = get_config_path(temple_dirs.global_config());
            let local_config = temple_dirs.local_config().map(|p| get_config_path(p));

            let global_config_str = std::fs::read_to_string(&global_config)?;
            let local_config_str = local_config.as_ref().map(|v| std::fs::read_to_string(v));

            let mut global_config =
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

            // let contents = " Hola ma llamo {{ name }} y {{ if xp == 4 }}soy nuevo{{ else }}soy experimentado{{}} en esto";
            let contents = " Hola ma llamo {{ if name }} mas texto {{}} {{";

            let mut path = PathBuf::default();
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

            let templates = temple_dirs
                .get_available_templates()
                .map_err(|e| anyhow!("Failed to get templates: {e}"))?;

            println!("{templates:?}");

            Ok(())
        }

        _ => unimplemented!(),
    }
}

fn confirm_remove(path: &std::path::Path) -> bool {
    let ans = inquire::Confirm::new(&format!("Do you want to remove {}?", path.display()))
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
