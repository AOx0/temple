use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::{path::PathBuf, process::ExitCode};
use temple::{
    args::{Args, Commands, InitOpt},
    config::TempleDirs,
    error, info,
    replacer::Contents,
    trace,
    values::Values,
};

fn app(args: &Args) -> Result<()> {
    let temple_dirs = TempleDirs::default_paths()?;

    trace!("Global config: {}", temple_dirs.global_config().display());
    trace!(
        "Local config: {}",
        temple_dirs
            .local_config()
            .map(|e| e.display())
            .unwrap_or(std::path::PathBuf::from("None").display())
    );

    match args.command {
        Commands::List { .. } => temple_dirs.display_available_templates(&args.command),
        Commands::Init { sub } => match sub {
            InitOpt::Global => {
                temple_dirs.create_global_dir()?;
                temple_dirs
                    .create_global_config()?
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
                std::fs::remove_dir_all(temple_dirs.global_config()).map_err(|e| anyhow!("{e}"))
            }
            temple::args::DeinitOpt::Local => {
                if let Some(dir) = temple_dirs.local_config() {
                    std::fs::remove_dir_all(dir).map_err(|e| anyhow!("{e}"))
                } else {
                    anyhow::bail!("No local config")
                }
            }
        },
        Commands::DebugConfig { ref path } => {
            trace!("Reading: {}", path.display());

            let contents = std::fs::read_to_string(path)?;

            let values = Values::from_str(&contents, path).map_err(|err| {
                eprintln!("{err:?}");
                anyhow!("Failed to parse values from {}", path.display())
            })?;

            println!("{:#?}", values);

            values.verify_types().map_err(|err| {
                error!(err);
                anyhow!("Invalid types")
            })
        }
        Commands::New {
            ref template_name,
            ref project_name,
            ref cli_keys,
            ref local,
            ref in_place,
            ref overwrite,
        } => {
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

            let contents = " Hola ma llamo {{ name }} y {{ if xp == 4 }}soy nuevo{{ else }}soy experimentado{{}} en esto";

            let contents = Contents {
                contents: contents.to_string(),
                origin: PathBuf::default(),
            };

            let templates = temple_dirs
                .get_available_templates()
                .map_err(|e| anyhow!("Failed to get templates: {e}"))?;

            println!("{templates:?}");

            Ok(())
        }

        _ => unimplemented!(),
    }
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
