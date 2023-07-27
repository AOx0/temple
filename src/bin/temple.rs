use clap::Parser;
use temple::*;

fn main() {
    let args = Args::parse();
    let temple_files = ConfigFiles::get();

    let result = match args.command {
        Commands::New {
            template_name,
            project_name,
            cli_keys,
            local,
            in_place,
            overwrite,
        } => create_project_from_template(
            &template_name.trim_start_matches("local:"),
            &project_name,
            cli_keys,
            temple_files,
            local || template_name.starts_with("local:"),
            in_place,
            overwrite,
        ),
        Commands::List { short, path } => temple_files.list_available_templates(!short, path),
        Commands::Init => temple_files.init_temple_config_files(),
        Commands::ListArgs {
            template_name,
            local,
        } => get_template_keys(
            &template_name.trim_start_matches("local:"),
            local || template_name.starts_with("local:"),
            temple_files,
        ),
    };

    if let Err(msg) = result {
        println!("{msg}")
    }
}
