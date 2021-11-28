use minijinja::{Environment, Error, Source, State};
use rust_embed::RustEmbed;

fn truncate(_state: &State, value: String, new_len: usize) -> Result<String, Error> {
    let mut s = value.clone();
    s.truncate(new_len);
    Ok(s)
}

trait LoadFromEmbedded {
    fn add_embedded_template<E: RustEmbed>(&mut self, e: &E, fname: &str);
}

impl LoadFromEmbedded for Source {
    fn add_embedded_template<E: RustEmbed>(&mut self, _: &E, fname: &str) {
        let templ = String::from_utf8(E::get(fname).unwrap().to_vec()).unwrap();
        self.add_template(fname, templ).unwrap();
    }
}

pub fn create_env(path: &str, extensions: &[&str]) -> Environment<'static> {
    let mut env = Environment::new();
    env.add_filter("truncate", truncate);
    let mut source = Source::new();
    source.load_from_path(path, extensions).unwrap();
    env.set_source(source);
    env
}

pub fn create_env_embedded<E: RustEmbed>(e: &E) -> Environment<'static> {
    let mut env = Environment::new();
    env.add_filter("truncate", truncate);
    let mut source = Source::new();
    for f in E::iter() {
        source.add_embedded_template(e, &f);
    }
    env.set_source(source);
    env
}
