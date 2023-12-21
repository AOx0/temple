use anyhow::Result;
use clap::Parser;
use std::process::ExitCode;
use temple::{
    args::{Args, Commands},
    config::TempleDirs,
};

fn app() -> Result<()> {
    let args = Args::parse();
    let temple_dirs = TempleDirs::default_paths()?;

    let result = match args.command {
        Commands::List { .. } => temple_dirs.display_available_templates(args.command),
        _ => unimplemented!(),
    };

    if let Err(msg) = result {
        println!("{msg}")
    }

    Ok(())
}

fn main() -> ExitCode {
    match app() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
