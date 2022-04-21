pub use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(version)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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
