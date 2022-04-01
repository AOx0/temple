use std::env::set_current_dir;
use clap::Parser;
use include_dir::{include_dir, Dir};
use inflector::cases::snakecase::is_snake_case;
use path_absolutize::Absolutize;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::exit;
use fs_extra::dir::CopyOptions;
use subprocess::Exec;

static TEMPLATE_DIR: Dir = include_dir!("embedded");

/// Simple program to greet a person
#[derive(Parser, Debug)]
struct Args {
    ///Name of the project
    #[clap(required = true)]
    name: String,

    ///Where to create the project
    #[clap(default_value = ".")]
    path: PathBuf,

    ///Replace strings
    #[clap(short)]
    replaces: Option<Vec<String>>
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

fn set_permissions_for_steamcmd(path: &Path) {
    let files = path.read_dir().unwrap();

    for file in files {
        let file = file.unwrap();

        if !file.file_type().unwrap().is_dir() {
            let mut perms = fs::metadata(file.path()).unwrap().permissions();
            perms.set_mode(0o744);
            std::fs::set_permissions(file.path(), perms).unwrap();
        } else {
            set_permissions_for_steamcmd(&file.path());
        }
    }
}

fn main() {
    let Args { name, path, replaces } = Args::parse();
    let path = path.absolutize().unwrap();
    let path = path.join(&format!("{}.tmp", name));

    if !is_snake_case(&name) {
        eprintln!("Error: Name must follow snake case. Example: new_project");
        exit(1);
    };

    fs::create_dir(&path).unwrap();
    TEMPLATE_DIR.extract(&path).unwrap();

    change_name!(path.join("main.rs"), name);
    change_name!(path.join("Cargo.toml"), name);

    let mut replaces_result = String::with_capacity(1000);

    replaces_result.push_str("
    for file in files {
        let file = file.unwrap();
        if !file.file_type().unwrap().is_dir() {
            REPLACE
        }
    }");

    if replaces.is_some() {
        for r in replaces.unwrap() {
            let map: Vec<&str> = r.split(":::").collect();
            let (key, value) = (map[0], map[1]);
            replaces_result = replaces_result.replace("REPLACE", &format!("change_string!(file.path(), \"{key}\", \"{value}\"); REPLACE"));
        }
    }

    replaces_result = replaces_result.replace("REPLACE", "");

    change_string!(path.join("main.rs"), "REPLACE", replaces_result );

    let options = CopyOptions::new();
    fs_extra::dir::copy(&name, &path, &options).unwrap();

    set_current_dir(&path).unwrap();

    Exec::shell("cargo build --release").join().unwrap();
    
    let name_exe = format!("{}", 
        &name
        .replace("a", "x")
        .replace("e", "x")
        .replace("i", "x")
        .replace("o", "x")
        .replace("u", "x")
    );

    fs::copy(&path.join("target").join("release").join(&name), PathBuf::from("/Users/alejandro/temple").join(name_exe)).unwrap();
    
    set_current_dir(&path.parent().unwrap()).unwrap();

    fs_extra::dir::remove(&path).unwrap();
}
