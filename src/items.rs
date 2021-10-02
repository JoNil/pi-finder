use std::{
    collections::HashSet,
    env,
    fs::{self, DirEntry},
    path::Path,
};

fn walk_dir(dir: impl AsRef<Path>) -> impl Iterator<Item = DirEntry> {
    fs::read_dir(dir.as_ref())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|p| !p.path().is_dir())
}

pub fn get_matching(search_term: &str) -> Vec<String> {
    let mut res = Vec::new();

    let mut dirs_to_search = Vec::new();
    dirs_to_search.push(Path::new("/usr/share/applications").to_owned());

    #[allow(deprecated)]
    if let Some(home) = env::home_dir() {
        dirs_to_search.push(home.join(".local/share/applications"));
    }

    if let Ok(path) = env::var("PATH") {
        let mut unique_dirs = HashSet::new();
        for entry in dbg!(path).split(":") {
            if !unique_dirs.contains(&entry) {
                unique_dirs.insert(entry);
                dirs_to_search.push(Path::new(entry).to_owned());
            }
        }
    }

    for dir in dbg!(dirs_to_search) {
        for entry in walk_dir(&dir) {
            let path = entry.path();
            let path_string = path.to_string_lossy();

            if path_string.contains(search_term) {
                if let Some(file_name) = path.file_name() {
                    res.push(format!(
                        "{} ({})",
                        file_name.to_string_lossy(),
                        dir.to_string_lossy()
                    ));

                    if res.len() > 25 {
                        return res;
                    }
                }
            }
        }
    }

    res
}
