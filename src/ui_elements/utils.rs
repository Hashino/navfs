use itertools::Itertools;
use std::{env::current_dir, io::Result, path::PathBuf};

pub struct Dir {
    pub pathbuf: PathBuf,
    pub display_name: String,
}

pub fn get_cur_dir_entries_ordered() -> Result<Vec<Dir>> {
    //gets sorted list of files/directories in current working dir
    let entries: Vec<PathBuf> = current_dir()?
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

    //vector of pointers tothe items in entries that are files (not folder)
    let file_entries: Vec<PathBuf> = entries
        .iter()
        .filter(|r| !r.is_dir())
        .map(|r| (*r).clone())
        .collect();

    let ordered_entries: Vec<PathBuf> = vec![folder_entries, file_entries].concat();

    let mut res = <Vec<Dir>>::new();

    let parent = Dir {
        pathbuf: current_dir()?
            .parent()
            .unwrap_or(&current_dir()?)
            .to_path_buf(),
        display_name: " ..".to_string(),
    };

    res.push(parent);

    for entry in ordered_entries {
        let d = Dir {
            pathbuf: entry.clone(),
            display_name: match entry.is_dir() {
                true => "  ".to_string(),
                false => "  ".to_string(),
            } + entry
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default(),
        };
        res.push(d);
    }

    return Ok(res);
}

pub fn get_cur_dir_path() -> Result<String> {
    let path_res = current_dir();
    match path_res {
        Ok(path) => Ok(path.as_path().to_str().unwrap_or_default().to_string()),
        Err(e) => Err(e),
    }
}
