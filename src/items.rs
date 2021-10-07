use std::{
    cmp::Ordering,
    collections::HashSet,
    env, fmt,
    fs::{self},
    path::{Path, PathBuf},
    process::Command,
};

use freedesktop_entry_parser::parse_entry;

pub(crate) struct Item {
    filename: String,
    dir: PathBuf,
}

impl Item {
    fn new(filename: String, dir: PathBuf) -> Self {
        Self { filename, dir }
    }

    pub(crate) fn execute(&self) {
        if self.is_desktop() {
            let path = self.dir.join(&self.filename);

            println!("{}", fs::read_to_string(&path).unwrap());

            if let Ok(desktop) = parse_entry(path) {
                let section = desktop.section("Desktop Entry");
                let terminal = section
                    .attr("Terminal")
                    .map(|t| t == "true")
                    .unwrap_or(false);

                if let Some(exec) = section.attr("Exec") {
                    if terminal {
                        Command::new("x-terminal-emulator")
                            .arg("-e")
                            .arg(format!("{}; $SHELL", exec))
                            .spawn()
                            .ok();
                    } else {
                        let exec = exec
                            .split(" ")
                            .filter(|s| *s != "%F" && *s != "%f")
                            .collect::<Vec<_>>();

                        if exec.len() == 1 {
                            Command::new(exec[0]).spawn().ok();
                        } else {
                            Command::new(exec[0]).args(&exec[1..]).spawn().ok();
                        }
                    }
                }
            }
        } else {
            Command::new("x-terminal-emulator")
                .arg("-e")
                .arg(format!("{}; $SHELL", self.filename))
                .spawn()
                .ok();
        }
    }

    pub(crate) fn is_desktop(&self) -> bool {
        self.filename.ends_with(".desktop")
    }

    pub(crate) fn name(&self) -> &str {
        self.filename.trim_end_matches(".desktop")
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_desktop() {
            let path = self.dir.join(&self.filename);
            let entry = parse_entry(path);

            if let Some(name) = entry.ok().and_then(|e| {
                e.section("Desktop Entry")
                    .attr("Name")
                    .map(|s| s.to_owned())
            }) {
                write!(f, "{} ({})", name, self.dir.to_string_lossy())
            } else {
                write!(f, "{} ({})", self.filename, self.dir.to_string_lossy())
            }
        } else {
            write!(f, "{} ({})", self.filename, self.dir.to_string_lossy())
        }
    }
}

fn walk_dir(dir: impl AsRef<Path>) -> impl Iterator<Item = PathBuf> {
    fs::read_dir(dir.as_ref()).into_iter().flat_map(|i| {
        i.filter_map(|e| e.ok())
            .filter(|p| !p.path().is_dir())
            .map(|p| p.path())
    })
}

struct MatcherOutput<'a> {
    items: Vec<Item>,
    search_term: &'a str,
}

impl<'a> MatcherOutput<'a> {
    fn output(self) -> Vec<Item> {
        let Self {
            mut items,
            search_term,
        } = self;
        items.sort_by(|a, b| {
            let name_a = a.name();
            let name_b = b.name();

            match (
                name_a == search_term,
                name_b == search_term,
                a.is_desktop(),
                b.is_desktop(),
                name_a,
                name_b,
            ) {
                (true, false, ..) => Ordering::Less,
                (false, true, ..) => Ordering::Greater,
                (_, _, true, false, ..) => Ordering::Less,
                (_, _, false, true, ..) => Ordering::Greater,
                (_, _, _, _, a, b) => a.len().cmp(&b.len()),
            }
        });

        items.into_iter().take(25).collect()
    }
}

pub(crate) fn get_matching(search_term: &str) -> Vec<Item> {
    let mut res = MatcherOutput {
        items: Vec::new(),
        search_term,
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
                if file_name.trim_end_matches(".desktop").contains(search_term) {
                    res.items
                        .push(Item::new(file_name.into_owned(), dir.clone()));
                }
            }
        }
    }

    return res.output();
}
