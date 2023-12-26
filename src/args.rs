use std::path::PathBuf;

pub use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(version)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

impl Args {
    #[must_use]
    pub fn no_errors(&self) -> bool {
        match self.command {
            Commands::List { errors, .. } => !errors,
            Commands::Init | Commands::New { .. } | Commands::DebugConfig { .. } => false,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new project from a template
    New {
        /// Name of the template
        template_name: String,

        /// Name of the project
        project_name: String,

        /// Custom defined keys from terminal
        #[clap(default_value = "")]
        cli_keys: Vec<String>,

        /// Prefer local (./.temple/template_name) if available [default: prefer ~/.temple/template_name]
        #[clap(long, short)]
        local: bool,

        /// Place contents in_place (./.) instead of creating a folder
        #[clap(long, short)]
        in_place: bool,

        /// Overwrite any already existing files
        #[clap(long, short)]
        overwrite: bool,
    },
    /// List existing templates
    List {
        /// Show templates in a single space separated list
        #[clap(long, short)]
        short: bool,
        /// Show templates path
        #[clap(long, short)]
        path: bool,
        /// Don't show error messages
        #[clap(long, short)]
        errors: bool,
    },
    /// Parse and dump objects to stdout
    DebugConfig {
        /// The path to the configuration file
        path: PathBuf,
    },
    /// Initialize temple configuration directory
    Init,
}
