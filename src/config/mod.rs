use anyhow::{anyhow, ensure, Context};
use derive_builder::Builder;
use directories::UserDirs;
use std::path::{Path, PathBuf};

use crate::template::Template;

#[derive(Builder)]
pub struct TempleDirs {
    user_home: PathBuf,
    global_config: PathBuf,
    local_config: Option<PathBuf>,
}

impl TempleDirs {
    /// Create a new [`TempleDirs`] builder
    #[must_use]
    pub fn builder() -> TempleDirsBuilder {
        TempleDirsBuilder::create_empty()
    }

    /// Attempt to create a new [`TempleDirs`] instance with sane defaults for
    /// path locations
    ///
    /// # Errors
    ///
    /// Returns an [`Err`] if .
    pub fn default_paths() -> anyhow::Result<Self> {
        let home = Self::get_user_home()?;
        Ok(Self {
            global_config: Self::get_config_dir(&home)?,
            local_config: Self::get_local_dir(Self::get_current_dir()?.as_path())?,
            user_home: home,
        })
    }

    /// Returns the path for the user home `~/`
    ///
    /// # Errors
    ///
    /// Returns an [`Err`] if a path for the users home can not
    /// be found
    pub fn get_user_home() -> anyhow::Result<PathBuf> {
        Ok(UserDirs::new()
            .context("Failed to get user's home directory")?
            .home_dir()
            .to_owned())
    }

    /// Returns the path where the global templates and config live
    ///
    /// Looks for the global configuration dir, in order:
    /// - `$XDG_CONFIG_HOME/temple`
    /// - `~/.config/temple`
    /// - `~/.temple`
    ///
    /// # Errors
    ///
    /// Returns an [`Err`] if the path where the config directory should
    /// exist contains anything but a directory (E.g. a file)
    pub fn get_config_dir(home: &Path) -> anyhow::Result<PathBuf> {
        let config_home = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or(home.join(".config"));

        let config = if config_home.exists() {
            config_home.join("temple")
        } else {
            home.join(".temple")
        };

        ensure!(
            config.is_dir(),
            anyhow!("Path {} is not a directory", config.display())
        );

        Ok(config)
    }

    /// Returns the current working directory as a [`PathBuf`]
    ///
    /// # Errors
    ///
    /// Returns an [`Err`] if the current working directory value is invalid.
    /// Possible cases:
    ///
    /// * Current directory does not exist.
    /// * There are insufficient permissions to access the current directory.
    pub fn get_current_dir() -> anyhow::Result<PathBuf> {
        Ok(std::env::current_dir()?)
    }

    /// Looks for any local
    ///
    /// # Errors
    ///
    /// Returns an [`Err`] if any IO error happens while getting path
    /// file types or reding entries in directories.
    pub fn get_local_dir(relative_to: &Path) -> anyhow::Result<Option<PathBuf>> {
        ensure!(
            relative_to.is_dir(),
            anyhow!("Path {} is not a directory", relative_to.display())
        );

        let mut curr = relative_to;

        for entry in curr.read_dir()? {
            let entry = entry?;
            if entry.file_type()?.is_dir()
                && (entry.file_name() == ".temple" || entry.file_name() == "temple")
            {
                return Ok(Some(entry.path()));
            } else if let Some(parent) = curr.parent() {
                curr = parent;
            } else {
                return Ok(None);
            }
        }

        Ok(None)
    }

    /// Returns a reference to the user home of this [`TempleDirs`].
    #[must_use]
    pub fn user_home(&self) -> &Path {
        self.user_home.as_path()
    }

    /// Returns a reference to the global config of this [`TempleDirs`].
    #[must_use]
    pub fn global_config(&self) -> &Path {
        self.global_config.as_path()
    }

    /// Returns the local config of this [`TempleDirs`].
    #[must_use]
    pub fn local_config(&self) -> Option<&Path> {
        if let Some(ref p) = self.local_config {
            Some(p.as_path())
        } else {
            None
        }
    }
}
