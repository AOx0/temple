use anyhow::{anyhow, bail, ensure, Context};
use derive_builder::Builder;
use directories::UserDirs;
use std::path::{Path, PathBuf};

use crate::args::Commands;

#[derive(Builder)]
pub struct TempleDirs {
    user_home: PathBuf,
    global_config: PathBuf,
    local_config: Option<PathBuf>,
}

#[derive(Clone)]
pub struct Template {
    pub path: PathBuf,
    pub name: String,
}

pub struct Templates {
    pub global: Vec<Template>,
    pub local: Vec<Template>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Prefer {
    Local,
    Global,
}

impl Templates {
    #[must_use]
    pub fn get_named(&self, name: &str, prefers: &Prefer) -> Option<&Template> {
        let local = self.local.iter().find(|&t| t.name == name);
        let global = self.global.iter().find(|&t| t.name == name);

        match (local, global) {
            (Some(local), _) if prefers == &Prefer::Local => Some(local),
            (_, Some(global)) if prefers == &Prefer::Global => Some(global),
            (None, Some(found)) | (Some(found), None) => Some(found),
            _ => None,
        }
    }
}

impl TempleDirs {
    /// Creates the global config inside the global directory.
    ///
    /// # Errors
    ///
    /// This function will return an error if the global config directory does not exist or any IO error occurs
    pub fn create_global_config(&self) -> anyhow::Result<()> {
        let dir_exists = self.global_config().exists() && self.global_config().is_dir();

        ensure!(
            dir_exists,
            anyhow!(
                "Global config directory {} does not exist",
                self.global_config().display()
            )
        );

        let config_file = self.global_config().join("temple.conf");
        let exists = config_file.exists();

        if exists && config_file.is_file() {
            println!(
                "The global config file at path {} already exists. Skipping creation.",
                config_file.display()
            );

            return Ok(());
        } else if exists {
            println!(
                "The global config file at path {} exists but is not a file. Removing existing path.",
                config_file.display(),
            );

            Self::remove_path(&config_file)?;
        }

        std::fs::OpenOptions::new()
            .create_new(true)
            .open(config_file)?;

        Ok(())
    }

    /// Creates the global directory.
    ///
    /// # Errors
    ///
    /// This function will return an error if any IO error occurs
    pub fn create_global_dir(&self) -> anyhow::Result<()> {
        let exists = self.global_config().exists();

        if exists && self.global_config().is_dir() {
            println!(
                "The global config directory at path {} already exists. Skipping creation.",
                self.global_config().display()
            );

            return Ok(());
        } else if exists {
            println!(
                "The global config directory at path {} exists but is not a directory. Removing existing path.",
                self.global_config().display(),
            );

            Self::remove_path(self.global_config())?;
        }

        std::fs::create_dir_all(self.global_config())?;

        Ok(())
    }

    pub fn display_available_templates(&self, config: &Commands) -> anyhow::Result<()> {
        if let Commands::List { short, path, .. } = config {
            let long = !short;
            let globals = Self::get_templates_in_dir(&self.global_config)?;
            let locals = if let Some(l) = self
                .local_config
                .as_ref()
                .map(|c| Self::get_templates_in_dir(c.as_path()))
            {
                l?
            } else {
                Vec::default()
            };

            let iter = [globals, locals];

            for (i, iter) in iter.iter().enumerate() {
                (!iter.is_empty()).then(|| {
                    if long {
                        println!(
                            "Available {} templates (def at '{}'): ",
                            if i == 0 { "global" } else { "local" },
                            (i == 0)
                                .then_some(self.global_config())
                                .unwrap_or_else(|| {
                                    iter.first()
                                        .expect("TODO: Fix this")
                                        .path
                                        .parent()
                                        .expect("TODO: Fix this")
                                })
                                .to_str()
                                .unwrap_or("Failed to convert path to str")
                                .replace(
                                    self.user_home()
                                        .to_str()
                                        .unwrap_or("Failed to convert path to str"),
                                    "~"
                                )
                        );
                    }

                    print!(
                        "{}",
                        iter.iter()
                            .map(|a| format!(
                                "{dotchr}{tename}{tepath}{spacer}",
                                dotchr = if long { "    " } else { "" },
                                tename = (long || i == 0)
                                    .then_some(a.name.clone())
                                    .unwrap_or_else(|| format!("local:{}", a.name)),
                                tepath = path
                                    .then_some(format!(
                                        "\t'{}'",
                                        a.path.to_str().unwrap_or("Failed to convert path to str")
                                    ))
                                    .unwrap_or(String::new()),
                                spacer = if long { "\n" } else { " " }
                            )
                            .replace(
                                self.user_home()
                                    .to_str()
                                    .unwrap_or("Failed to convert path to str"),
                                "~"
                            ))
                            .collect::<String>(),
                    );
                    (long || i != 0).then(|| println!());
                });
            }

            Ok(())
        } else {
            bail!("Invalid argument")
        }
    }
}

impl TempleDirs {
    /// Create a new [`TempleDirs`] builder
    #[must_use]
    pub fn builder() -> TempleDirsBuilder {
        TempleDirsBuilder::create_empty()
    }

    pub fn remove_path(path: &Path) -> anyhow::Result<()> {
        let file_type = path.metadata()?.file_type();

        if file_type.is_symlink() {
            // https://stackoverflow.com/questions/76351822/creating-and-removing-symlinks

            #[cfg(target_os = "windows")]
            std::fs::remove_dir(path)?;

            #[cfg(not(target_os = "windows"))]
            std::fs::remove_file(path)?;
        } else if file_type.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else if file_type.is_file() {
            std::fs::remove_file(path)?;
        } else {
            unreachable!("Path {} not a symlink, file or dir", path.display())
        }

        Ok(())
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

    /// Get a [`Vec`] of all templates inside a given directory. A path is considered
    /// to be a template if:
    /// - The path is a directory
    /// - It contains a `.temple` file
    ///
    /// # Errors
    ///
    /// This function will return an error if any IO error happens while getting path
    /// file types or reding entries in directories or the path is not a dir.
    pub fn get_templates_in_dir(path: &Path) -> anyhow::Result<Vec<Template>> {
        ensure!(
            path.is_dir(),
            anyhow!("Path {} is not a directory", path.display())
        );
        let mut res = Vec::new();

        for entry in path.read_dir()? {
            let entry = entry?;
            let root = entry.path().join(".temple");

            if entry.file_type()?.is_dir() && (root.exists() && root.is_file()) {
                res.push(Template {
                    name: entry
                        .file_name()
                        .to_str()
                        .context("Failed to convert OsString to String")?
                        .to_string(),
                    path: entry.path(),
                });
            }
        }

        Ok(res)
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
