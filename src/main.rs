use core::panic;
use itertools::Itertools;
use std::{
    env::{current_dir, set_current_dir},
    io::{self, BufRead},
    path::{Path, PathBuf},
};

fn get_cur_dir_entries() -> std::io::Result<Vec<PathBuf>> {
    //gets sorted list of files/directories in current working dir
    let entries: Vec<PathBuf> = current_dir()?
        .read_dir()?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<PathBuf>, io::Error>>()?
        .into_iter()
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

    return Ok(ordered_entries);
}

fn show_entries(entries: Vec<PathBuf>) {
    println!("..");
    for entry in entries {
        println!(
            "{0} {1}",
            if entry.is_dir() { '' } else { '' },
            entry
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
        );
    }

    println!();
    println!("Type the name of the folder/file to open (or quit to exit):");
}

fn main() -> std::io::Result<()> {
    loop {
        let curr_dir_entries: Vec<PathBuf> = get_cur_dir_entries()?;

        show_entries(curr_dir_entries.clone());

        let input = match io::stdin().lock().lines().next() {
            Some(x) => x?,
            None => "IO Error".to_string(),
        };

        match input.as_str() {
            "quit" => break (),
            ".." => match set_current_dir(
                current_dir()?
                    .parent()
                    .unwrap_or(&(Path::new(&(current_dir()?)))),
            ) {
                Ok(..) => continue,
                Err(error) => panic!("Error while opening the directory/folder: {:?}", error),
            },
            _ => match set_current_dir(
                curr_dir_entries
                    .into_iter()
                    .filter(|x| x.file_name().unwrap_or_default().to_string_lossy() == input)
                    .collect::<PathBuf>(),
            ) {
                Ok(..) => continue,
                Err(error) => panic!("Error while opening the directory/folder: {:?}", error),
            },
        }
    }
    Ok(())
}
