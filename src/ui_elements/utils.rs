use itertools::Itertools;
use std::{env::current_dir, io::Result, path::PathBuf};

use super::FilePicker;

pub fn get_directory_display_entries(picker: &FilePicker) -> Vec<String> {

        let mut display_entries = <Vec<String>>::new();

        for entry in picker.items.clone().into_iter() {
            let prefix: String = match entry.is_dir() {
                true => "  ".to_string(),
                false => "  ".to_string(),
            };
            let file_name: String = prefix
                + entry
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();
            display_entries.push(file_name);
        }
        
        display_entries[0] = " ..".to_string(); 

        return display_entries;
}

pub fn get_cur_dir_entries_ordered() -> Result<Vec<PathBuf>> {
    
    let parent: PathBuf = current_dir()?.parent().unwrap_or(&current_dir()?).to_path_buf()
;
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

    let ordered_entries: Vec<PathBuf> = vec![vec![parent], folder_entries, file_entries].concat();

    return Ok(ordered_entries);
}

pub fn get_cur_dir_path() -> Result<String> {
    let path_res = current_dir();
    match path_res {
        Ok(path) => Ok(path.as_path().to_str().unwrap_or_default().to_string()),
        Err(e) => Err(e),
    }
}
