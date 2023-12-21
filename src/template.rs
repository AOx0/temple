use std::path::PathBuf;

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
    pub fn get_named(&self, name: &str, prefers: Prefer) -> Option<&Template> {
        let local = self.local.iter().find(|&t| t.name == name);
        let global = self.global.iter().find(|&t| t.name == name);

        match (local, global) {
            (Some(local), _) if prefers == Prefer::Local => Some(local),
            (_, Some(global)) if prefers == Prefer::Global => Some(global),
            (None, Some(found)) | (Some(found), None) => Some(found),
            _ => None,
        }
    }
}
