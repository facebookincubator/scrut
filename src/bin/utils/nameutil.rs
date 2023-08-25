use std::borrow::Cow;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

#[derive(Default)]
pub(crate) struct UniqueNamer {
    names: HashSet<PathBuf>,
    directory: PathBuf,
}

impl UniqueNamer {
    pub fn new(directory: &Path) -> Self {
        UniqueNamer {
            names: HashSet::new(),
            directory: directory.into(),
        }
    }

    pub fn next_name<'a>(&mut self, name: &'a Path) -> Cow<'a, Path> {
        if !self.names.contains(name) && !Path::new(&self.directory).join(name).exists() {
            self.names.insert(name.into());
            return Cow::from(name);
        }
        let mut counter = 1;
        let mut next_name: PathBuf = format!("{}-{}", name.to_string_lossy(), counter).into();
        while self.names.contains(&next_name)
            || Path::new(&self.directory).join(&next_name).exists()
        {
            counter += 1;
            next_name = format!("{}-{}", name.to_string_lossy(), counter).into();
        }
        self.names.insert(next_name.to_owned());
        next_name.into()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tempdir::TempDir;

    use super::UniqueNamer;

    #[test]
    fn generate_names_within_directory() {
        let tempdir = TempDir::new("temp").expect("create temporary directory");
        let mut namer = UniqueNamer::new(tempdir.path());

        let prefix = Path::new("name");
        let name1 = namer.next_name(prefix);
        assert_eq!(name1, Path::new("name"));

        let name2 = namer.next_name(prefix);
        assert_eq!(name2, Path::new("name-1"));

        let name3 = namer.next_name(prefix);
        assert_eq!(name3, Path::new("name-2"));

        let name4 = namer.next_name(prefix);
        assert_eq!(name4, Path::new("name-3"));
    }
}
