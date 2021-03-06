use freedesktop_entry_parser::parse_entry;
use std::{
    cmp::Ordering,
    collections::HashSet,
    env, fmt,
    fs::{self},
    path::{Path, PathBuf},
    process::Command,
};

fn spawn_in_terminal(name: &str, path: Option<&str>) {
    let mut command = Command::new("x-terminal-emulator");
    command.arg("-e");
    command.arg(format!("{}; $SHELL", name));

    if let Some(path) = path {
        command.current_dir(path);
    }

    command.spawn().ok();
}

pub(crate) enum Item {
    Desktop {
        name: String,
        filename: String,
        terminal: bool,
        exec: String,
        args: Vec<String>,
        path: Option<String>,
        dir: PathBuf,
    },
    Path {
        name: String,
        dir: PathBuf,
    },
}

impl Item {
    fn new(filename: String, dir: PathBuf) -> Option<Self> {
        if filename.ends_with(".desktop") {
            let path = dir.join(&filename);

            let entry = parse_entry(path).ok()?;

            let section = entry.section("Desktop Entry");

            let name = section.attr("Name")?.to_string();

            let filename = filename.trim_end_matches(".desktop").to_owned();

            let terminal = section
                .attr("Terminal")
                .map(|t| t == "true")
                .unwrap_or(false);

            let mut args = section
                .attr("Exec")?
                .split(" ")
                .filter(|s| *s != "%F" && *s != "%f")
                .filter(|s| *s != "%U" && *s != "%u")
                .map(|s| s.to_owned())
                .collect::<Vec<_>>();

            let path = section.attr("Path").map(|s| s.to_owned());

            if args.len() == 0 {
                return None;
            }

            let exec = args.remove(0);

            Some(Self::Desktop {
                name,
                filename,
                terminal,
                exec,
                args,
                path,
                dir,
            })
        } else {
            Some(Self::Path {
                name: filename,
                dir,
            })
        }
    }

    pub(crate) fn execute(&self) {
        match self {
            Item::Desktop {
                terminal,
                exec,
                args,
                path,
                ..
            } => {
                if *terminal {
                    spawn_in_terminal(exec, path.as_deref());
                } else {
                    let mut command = Command::new(exec);
                    command.args(args);

                    if let Some(path) = path {
                        command.current_dir(path);
                    }

                    command.spawn().ok();
                }
            }
            Item::Path { name, .. } => spawn_in_terminal(name, None),
        }
    }

    pub(crate) fn name(&self) -> &str {
        match self {
            Item::Desktop { name, .. } => name,
            Item::Path { name, .. } => name,
        }
    }

    pub(crate) fn short_name(&self) -> &str {
        match self {
            Item::Desktop { filename, .. } => filename,
            Item::Path { name, .. } => name,
        }
    }

    pub(crate) fn is_desktop(&self) -> bool {
        match self {
            Item::Desktop { .. } => true,
            Item::Path { .. } => false,
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Item::Desktop { name, dir, .. } => write!(f, "{} ({})", name, dir.to_string_lossy()),
            Item::Path { name, dir } => write!(f, "{} ({})", name, dir.to_string_lossy()),
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

pub(crate) fn filter<'a>(items: &'a [Item], search_term: &str) -> Vec<&'a Item> {
    let search_term = search_term.to_lowercase();
    let mut items = items
        .iter()
        .filter(|i| {
            i.name().to_lowercase().contains(&search_term)
                || i.short_name().to_lowercase().contains(&search_term)
        })
        .collect::<Vec<_>>();

    items.sort_by(|a, b| {
        let name_a = a.name();
        let name_b = b.name();

        let short_a = a.short_name();
        let short_b = b.short_name();

        match (
            name_a == search_term || short_a == search_term,
            name_b == search_term || short_b == search_term,
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

pub(crate) fn get() -> Vec<Item> {
    let mut res = Vec::new();

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
                if let Some(item) = Item::new(file_name.into_owned(), dir.clone()) {
                    res.push(item);
                }
            }
        }
    }

    res
}
