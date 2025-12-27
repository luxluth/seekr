use freedesktop_desktop_entry::{DesktopEntry, get_languages_from_env};
use std::{path::PathBuf, process::Command};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub exec: String,
    pub need_terminal: bool,
    pub icon: String,
    pub name: String,
    pub description: String,
    pub _path: PathBuf,
}

const PATTERNS: [&str; 13] = [
    "%f", "%F", "%u", "%U", "%d", "%D", "%n", "%N", "%i", "%c", "%k", "%v", "%m",
];

fn get_exec(exec: &str) -> String {
    let mut res = exec.to_string();
    PATTERNS.iter().for_each(|f| {
        res = res.replace(*f, "");
    });
    res.trim().to_string()
}

impl AppEntry {
    pub fn launch(&self, term: String, term_launch_args: Vec<String>) {
        let exec = get_exec(&self.exec);
        debug!("{exec:?}");
        if self.need_terminal {
            let mut args = term_launch_args;
            args.push(exec);
            let _ = Command::new("sh")
                .arg("-c")
                .arg(format!("{term} {}", args.join(" ")))
                .spawn();
        } else {
            let _ = Command::new("sh").arg("-c").arg(exec).spawn();
        }
    }
}

pub fn collect_apps() -> Vec<AppEntry> {
    let locales = get_languages_from_env();
    freedesktop_desktop_entry::Iter::new(freedesktop_desktop_entry::default_paths())
        .into_iter()
        .filter_map(|p| {
            if let Ok(entry) = DesktopEntry::from_path(p.clone(), Some(&locales)) {
                if entry.no_display() {
                    return None;
                } else {
                    return Some(AppEntry {
                        exec: entry.exec().unwrap_or_default().to_string(),
                        need_terminal: entry.terminal(),
                        icon: entry
                            .icon()
                            .unwrap_or("application-x-executable")
                            .to_string(),
                        name: entry.name(&locales).unwrap_or_default().to_string(),
                        description: entry.comment(&locales).unwrap_or_default().to_string(),
                        _path: p,
                    });
                }
            }

            None
        })
        .collect()
}
