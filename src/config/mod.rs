use anyhow::{anyhow, bail, ensure, Context, Ok};
use directories::UserDirs;
use std::path::{Path, PathBuf};

use crate::{args::Commands, info, warn};

pub struct TempleDirs {
    user_home: PathBuf,
    global_config: PathBuf,
    local_config: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct Template(pub PathBuf);

impl Template {
    #[must_use]
    pub fn name(&self) -> &str {
        self.file_name()
            .and_then(|name| name.to_str())
            .expect("Failed to get path file_name")
    }
}

impl std::ops::Deref for Template {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Template {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
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
        let local = self
            .local
            .iter()
            .find(|&t| t.file_name().is_some_and(|n| n == name));
        let global = self
            .global
            .iter()
            .find(|&t| t.file_name().is_some_and(|n| n == name));

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
    pub fn create_global_config(&self) -> anyhow::Result<Option<std::fs::File>> {
        self.create_config_file(self.global_config())
    }

    pub fn create_config_file(&self, inside: &Path) -> anyhow::Result<Option<std::fs::File>> {
        crate::trace!("Checking if config dir {} exists", inside.display());

        let dir_exists = inside.exists() && inside.is_dir();

        ensure!(
            dir_exists,
            anyhow!("Config directory {} does not exist", inside.display())
        );

        let config_file = inside.join("config.tpl");
        let exists = config_file.exists();

        if exists && config_file.is_file() {
            info!(
                "The config file at path {} already exists. Skipping creation.",
                config_file.display()
            );

            return Ok(None);
        } else if exists {
            warn!(
                "The config file at path {} exists but is not a file. Removing existing path.",
                config_file.display(),
            );

            Self::remove_path(&config_file)?;
        }

        crate::info!("Creating configuration file at {}", config_file.display());

        Ok(Some(
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(config_file)?,
        ))
    }

    /// Creates the global directory.
    ///
    /// # Errors
    ///
    /// This function will return an error if any IO error occurs
    pub fn create_global_dir(&self) -> anyhow::Result<()> {
        self.create_config_dir(self.global_config())
    }

    pub fn create_config_dir(&self, at: &Path) -> anyhow::Result<()> {
        crate::trace!("Checking if directory {} exists", at.display());

        let exists = at.exists();

        if exists && at.is_dir() {
            info!(
                "The config directory at path {} already exists. Skipping creation.",
                at.display()
            );

            return Ok(());
        } else if exists {
            warn!(
                "The config directory at path {} exists but is not a directory. Removing existing path.",
                at.display(),
            );

            Self::remove_path(at)?;
        }

        crate::info!("Creating config directory {}", at.display());

        std::fs::create_dir_all(at)?;

        Ok(())
    }

    pub fn get_available_templates(&self) -> anyhow::Result<Templates> {
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

        Ok(Templates {
            global: globals,
            local: locals,
        })
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
                                    .then_some(a.name().to_string())
                                    .unwrap_or_else(|| format!("local:{}", a.name())),
                                tepath = path
                                    .then_some(format!(
                                        "\t'{}'",
                                        a.to_str().unwrap_or("Failed to convert path to str")
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
        crate::trace!("Looking for local configs");

        ensure!(
            relative_to.is_dir(),
            anyhow!("Path {} is not a directory", relative_to.display())
        );

        let mut curr = Some(relative_to);

        while let Some(parent) = curr {
            for opt in [".temple", "temple"] {
                let path = parent.join(opt);

                crate::trace!("Looking at {}", path.display());

                if path.exists() && path.is_dir() {
                    for opt in ["config.tpl", "config.temple"] {
                        let config = path.join(opt);
                        if config.exists() && config.is_file() {
                            return Ok(Some(path));
                        }
                    }
                }
            }
            curr = parent.parent();
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
            let root = entry.path().join("config.tpl");
            let root2 = entry.path().join("config.temple");

            if entry.file_type()?.is_dir()
                && ((root.exists() && root.is_file()) || (root2.exists() && root2.is_file()))
            {
                res.push(Template(entry.path()));
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
