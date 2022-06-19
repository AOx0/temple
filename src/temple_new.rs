use clap::Parser;
use temple_shared::*;

mod args;
use args::ArgsNew;
fn main() {
    let args = ArgsNew::parse();
    let temple_files = ConfigFiles::default();

    let result = temple_shared::create_project_from_template(
        &args.template_name,
        &args.project_name,
        args.cli_keys,
        temple_files,
        args.local,
        args.in_place,
        args.overwrite,
    );

    if let Err(msg) = result {
        println!("{msg}")
    }
}
