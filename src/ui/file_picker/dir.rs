use chrono::{DateTime, Utc};
use itertools::Itertools;
use std::{
    env::{self, current_dir, set_current_dir},
    io::{Error, Result},
    os::unix::
        fs::{MetadataExt, PermissionsExt}
    ,
    path::PathBuf,
    time::SystemTime,
};
use users::{get_group_by_gid, get_user_by_uid, Group, User};

use crate::ui::popup::popup::show_error;

pub struct Dir {
    pub pathbuf: PathBuf,
    pub display_name: String,
}

impl Default for Dir {
    fn default() -> Self {
        let home = current_dir().unwrap_or_default();
        Dir {
            pathbuf: home.clone(),
            display_name: home.display().to_string(),
        }
    }
}

pub fn get_dir_entries_ordered(dir: PathBuf) -> Result<Vec<Dir>> {
    //gets sorted list of files/directories in current working dir
    let entries: Vec<PathBuf> = dir
        .read_dir()?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<PathBuf>>>()?
        .into_iter() // TODO implement own Ord
        .sorted()
        .collect();

    //vector of pointers to the items in entries that are folders
    let folder_entries: Vec<PathBuf> = entries
        .iter()
        .filter(|r| r.is_dir())
        .map(|r| (*r).clone())
        .collect();

    //vector of pointers to the items in entries that are files (not folder)
    let file_entries: Vec<PathBuf> = entries
        .iter()
        .filter(|r| !r.is_dir())
        .map(|r| (*r).clone())
        .collect();

    let ordered_entries: Vec<PathBuf> = vec![folder_entries, file_entries].concat();

    let mut res = <Vec<Dir>>::new();

    let parent = Dir {
        pathbuf: current_dir()?.parent().unwrap_or(&dir).to_path_buf(),
        display_name: " ..".to_string(),
    };

    res.push(parent);

    for entry in ordered_entries {
        let d = Dir {
            pathbuf: entry.clone(),
            display_name: get_display_name(entry),
        };
        res.push(d);
    }

    return Ok(res);
}

pub fn get_display_name(entry: PathBuf) -> String {
    let display_name = match entry.is_dir() {
        true => "  ".to_string(),
        false => "  ".to_string(),
    } + entry
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    display_name
}

pub fn get_cur_dir() -> Dir {
    match current_dir() {
        Ok(cd) => Dir {
            pathbuf: cd.to_path_buf().clone(),
            display_name: get_display_name(cd.to_path_buf()),
        },
        Err(e) => {
            show_error("Couldn't get current directory", e);
            return Dir {
                pathbuf: PathBuf::default(),
                display_name: get_display_name(PathBuf::default()),
            };
        }
    }
}

pub fn get_parent_dir(path: PathBuf) -> Dir {
    match path.parent() {
        Some(p) => Dir {
            pathbuf: p.to_path_buf(),
            display_name: get_display_name(p.to_path_buf()),
        },
        None => {
            show_error("Couldn't get parent directory", Error::last_os_error());
            Dir {
                pathbuf: path.clone(),
                display_name: get_display_name(path),
            }
        }
    }
}

pub fn change_working_dir(path: PathBuf) {
    match set_current_dir(path) {
        Ok(_) => (),
        Err(error) => show_error("Error opening directory/file", error),
    }
}

pub fn get_shortened_path(path: PathBuf) -> String {
    let home = env::var("HOME").unwrap_or_default();

    return path.display().to_string().replace(&home.clone(), "~");
}

pub fn get_entry_name(path: PathBuf) -> String {
    return path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
}

pub fn get_entry_permissions_to_display(path: PathBuf) -> String {
    match path.metadata() {
        Ok(m) => {
            let mut result = "".to_string();

            result = result
                + if path.is_symlink() {
                    "l"
                } else if path.is_dir() {
                    "d"
                } else if path.is_file() {
                    "-"
                } else {
                    "?"
                };
            let st_mode = format!("{:o}", m.permissions().mode());

            let mut permissions = st_mode.clone();
            let _ = permissions.drain(0..permissions.len() - 3);

            permissions = permissions
                .replace("0", "---")
                .replace("1", "--x")
                .replace("2", "-w-")
                .replace("3", "-wx")
                .replace("4", "r--")
                .replace("5", "r-x")
                .replace("6", "rw-")
                .replace("7", "rwx");

            result.push_str(&permissions);
            result.push_str(" ");

            let mut n_links = m.nlink().to_string();
            if n_links.len() == 1 {
                n_links = " ".to_string() + &n_links;
            }
            result.push_str(&(&n_links));

            let owner_user = get_user_by_uid(m.uid());
            let owner_group = get_group_by_gid(m.gid());

            result.push_str(" ");
            result.push_str(
                &owner_user
                    .unwrap_or(User::new(000, "invalid", 0))
                    .name()
                    .to_string_lossy(),
            );

            result.push_str(" ");
            result.push_str(
                &owner_group
                    .unwrap_or(Group::new(000, "invalid"))
                    .name()
                    .to_string_lossy(),
            );

            result.push_str(" ");

            let date_modified: DateTime<Utc> = m
                .modified()
                .or::<SystemTime>(Ok(SystemTime::UNIX_EPOCH))
                .unwrap()
                .into();

            let date_string = date_modified.format("%d %b %Y %H:%S").to_string();

            result.push_str(&date_string);

            result
        }
        Err(e) => {
            show_error("Error reading file permissions", e);
            return "error reading file permissions".to_string();
        }
    }
}
