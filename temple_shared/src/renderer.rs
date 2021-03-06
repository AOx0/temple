use std::{
    cell::RefCell,
    fs::{self, create_dir_all, OpenOptions},
    path::{Path, PathBuf},
    rc::Rc,
    thread::{self, JoinHandle},
};

use temple_core::*;

pub type Mu<T> = Rc<RefCell<T>>;
pub fn render_recursive(
    handler: Mu<Vec<JoinHandle<Result<(), String>>>>,
    dir: &Path,
    target: PathBuf,
    keys: &Keys,
    dip: bool,
    indicators: &Indicators,
    dry_run: bool,
    overwrite: bool,
    in_place: bool,
) -> Result<(), String> {
    if dir.is_dir() {
        let mut contents = if dip && !in_place {
            let project_template =  format!("{}project{}", std::string::String::from(indicators.start.clone()),  std::string::String::from(indicators.end.clone()) );
            // println!("{project_template}");
            Contents::from(project_template.as_str())
        } else {
            Contents::from(dir.file_name().unwrap().to_str().unwrap())
        };

        let dir_name = contents.replace(indicators, keys);

        if let Err(e) = dir_name {
            return Err(e);
        }

        let dir_name = Contents::get_str_from_result(&dir_name.unwrap().1);

        if !overwrite
            && !in_place
            && dip
            && target.parent().unwrap().join(dir_name.as_str()).exists()
        {
            return Err(format!(
                "Error: directory {} already exists",
                target
                    .parent()
                    .unwrap()
                    .join(dir_name.as_str())
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
            )
            .into());
        }

        if !dry_run {
            create_dir_all(if !dip {
                target.parent().unwrap().join(dir_name.as_str())
            } else {
                target.clone()
            })
            .unwrap()
        }

        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                let mut contents = Contents::from(path.file_name().unwrap().to_str().unwrap());
                let replacement = contents.replace(indicators, keys);

                if let Err(e) = replacement {
                    return Err(e);
                }

                let replacement = Contents::get_str_from_result(&replacement.unwrap().1);

                if !dry_run {
                    render_recursive(
                        handler.clone(),
                        &path,
                        target.join(replacement.as_str()),
                        keys,
                        false,
                        indicators,
                        dry_run,
                        overwrite,
                        in_place,
                    )?;
                }
            } else {
                let name = path.file_name().unwrap().to_str().unwrap();
                if dip && name == ".temple" {
                    continue;
                }

                if [".DS_Store"].contains(&name) {
                    continue;
                }

                let actions = {
                    let indicators = indicators.to_owned();
                    let keys = keys.to_owned();
                    let target = target.to_owned();

                    move || {
                        {
                            let mut contents =
                                Contents::from(path.file_name().unwrap().to_str().unwrap());
                            let replacement = contents.replace(&indicators, &keys);
                            
                            
                            if let Err(e) = replacement {
                                return Err(e);
                            }


                            let replacement =
                                Contents::get_str_from_result(&replacement.unwrap().1);

                            // println!("Replacing in {} file name {} to {}", target.display(), path.file_name().unwrap().to_str().unwrap(), replacement );


                            if !overwrite
                                && dip
                                && target.clone().join(replacement.as_str()).exists()
                            {
                                return Err(format!(
                                    "Error: file {} already exists",
                                    target
                                        .clone()
                                        .join(replacement.as_str())
                                        .file_name()
                                        .unwrap()
                                        .to_str()
                                        .unwrap()
                                )
                                .into());
                            }

                            if !dry_run {
                                let new = OpenOptions::new()
                                    .write(true)
                                    .truncate(true)
                                    .create(!dry_run)
                                    .open(target.clone().join(replacement.as_str()))
                                    .unwrap();

                                let mut contents = Contents::from_file(
                                    path.parent().unwrap().join(path.file_name().unwrap()),
                                )?;

                                let replacement = contents
                                    .replace(&indicators, &keys);


                                let result = match replacement {
                                    Ok(o) => o,
                                    Err(e) => return Err(e),
                                };

                                //println!("Replacing contents of \"{}\".", path.parent().unwrap().join(path.file_name().unwrap()).display());

                                Contents::write_to_target(&result.1, new);
                            }
                        };
                        Ok(())
                    }
                };

                if !dry_run {
                    let handle = thread::spawn(actions);

                    handler.borrow_mut().push(handle);
                } else {
                    actions()?;
                }
            }
        }
    }
    Ok(())
}
