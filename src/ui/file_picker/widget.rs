use crate::{
    theme::Theme,
    ui::popup::popup::{show_confirmation, show_error, show_info},
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, widgets::*};
use std::{
    fs::{remove_dir_all, remove_file},
    io::Result,
    path::PathBuf,
};

use super::dir::{change_working_dir, get_cur_dir, get_dir_entries_ordered, get_parent_dir, Dir};

pub struct FilePicker {
    items: Vec<Dir>,
    index: usize,
    buffer: Vec<PathBuf>,
    pub selected: bool,
    pub needs_redraw: bool,
}

impl FilePicker {
    pub fn new(is_selected: bool) -> FilePicker {
        FilePicker {
            items: <Vec<Dir>>::new(),
            index: 0,
            buffer: <Vec<PathBuf>>::new(),
            selected: is_selected,
            needs_redraw: false,
        }
    }
    /// runs the application's main loop until the user quits
    pub fn initialize(&mut self, dir: Option<PathBuf>, index: Option<usize>) {
        // poor man try catch
        if let Err(error) = (|| -> Result<()> {
            let mut items = get_dir_entries_ordered(dir.unwrap_or(get_cur_dir().pathbuf))?;

            // displays which entries are in the buffer to the user
            for item in items.iter_mut() {
                if self.buffer.contains(&item.pathbuf) {
                    item.display_name.replace_range(
                        0..1,
                        if self.buffer.contains(&item.pathbuf) {
                            "-"
                        } else {
                            " "
                        },
                    );
                }
            }

            self.items = items;

            // chooses where the cursor should be
            // if an index was passed as argument it puts the cursor on that index if it exists, if
            // not, or if no index passed, does this:
            // if the directory has entries the cursor is put on the first entry
            // if the directory is empty the cursor is put on the parent folder entry
            match index {
                Some(value) => {
                    self.index = if self.items.len() as i32 > value as i32 - 1 {
                        value
                    } else {
                        if self.items.len() as i32 > 1 {
                            1
                        } else {
                            0
                        }
                    }
                }
                None => self.index = if self.items.len() as i32 > 1 { 1 } else { 0 },
            }

            Ok(())
        })() {
            show_error("Error while reading directory", error);
        }
    }

    pub fn handle_keys(&mut self, key: KeyEvent) {
        // it's important to check that the event is a key press event as
        // crossterm also emits key release and repeat events on Windows.
        match key.code {
            KeyCode::Char('h') => self.up_dir(),
            KeyCode::Char('j') => self.select_next(),
            KeyCode::Char('k') => self.select_prev(),
            KeyCode::Char('l') => self.open_selected_dir(),
            KeyCode::Char(' ') => self.buffer_item(),
            KeyCode::Char('d') => self.delete_sel_entry(),
            KeyCode::Char('g') => self.select_first(),
            KeyCode::Char('G') => self.select_last(),
            KeyCode::Char('b') => {
                // this second loop is to handle sequence keybindings
                // after the first key in a sequence keybinding is detected it waits for the next
                // key event to see if it is one of the keybindings
                if let Ok(event) = event::read() {
                    // it's important to check that the event is a key press event as
                    // crossterm also emits key release and repeat events on Windows.
                    match event {
                        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                            match key_event.code {
                                // [bc] clear buffer, ie: unmarks all files
                                KeyCode::Char('c') => {
                                    self.buffer.clear();
                                    let curr_displaying_dir = self.get_curr_displaying_dir();
                                    self.initialize(Some(curr_displaying_dir), Some(self.index))
                                }
                                // [bd] deletes all files in buffer
                                KeyCode::Char('d') => {
                                    if self.delete_files(self.buffer.clone()) {
                                        self.buffer.clear();
                                    }
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    }
                }
            }
            KeyCode::Char('?') => self.show_help(),
            _ => {}
        }
    }

    fn show_help(&mut self) {
        self.needs_redraw = true;
        let _ = show_info(
            "Keybindings",
            "".to_string()
                + "[?]         - Show this window\n"
                + "[j/k]       - Navigate up/down in list\n"
                + "[l]         - Open directory/file\n"
                + "[h]         - Go to parent directory\n"
                + "[Space]     - Adds/Removes directory/files to/from buffer\n"
                + "[d]         - Delete directory/file\n"
                + "[bc]        - Clears buffer, ie: unselects all\n"
                + "[bd]        - Deletes all files in buffer\n"
                + "[Ctrl+ h/l] - Switch selected panel\n",
        );
    }

    fn select_first(&mut self) {
        self.index = if self.items.len() > 1 { 1 } else { 0 }
    }

    fn select_last(&mut self) {
        self.index = self.items.len() - 1;
    }

    fn select_next(&mut self) {
        if self.index < self.items.len() - 1 {
            self.index += 1;
        }
    }

    fn select_prev(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }

    pub fn curr_sel_entry(&mut self) -> PathBuf {
        self.items[self.index].pathbuf.clone()
    }

    fn open_selected_dir(&mut self) {
        let current_selected = self.curr_sel_entry();
        if current_selected.is_dir() {
            self.change_curr_dir(current_selected);
        }
    }

    // adds/removes the item under the cursor to the buffer list for later use
    fn buffer_item(&mut self) {
        let value = self.curr_sel_entry();
        if self.buffer.contains(&value) {
            self.buffer.retain(|b| *b != value) // removes if already in buffer
        } else {
            self.buffer.push(value.clone()) // adds it if not
        }
        let curr_displaying_dir = self.get_curr_displaying_dir();
        self.initialize(Some(curr_displaying_dir), Some(self.index));
    }

    fn delete_sel_entry(&mut self) {
        let curr_sel = self.curr_sel_entry();
        self.delete_files(vec![curr_sel]);
    }

    fn delete_files(&mut self, files: Vec<PathBuf>) -> bool {
        let mut files_string: String = "".to_string();

        // builds the list of files that will be deleted to display to the user
        for path in files.clone() {
            files_string = files_string
                + &path
                    .as_path()
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .to_string()
                + "\n";
        }

        self.needs_redraw = true;

        //poor man try catch
        match (|| -> Result<bool> {
            if show_confirmation("Confirm deletion?", files_string)? {
                // deletes all file(s) sent as argument
                for entry in files {
                    if entry.clone().is_dir() {
                        remove_dir_all(entry)?;
                    } else if entry.is_file() {
                        remove_file(entry)?;
                    }
                }
                // reinitializes the directory list
                let curr_displaying_dir = self.get_curr_displaying_dir();
                self.initialize(
                    Some(curr_displaying_dir),
                    if self.index > 2 {
                        Some(self.index - 1)
                    } else {
                        None
                    },
                );

                Ok(true)
            } else {
                Ok(false)
            }
        })() {
            Err(error) => {
                show_error("Error deleting file", error);
                false
            }
            Ok(res) => res,
        }
    }
    
    fn up_dir(&mut self) {
        self.change_curr_dir(get_parent_dir(get_cur_dir().pathbuf).pathbuf);
    }

    // returns current directory being displayed in the list
    // *not* the apllication working directory
    fn get_curr_displaying_dir(&mut self) -> PathBuf {
        get_parent_dir(self.curr_sel_entry()).pathbuf
    }

    fn change_curr_dir(&mut self, path: PathBuf) {
        change_working_dir(path.clone());
        self.initialize(Some(path.clone()), None);
    }

}

impl Widget for &FilePicker {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut style = Theme::new();

        // dims widget if not selected
        if !self.selected {
            style.normal = style.normal.add_modifier(Modifier::DIM);
            style.selected = style.selected.add_modifier(Modifier::DIM);
        }

        let list = List::new(
            self.items
                .iter()
                .map(|dir| dir.display_name.clone())
                .collect::<Vec<String>>(),
        )
        .style(style.normal)
        .highlight_style(style.selected);

        // necessary for catching events
        let mut state = ListState::default().with_selected(Some(self.index));

        StatefulWidget::render(list, area, buf, &mut state);
    }
}
