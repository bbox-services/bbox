use minijinja::{Environment, Source};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
pub struct TemplateDirCfg {
    /// endpoint path for publishing
    pub path: String,
    /// template file directory
    pub dir: String,
}

pub struct RuntimeTemplates {
    envs: HashMap<String, Environment<'static>>,
}

// static RUNTIME_TEMPLATE_ENV: Lazy<Environment<'static>> = Lazy::new(|| {
//     let mut env = Environment::new();
//     env.set_source(Source::with_loader(|name| {
//         if name == "layout.html" {
//             Ok(Some("...".into()))
//         } else {
//             Ok(None)
//         }
//     }));
//     env
// });

impl RuntimeTemplates {
    pub fn new() -> Self {
        Self {
            envs: HashMap::new(),
        }
    }
    pub fn add(&mut self, dir: &str) {
        let mut env = Environment::new();
        // env.set_source(Source::with_loader(|name| {
        //     if name == "layout.html" {
        //         Ok(Some("...".into()))
        //     } else {
        //         Ok(None)
        //     }
        // }));
        self.envs.insert(dir.to_string(), env);
    }
    pub fn is_empty(&self) -> bool {
        self.envs.is_empty()
    }
}

struct TemplateDir {
    path: Path,
}

// impl TemplateDir {
//     fn loader(&self, name &str) -> Result<Option<String>> {

//     }
// }
