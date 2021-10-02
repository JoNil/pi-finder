use std::{
    collections::HashSet,
    env,
    fs::{self, DirEntry},
    path::Path,
};

fn walk_dir(dir: impl AsRef<Path>) -> impl Iterator<Item = DirEntry> {
    fs::read_dir(dir.as_ref())
        .into_iter()
        .flat_map(|i| i.filter_map(|e| e.ok()).filter(|p| !p.path().is_dir()))
}

struct MatcherOutput {
    primary: Vec<String>,
    secondary: Vec<String>,
}

impl MatcherOutput {
    fn output(mut self) -> Vec<String> {
        self.primary.sort_by(|a, b| a.len().cmp(&b.len()));
        self.secondary.sort_by(|a, b| a.len().cmp(&b.len()));

        self.primary
            .into_iter()
            .chain(self.secondary.into_iter())
            .take(25)
            .collect()
    }
}

fn format(file_name: &str, dir: &Path) -> String {
    format!("{} ({})", file_name, dir.to_string_lossy())
}

pub fn get_matching(search_term: &str) -> Vec<String> {
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
            let path = entry.path();

            if let Some(file_name) = path.file_stem().map(|s| s.to_string_lossy()) {
                if file_name == search_term {
                    res.primary.push(format(&file_name, &dir));
                } else if file_name.contains(search_term) {
                    res.secondary.push(format(&file_name, &dir));
                }
            }
        }
    }

    return res.output();
}
