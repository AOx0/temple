use clap::Parser;
use fs_extra::dir::CopyOptions;
use include_dir::{include_dir, Dir};
use inflector::cases::snakecase::is_snake_case;
use path_absolutize::Absolutize;
use std::env::set_current_dir;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;
use subprocess::Exec;

static TEMPLATE_DIR: Dir = include_dir!("embedded");

/// Simple program to greet a person
#[derive(Parser, Debug)]
struct Args {
    ///Path of the template
    #[clap(required = true)]
    pub template: String,

    ///Name of the binary
    #[clap(required = true)]
    pub name: String,

    ///Name of the default project
    #[clap(required = true)]
    pub default_name: String,

    ///Where to create the project
    #[clap(default_value = ".")]
    pub path: PathBuf,

    ///Replace strings
    #[clap(short, long)]
    pub replaces: Option<Vec<String>>,
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

const CREATE: bool = true;
const COMPILE: bool = true;
const MOVE: bool = true;
const REMOVE: bool = true;

fn main() {
    let mut args = Args::parse();
    args.path = PathBuf::from(args.path.absolutize().unwrap());
    args.path = args.path.join(&format!("{}.tmp", args.name));

    if !is_snake_case(&args.name) {
        eprintln!("Error: Name must follow snake case. Example: new_project");
        exit(1);
    };

    if CREATE
    /* Create temporary file */
    {
        create_dir(&mut args);
    }

    replace_all_keys(&mut args);

    move_to_target_path(&mut args);

    if COMPILE
    /* Compile replaced template */
    {
        Exec::shell("cargo build --release").join().unwrap();
    }

    if MOVE
    /* Move compiled binary to temple/ dir */
    {
        move_binary_to_target_path(&mut args);
    }

    if REMOVE
    /* Remove tmp dir */
    {
        set_current_dir(&args.path.parent().unwrap()).unwrap();
        fs_extra::dir::remove(&args.path).unwrap();
    }
}

fn move_binary_to_target_path(args: &mut Args) {
    fs::copy(
        &args.path.join("target").join("release").join(&args.name),
        PathBuf::from("/Users/alejandro/temple").join(&args.name),
    )
    .unwrap();
}

fn move_to_target_path(args: &mut Args) {
    let options = CopyOptions::new();
    fs_extra::dir::copy(&args.template, &args.path, &options).unwrap();

    set_current_dir(&args.path).unwrap();
}

fn create_dir(args: &mut Args) {
    fs::create_dir(&args.path).unwrap();
    TEMPLATE_DIR.extract(&args.path).unwrap();

    change_string!(args.path.join("main.rs"), "PATH", args.template);
    change_string!(args.path.join("main.rs"), "DEFAULT", args.default_name);
    change_string!(args.path.join("main.rs"), "NAME", args.name);
    change_string!(args.path.join("Cargo.toml"), "NAME", args.name);
}

struct Replacements {
    pub member: String,
    pub key: String,
    pub help: String,
}

impl Replacements {
    fn new_from(pattern: &str) -> Replacements {
        let map: Vec<&str> = pattern.split("...").collect();
        Replacements {
            member: map[0].to_string().to_lowercase(),
            key: map[0].to_string(),
            help: map[1].to_string(),
        }
    }
}

fn get_replacements(args: &mut Args) -> Vec<Replacements> {
    let mut result = Vec::new();
    for r in args.replaces.clone().unwrap() {
        result.push(Replacements::new_from(&r));
    }

    result
}

fn replace_all_keys(args: &mut Args) {
    let mut replaces_result = String::with_capacity(1000);
    let mut replaces_def = String::with_capacity(1000);

    replaces_def.push_str("REPLACE_DEF");

    replaces_result.push_str(
        "
    for file in files {
        let file = file.unwrap();
        if !file.file_type().unwrap().is_dir() {
            REPLACE_RES
        }
    }",
    );

    if args.replaces.is_some() {
        let replacements: Vec<Replacements> = get_replacements(args);

        for replace in replacements {
            replaces_def = replaces_def.replace(
            "REPLACE_DEF",
            &format!(
                    "    /// {}\n    #[clap(short, long, required = true)]\n    pub {}: String,\nREPLACE_DEF",
                    replace.help, replace.member
                )
            );

            replaces_result = replaces_result.replace(
                "REPLACE_RES",
                &format!(
                    "change_string!(file.path(), \"{}\", args.{}); REPLACE_RES",
                    replace.key, replace.member
                ),
            );
        }
    }

    replaces_def = replaces_def.replace("REPLACE_DEF", "");
    replaces_result = replaces_result.replace("REPLACE_RES", "");

    change_string!(args.path.join("main.rs"), "REPLACE_DEF", replaces_def);
    change_string!(args.path.join("main.rs"), "REPLACE_RES", replaces_result);
}
