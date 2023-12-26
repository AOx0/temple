use anyhow::{anyhow, Result};
use clap::Parser;
use std::process::ExitCode;
use temple::{
    args::{Args, Commands},
    config::TempleDirs,
    error, info, trace,
    values::Values,
};

fn app(args: &Args) -> Result<()> {
    let temple_dirs = TempleDirs::default_paths()?;

    match args.command {
        Commands::List { .. } => temple_dirs.display_available_templates(&args.command),
        Commands::Init => {
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
