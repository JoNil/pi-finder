use std::{
    env,
    fs::{self, DirEntry},
    ops::ControlFlow,
    path::Path,
};

fn walk_dir(
    dir: impl AsRef<Path>,
    callback: &mut dyn FnMut(&DirEntry) -> ControlFlow<()>,
) -> ControlFlow<()> {
    if let Ok(read_dir) = fs::read_dir(dir.as_ref()) {
        for entry in read_dir {
            if let Ok(entry) = entry {
                let path = entry.path();
                match if path.is_dir() {
                    walk_dir(&path, callback)
                } else {
                    callback(&entry)
                } {
                    ControlFlow::Break(_) => {
                        return ControlFlow::Break(());
                    }
                    ControlFlow::Continue(_) => {
                        continue;
                    }
                }
            }
        }
    }

    ControlFlow::Continue(())
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
        for entry in path.split(":") {
            dirs_to_search.push(Path::new(entry).to_owned());
        }
    }

    for dir in dirs_to_search {
        if let ControlFlow::Break(()) = walk_dir(dir, &mut |entry| {
            let path = entry.path();
            let path_string = path.to_string_lossy();

            if path_string.contains(search_term) {
                if let Some(file_name) = path.file_name() {
                    res.push(file_name.to_string_lossy().to_string());

                    if res.len() > 25 {
                        return ControlFlow::Break(());
                    }
                }
            }

            ControlFlow::Continue(())
        }) {
            break;
        }
    }

    res
}
