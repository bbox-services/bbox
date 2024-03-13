use minijinja::{path_loader, Environment};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TemplateDirCfg {
    /// endpoint path for publishing
    pub path: String,
    /// template file directory
    pub dir: String,
}

#[derive(Default)]
pub struct RuntimeTemplates {
    envs: HashMap<String, Environment<'static>>,
}

impl RuntimeTemplates {
    pub fn add(&mut self, dir: &str, path: &str) {
        let mut env = Environment::new();
        env.set_loader(path_loader(dir));
        self.envs.insert(path.to_string(), env);
    }
    pub fn get(&self, path: &str) -> Option<&Environment<'static>> {
        self.envs.get(path)
    }
}
