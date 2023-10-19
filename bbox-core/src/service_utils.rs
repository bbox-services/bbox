use std::collections::HashMap;

/// Store named objects (e.g. datasources)
pub struct NamedObjectStore<T> {
    /// Lookup name -> Object
    lookup: HashMap<String, T>,
    /// Name of default object.
    default: Option<String>,
}

impl<T> Default for NamedObjectStore<T> {
    fn default() -> Self {
        NamedObjectStore {
            lookup: HashMap::new(),
            default: None,
        }
    }
}

impl<T> NamedObjectStore<T> {
    /// Add named object.
    ///
    /// Notice: duplicate names overwrite existing entries.
    pub fn add(&mut self, name: &str, ds: T) {
        self.lookup.insert(name.to_string(), ds);
        //  First one added is default, when not explicitely set.
        self.default.get_or_insert(name.to_string());
    }

    pub fn get(&self, name: &str) -> Option<&T> {
        self.lookup.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut T> {
        self.lookup.get_mut(name)
    }

    pub fn get_default(&self) -> Option<&T> {
        let no_default = "".to_string();
        let name = self.default.as_ref().unwrap_or(&no_default);
        self.get(&name)
    }

    pub fn get_default_mut(&mut self) -> Option<&mut T> {
        let no_default = "".to_string();
        let name = self.default.as_ref().unwrap_or(&no_default).clone();
        self.get_mut(&name)
    }

    pub fn get_or_default(&self, name: Option<&str>) -> Option<&T> {
        if let Some(name) = name {
            self.get(name)
        } else {
            self.get_default()
        }
    }

    pub fn get_or_default_mut(&mut self, name: Option<&str>) -> Option<&mut T> {
        if let Some(name) = name {
            self.get_mut(name)
        } else {
            self.get_default_mut()
        }
    }
}
