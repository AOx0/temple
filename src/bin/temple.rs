use clap::Parser;
use temple::*;

fn main() {
    let args = Args::parse();
    let temple_files = ConfigFiles::default();

    let result = match args.command {
        Commands::New {
            template_name,
            project_name,
            cli_keys,
            local,
            in_place,
            overwrite,
        } => create_project_from_template(
            &template_name,
            &project_name,
            cli_keys,
            temple_files,
            local,
            in_place,
            overwrite,
        ),
        Commands::List { short, path } => list_available_templates(temple_files, !short, path),
        Commands::Init => init_temple_config_files(temple_files),
    };

    if let Err(msg) = result {
        println!("{msg}")
    }
}
