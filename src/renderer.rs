use std::{
    fs::{self, create_dir_all, OpenOptions},
    path::{Path, PathBuf},
    str::FromStr,
};

use super::*;

#[allow(clippy::too_many_arguments)]
pub fn render_recursive(
    target_dir: &Path,
    target: PathBuf,
    keys: &Keys,
    first_level: bool,
    indicators: Indicators<'_>,
    dry_run: bool,
    overwrite: bool,
    in_place: bool,
) -> Result<(), String> {
    if target_dir.is_dir() {
        let mut contents = if first_level && !in_place {
            Contents::from_str(&format!("{}project{}", indicators.start, indicators.end))
        } else {
            Contents::from_str(target_dir.file_name().unwrap().to_str().unwrap())
        }
        .unwrap();

        let mut buff = String::new();

        contents.replace(indicators, keys)?.extend_str(&mut buff);

        if !overwrite
            && !in_place
            && first_level
            && target.parent().unwrap().join(buff.as_str()).exists()
        {
            return Err(format!(
                "Error: directory {} already exists",
                target
                    .parent()
                    .unwrap()
                    .join(buff.as_str())
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
            ));
        }

        if !dry_run {
            create_dir_all(if !first_level {
                target.parent().unwrap().join(buff.as_str())
            } else {
                target.clone()
            })
            .unwrap()
        }

        for entry in fs::read_dir(target_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                let mut contents =
                    Contents::from_str(path.file_name().unwrap().to_str().unwrap()).unwrap();
                contents.replace(indicators, keys)?.extend_str(&mut buff);

                if !dry_run {
                    render_recursive(
                        &path,
                        target.join(buff.as_str()),
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
                if first_level && name == ".temple" {
                    continue;
                }

                if [".DS_Store"].contains(&name) {
                    continue;
                }

                let indicators = indicators.to_owned();
                let keys = keys.to_owned();
                let target = target.to_owned();

                let mut contents =
                    Contents::from_str(path.file_name().unwrap().to_str().unwrap()).unwrap();
                contents.replace(indicators, &keys)?.extend_str(&mut buff);

                // println!("Replacing in {} file name {} to {}", target.display(), path.file_name().unwrap().to_str().unwrap(), replacement );

                if !overwrite && first_level && target.clone().join(buff.as_str()).exists() {
                    return Err(format!(
                        "Error: file {} already exists",
                        target
                            .clone()
                            .join(buff.as_str())
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                    ));
                }

                if !dry_run {
                    let new = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(!dry_run)
                        .open(target.clone().join(buff.as_str()))
                        .unwrap();

                    let mut contents = Contents::from_file(
                        path.parent()
                            .unwrap()
                            .join(path.file_name().unwrap())
                            .as_path(),
                    )?;

                    contents.replace(indicators, &keys)?.write_to_file(new);

                    //println!("Replacing contents of \"{}\".", path.parent().unwrap().join(path.file_name().unwrap()).display());
                }
            }
        }
    }
    Ok(())
}
