use clap::Parser;
use include_dir::{include_dir, Dir};
use inflector::cases::snakecase::is_snake_case;
use path_absolutize::Absolutize;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;

static TEMPLATE_DIR: Dir = include_dir!("NAME");

/// Simple program to greet a person
#[derive(Parser, Debug)]
struct Args {
    ///Name of the project
    #[clap(long, short, default_value = "NAME_t")]
    name: String,

    ///Where to create the project
    #[clap(default_value = ".")]
    path: PathBuf,
}

macro_rules! change_name {
    ($f: expr, $name: expr) => {
        let mut file = fs::File::open(&$f).unwrap();
        let mut contents = vec![];
        file.read_to_end(&mut contents).unwrap();
        let contents = String::from_utf8(contents).unwrap();
        let new_contents = contents
            .replace("NAME", &$name);

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&$f)
            .unwrap();
        file.write_all(new_contents.as_ref()).unwrap();
    };
}


macro_rules! change_string {
    ($f: expr, $from: expr,  $to: expr) => {
        let mut file = fs::File::open(&$f).unwrap();
        let mut contents = vec![];
        file.read_to_end(&mut contents).unwrap();
        let contents = String::from_utf8(contents).unwrap();
        let new_contents = contents
            .replace($from, &$to);

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&$f)
            .unwrap();
        file.write_all(new_contents.as_ref()).unwrap();
    };
}

fn main() {
    let Args { name, path } = Args::parse();
    let path = path.absolutize().unwrap();
    let path = path.join(&name);

    if !is_snake_case(&name) {
        eprintln!("Error: Name must follow snake case. Example: new_project");
        exit(1);
    };

    fs::create_dir(&path).unwrap();
    TEMPLATE_DIR.extract(&path).unwrap();

    let files = path.read_dir().unwrap();

    REPLACE
}
