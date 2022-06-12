use std::{
    cell::RefCell,
    fs::{self, create_dir_all, OpenOptions},
    path::{Path, PathBuf},
    rc::Rc,
    thread::{self, JoinHandle},
};

use temple_core::*;

pub fn render_recursive(
    handler: Rc<RefCell<Vec<JoinHandle<Result<(), String>>>>>,
    dir: &Path,
    target: PathBuf,
    keys: &Keys,
    dip: bool,
    indicators: &Indicators,
) -> Result<(), String> {
    if dir.is_dir() {
        let mut contents = Contents::from(dir.file_name().unwrap().to_str().unwrap());
        let dir_name = contents.replace(indicators, keys);

        if let Err(e) = dir_name {
            return Err(e);
        }

        let dir_name = Contents::get_str_from_result(&dir_name.unwrap().1);

        create_dir_all(if !dip {
            target.parent().unwrap().join(dir_name.as_str())
        } else {
            target.clone()
        })
        .unwrap();

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

                render_recursive(
                    handler.clone(),
                    &path,
                    target.join(replacement.as_str()),
                    keys,
                    false,
                    indicators,
                )?;
            } else {
                if dip && path.file_name().unwrap().to_str().unwrap() == ".temple" {
                    continue;
                }

                let handle = thread::spawn({
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

                            let new = OpenOptions::new()
                                .write(true)
                                .truncate(true)
                                .create(true)
                                .open(target.clone().join(replacement.as_str()))
                                .unwrap();

                            let mut contents = Contents::from_file(
                                path.parent().unwrap().join(path.file_name().unwrap()),
                            )?;

                            let replacement =
                                contents.replace(&Indicators::new("{{ ", " }}").unwrap(), &keys);

                            let result = match replacement {
                                Ok(o) => o,
                                Err(e) => return Err(e),
                            };

                            Contents::write_to_target(&result.1, new);
                        };
                        Ok(())
                    }
                });

                handler.borrow_mut().push(handle);
            }
        }
    }
    Ok(())
}
