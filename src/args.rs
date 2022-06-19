pub use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(version)]
pub struct ArgsNew {
    /// Name of the template
    pub template_name: String,

    /// Name of the project
    pub project_name: String,

    /// Custom defined keys from terminal
    #[clap(default_value = "")]
    pub cli_keys: Vec<String>,

    /// Prefer local (./.temple/template_name) if available [default: prefer ~/.temple/template_name]
    #[clap(long, short)]
    pub local: bool,
}

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
        template_name: String,

        /// Name of the project
        project_name: String,

        /// Custom defined keys from terminal
        #[clap(default_value = "")]
        cli_keys: Vec<String>,

        /// Prefer local (./.temple/template_name) if available [default: prefer ~/.temple/template_name]
        #[clap(long, short)]
        local: bool,
    },
    List,
    Init,
}
