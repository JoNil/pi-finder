use std::{
    collections::HashSet,
    env, fmt,
    fs::{self},
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) struct Item {
    filename: String,
    dir: PathBuf,
}

impl Item {
    fn new(filename: String, dir: PathBuf) -> Self {
        Self { filename, dir }
    }

    pub(crate) fn execute(&self) {
        Command::new("x-terminal-emulator")
            .arg("-e")
            .arg(format!("{}; $SHELL", self.filename))
            .spawn()
            .ok();
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.filename, self.dir.to_string_lossy())
    }
}

fn walk_dir(dir: impl AsRef<Path>) -> impl Iterator<Item = PathBuf> {
    fs::read_dir(dir.as_ref()).into_iter().flat_map(|i| {
        i.filter_map(|e| e.ok())
            .filter(|p| !p.path().is_dir())
            .map(|p| p.path())
    })
}

struct MatcherOutput {
    primary: Vec<Item>,
    secondary: Vec<Item>,
}

impl MatcherOutput {
    fn output(mut self) -> Vec<Item> {
        self.primary
            .sort_by(|a, b| a.filename.len().cmp(&b.filename.len()));
        self.secondary
            .sort_by(|a, b| a.filename.len().cmp(&b.filename.len()));

        self.primary
            .into_iter()
            .chain(self.secondary.into_iter())
            .take(25)
            .collect()
    }
}

pub(crate) fn get_matching(search_term: &str) -> Vec<Item> {
    let mut res = MatcherOutput {
        primary: Vec::new(),
        secondary: Vec::new(),
    };

    let mut dirs_to_search = Vec::new();
    dirs_to_search.push(Path::new("/usr/share/applications").to_owned());

    #[allow(deprecated)]
    if let Some(home) = env::home_dir() {
        dirs_to_search.push(home.join(".local/share/applications"));
    }

    if let Ok(path) = env::var("PATH") {
        let mut unique_dirs = HashSet::new();
        for entry in path.split(":") {
            if !unique_dirs.contains(&entry) {
                unique_dirs.insert(entry);
                dirs_to_search.push(Path::new(entry).to_owned());
            }
        }
    }

    for dir in dirs_to_search {
        for entry in walk_dir(&dir) {
            if let Some(file_name) = entry.file_name().map(|s| s.to_string_lossy()) {
                if file_name == search_term {
                    res.primary
                        .push(Item::new(file_name.into_owned(), dir.clone()));
                } else if file_name.contains(search_term) {
                    res.secondary
                        .push(Item::new(file_name.into_owned(), dir.clone()));
                }
            }
        }
    }

    return res.output();
}
