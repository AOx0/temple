use temple_shared::*;

fn main() {
    let temple_files = ConfigFiles::default();

    let result = temple_shared::init_temple_config_files(temple_files);

    if let Err(msg) = result {
        println!("{msg}")
    }
}
