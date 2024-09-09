use std::path::PathBuf;

use crate::resources;
use linicon::lookup_icon;
use tracing::warn;

pub fn get_icon(name: &str) -> Option<PathBuf> {
    let mut res = resources::ICON_MAP.get().write().unwrap();
    if let Some(e) = res.get(name) {
        return e.clone();
    }

    let icons: Vec<_> = lookup_icon(name)
        .with_scale(1)
        .with_size(48)
        .with_search_paths(&["~/.local/share/icons"])
        .unwrap()
        .filter_map(|x| match x {
            Ok(i) => Some(i.path),
            Err(e) => {
                warn!("{e:?}");
                None
            }
        })
        .collect();

    if icons.is_empty() {
        res.insert(name.to_string(), None);
        None
    } else {
        res.insert(name.to_string(), Some(icons[0].clone()));
        Some(icons[0].clone())
    }
}
