use temple_shared::*;

fn main() {
    let temple_files = ConfigFiles::default();

    let result = temple_shared::list_available_templates(temple_files);

    if let Err(msg) = result {
        println!("{msg}")
    }
}
