use std::path::PathBuf;
pub struct ConfigFiles {
    pub temple_home: PathBuf,
    pub temple_config: PathBuf,
}

impl ConfigFiles {
    pub fn new() -> Self {
        let temple_home = directories::UserDirs::new()
            .unwrap()
            .home_dir()
            .join(".temple");
        let temple_config = directories::UserDirs::new()
            .unwrap()
            .home_dir()
            .join(".temple_conf");

        ConfigFiles {
            temple_home,
            temple_config,
        }
    }

    pub fn exists(&self) -> Result<(), String> {
        if !self.temple_home.exists() || !self.temple_home.exists() {
            Err(
                "Error: No \"~/.temple\" or \".temple_conf\".\n    Run `temple init` to create them"
                    .into(),
            )
        } else {
            Ok(())
        }
    }
}

impl Default for ConfigFiles {
    fn default() -> Self {
        Self::new()
    }
}
