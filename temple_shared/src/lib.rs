use std::{
    borrow::BorrowMut, cell::RefCell, env::current_dir, fs::OpenOptions, io::Write, rc::Rc,
    thread::JoinHandle,
};

pub use config_files::ConfigFiles;
use fs_extra::dir::create_all;
pub use temple_core::String;
use temple_core::*;

mod config_files;
mod renderer;

pub fn init_temple_config_files(config_files: ConfigFiles) -> Result<(), String> {
    create_all(&config_files.temple_home, true).unwrap();
    let mut conf = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(config_files.temple_config)
        .unwrap();

    let default_config = "\
            name=Your name;\n\
            github=your_github_user;\n"
        .as_bytes();

    conf.write_all(default_config).unwrap();

    println!("Created ~/.temple_conf file and ~/.temple dir");

    Ok(())
}

pub fn list_available_templates(config_files: ConfigFiles) -> Result<(), String> {
    config_files.exists()?;

    let contents = config_files.temple_home.read_dir().unwrap();
    let mut available: Vec<String> = vec![];

    for c in contents {
        let c = c.unwrap();

        if c.file_type().unwrap().is_dir() && c.path().join(".temple").exists() {
            available.push(c.file_name().as_os_str().to_str().unwrap().into())
        }
    }

    if available.is_empty() {
        println!("No available templates. To add templates add them in ~/.temple.");
    } else {
        println!("Available templates: ");
        available.iter().for_each(|a| println!("   * {}", a));
    }

    Ok(())
}

pub fn create_project_from_template(
    template_name: &str,
    project_name: &str,
    cli_keys: Vec<String>,
    config_files: ConfigFiles,
) -> Result<(), String> {
    config_files.exists()?;

    let home = config_files.temple_home;
    let config = config_files.temple_config;
    let handles = Rc::new(RefCell::new(vec![]));

    let template = home.join(template_name);

    if template.is_dir() && template.join(".temple").exists() {
        let keys_project_config = Keys::from_file_contents(&template.join(".temple"));
        let keys_project_user = Keys::from(cli_keys.join(" ").as_str());
        let mut project_keys = Keys::from(format!("project={}", &project_name).as_str());

        project_keys.add(keys_project_user);
        project_keys.add(keys_project_config);
        project_keys.add(Keys::from_file_contents(&config));

        let start = project_keys
            .get_match("start_indicator", None)
            .unwrap_or("{{ ");
        let end = project_keys
            .get_match("start_indicator", None)
            .unwrap_or(" }}");

        let indicators = &Indicators::new(start, end).unwrap();

        if let Err(e) = renderer::render_recursive(
            handles.clone(),
            &template,
            current_dir().unwrap().join(project_name),
            &project_keys,
            true,
            indicators,
        ) {
            fs_extra::dir::remove(current_dir().unwrap().join(project_name)).unwrap();
            return Err(format!("Error: {}", e).into());
        }

        let handlers = Rc::try_unwrap(handles)
            .expect("I hereby claim that my_ref is exclusively owned")
            .into_inner();

        for handler in handlers {
            let res = handler.join();
            if let Err(error) = res {
                return Err(format!("Error: {:?}", error).into());
            } else if let Ok(res) = res {
                if let Err(error) = res {
                    return Err(format!("Error: {:?}", error).into());
                }
            }
        }
    } else {
        return Err("Error: Template does not exist".into());
    }

    Ok(())
}
