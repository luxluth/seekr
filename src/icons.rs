use std::path::PathBuf;

use linicon::lookup_icon;
use tracing::warn;

pub fn get_icon(name: &str) -> Option<PathBuf> {
    let icons: Vec<_> = lookup_icon(name)
        .with_scale(1)
        .with_size(48)
        .filter_map(|x| match x {
            Ok(i) => Some(i.path),
            Err(e) => {
                warn!("{e:?}");
                None
            }
        })
        .collect();

    if icons.is_empty() {
        // let second_search: Vec<_> = lookup_icon(name)
        //     .with_scale(1)
        //     .with_size(64)
        //     .filter_map(|x| match x {
        //         Ok(i) => Some(i.path),
        //         Err(e) => {
        //             warn!("{e:?}");
        //             None
        //         }
        //     })
        //     .collect();
        //
        // if second_search.is_empty() {
        //     let third_search: Vec<_> = lookup_icon(name)
        //         .filter_map(|x| match x {
        //             Ok(i) => {
        //                 if i.icon_type == IconType::SVG {
        //                     Some(i.path)
        //                 } else {
        //                     None
        //                 }
        //             }
        //             Err(e) => {
        //                 warn!("{e:?}");
        //                 None
        //             }
        //         })
        //         .collect();
        //     if !third_search.is_empty() {
        //         Some(third_search[0].clone())
        //     } else {
        //         None
        //     }
        // } else {
        //     Some(second_search[0].clone())
        // }
        None
    } else {
        Some(icons[0].clone())
    }
}
