use std::collections::HashSet;
use std::path::Path;

#[derive(Default)]
pub(crate) struct UniqueNamer {
    names: HashSet<String>,
    directory: String,
}

impl UniqueNamer {
    pub fn new(directory: &str) -> Self {
        UniqueNamer {
            names: HashSet::new(),
            directory: directory.into(),
        }
    }

    pub fn next_name(&mut self, name: &str) -> String {
        if !self.names.contains(name) && !Path::new(&self.directory).join(name).exists() {
            self.names.insert(name.to_string());
            return name.to_string();
        }
        let mut counter = 1;
        let mut next_name = format!("{}-{}", name, counter);
        while self.names.contains(&next_name)
            || Path::new(&self.directory).join(&next_name).exists()
        {
            counter += 1;
            next_name = format!("{}-{}", name, counter);
        }
        self.names.insert(next_name.to_owned());
        next_name
    }
}
