use minijinja::{Environment, Error, Source, State};

fn truncate(_state: &State, value: String, new_len: usize) -> Result<String, Error> {
    let mut s = value.clone();
    s.truncate(new_len);
    Ok(s)
}

pub fn create_env(path: &str, extensions: &[&str]) -> Environment<'static> {
    let mut env = Environment::new();
    env.add_filter("truncate", truncate);
    let mut source = Source::new();
    source.load_from_path(path, extensions).unwrap();
    env.set_source(source);
    env
}
