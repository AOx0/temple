use anyhow::{anyhow, Result};
use clap::Parser;
use owo_colors::OwoColorize;
use std::process::ExitCode;
use temple::{
    args::{Args, Commands},
    config::TempleDirs,
    values::Values,
};

fn app(args: &Args) -> Result<()> {
    let temple_dirs = TempleDirs::default_paths()?;

    match args.command {
        Commands::List { .. } => temple_dirs.display_available_templates(&args.command),
        Commands::Init => {
            temple_dirs.create_global_dir()?;
            temple_dirs.create_global_config()
        }
        Commands::DebugConfig { ref path } => {
            println!("Reading: {}", path.display());

            let contents = std::fs::read_to_string(path)?;

            let values = Values::from_str(&contents, path).map_err(|err| {
                eprintln!("{err:?}");
                anyhow!("Failed to parse values from {}", path.display())
            })?;
            println!("{:?}", values);

            values.verify_types().map_err(|err| {
                eprintln!("{err:?}");
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
                eprintln!(
                    "{}: {e}",
                    "error".if_supports_color(owo_colors::Stream::Stdout, |s| s
                        .style(owo_colors::Style::new().bold().red()))
                );
            }
            ExitCode::FAILURE
        }
    }
}
