use clap::Parser;
use include_dir::{include_dir, Dir};
use inflector::cases::snakecase::is_snake_case;
use path_absolutize::Absolutize;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;

static TEMPLATE_DIR: Dir = include_dir!("PATH");

/// DESCRIPTION
#[derive(Parser, Debug)]
#[clap(about, long_about = None)]
struct Args {
    /// Name of the project
    #[clap(long, short, default_value = "DEFAULT")]
    name: String,

    /// Where to create the project
    #[clap(default_value = ".")]
    path: PathBuf,

    REPLACE_DEF
}

fn replace<T>(source: &[T], from: &[T], to: &[T]) -> Vec<T>
    where
        T: Clone + PartialEq,
{
    let mut result = source.to_vec();
    let from_len = from.len();
    let to_len = to.len();

    let mut i = 0;
    while i + from_len <= result.len() {
        if result[i..].starts_with(from) {
            result.splice(i..i + from_len, to.iter().cloned());
            i += to_len;
        } else {
            i += 1;
        }
    }

    result
}

#[allow(unused_macros)]
macro_rules! change_string {
    ($f: expr, $from: expr,  $to: expr) => {
        let mut file = fs::File::open(&$f).unwrap();
        let mut contents = vec![];
        file.read_to_end(&mut contents).unwrap();

        let new_contents = replace::<u8>(&contents[..], $from.as_bytes(), $to.as_bytes());

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&$f)
            .unwrap();
        file.write_all(new_contents.as_ref()).unwrap();
    };
}

fn main() {
    let args: Args = Args::parse();
    let path = args.path.absolutize().unwrap();
    let path = path.join(&args.name);

    if !is_snake_case(&args.name) {
        eprintln!("Error: Name must follow snake case. Example: new_project");
        exit(1);
    };

    fs::create_dir(&path).unwrap();
    TEMPLATE_DIR.extract(&path).unwrap();

    let files = path.read_dir().unwrap();

    REPLACE_RES
}
