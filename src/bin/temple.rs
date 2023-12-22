use anyhow::Result;
use clap::Parser;
use std::process::ExitCode;
use temple::{
    args::{Args, Commands},
    config::TempleDirs,
};

fn app(args: &Args) -> Result<()> {
    let temple_dirs = TempleDirs::default_paths()?;

    let result = match args.command {
        Commands::List { .. } => temple_dirs.display_available_templates(&args.command),
        Commands::Init => {
            temple_dirs.create_global_dir()?;
            temple_dirs.create_global_config()
        }
        _ => unimplemented!(),
    };

    if let Err(msg) = result {
        println!("{msg}")
    }

    Ok(())
}

fn main() -> ExitCode {
    let args = Args::parse();

    match app(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if !args.errors() {
                eprintln!("Error: {e}");
            }
            ExitCode::FAILURE
        }
    }
}
