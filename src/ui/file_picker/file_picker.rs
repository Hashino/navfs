use crate::{
    theme::Theme,
    ui::popup::{
        self,
        popup::{show_confirmation, show_error, show_info},
    },
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, widgets::*};
use std::{
    fs::{remove_dir_all, remove_file},
    io::Result,
    path::PathBuf,
};

use super::dir::Dir;

/// Wrapper widget around the [List](ratatui::widgets::List) to manage its events and state
///
/// [items](Vec<Dir>): the current entries rendered in the widget
/// [index](usize): the index of the entry under the cursor
/// [buffer](Vec<PathBuf): buffered (selected) items currently
/// [active](bool): if the widget is currently selected
/// [needs_redraw](bool): tells the parent widget it needs to redraw itself
pub struct FilePicker {
    items: Vec<Dir>,
    index: usize,
    buffer: Vec<PathBuf>,
    pub active: bool,
    pub needs_redraw: bool,
}

impl FilePicker {
    pub fn new(is_selected: bool) -> FilePicker {
        FilePicker {
            items: <Vec<Dir>>::new(),
            index: 0,
            buffer: <Vec<PathBuf>>::new(),
            active: is_selected,
            needs_redraw: false,
        }
    }

    /// renders the widget with [dir](Option<PathBuf>) as the working directory
    /// and the entry at the [index](Option<usize>) selected
    ///
    /// on [dir](Option<PathBuf>) == `None` defaults current working directory
    /// on [index](Option<usize>) == `None` defaults to the first entry on the list (if the directory
    /// is not empty, the first after the parent entry)
    pub fn initialize(&mut self, dir: Option<PathBuf>, index: Option<usize>) {
        // poor man try catch
        if let Err(error) = (|| -> Result<()> {
            let mut items =
                Dir::get_dir_entries_ordered(dir.unwrap_or(Dir::get_cur_dir().pathbuf))?;

            // displays which entries are in the buffer to the user
            for item in items.iter_mut() {
                if self.buffer.contains(&item.pathbuf) {
                    item.display_name.replace_range(
                        0..1,
                        // TODO: find a better symbol to indicate buffered entries
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
                    // important to not index a negative value
                    self.index = if self.items.len() as i32 > value as i32 - 1 {
                        value
                    } else {
                        // checks if the directory is not empty
                        // (in which case the only entry will be the parent)
                        if self.items.len() as i32 > 1 {
                            1
                        } else {
                            0
                        }
                    }
                }
                // in case no index is passed it selects either the parent, if the directory is
                // empty, or the first entry after parent, if not.
                None => self.index = if self.items.len() as i32 > 1 { 1 } else { 0 },
            }

            Ok(())
        })() {
            self.items = vec![Dir {
                pathbuf: Dir::get_cur_dir().pathbuf,
                display_name: "Couldn't read entry: ".to_string() + &error.to_string(),
            }];
        }
    }

    pub fn handle_keys(&mut self, key: KeyEvent) {
        // it's important to check that the event is a key press event as
        // crossterm also emits key release and repeat events on Windows.
        match key.code {
            KeyCode::Char('h') => self.up_dir(), // go to parent directory
            KeyCode::Char('j') => self.select_next(), // moves the cursor down in the list
            KeyCode::Char('k') => self.select_prev(), // moves the cursor up in the list
            KeyCode::Char('l') => self.open_selected_dir(), // opens the entry under cursor
            KeyCode::Char(' ') => self.buffer_item(), // adds/removes item under cursor from buffer
            KeyCode::Char('d') => self.delete_sel_entry(), // deletes entry under cursor
            KeyCode::Char('g') => self.select_first(), // selects first entry after parent entry
            KeyCode::Char('G') => self.select_last(), // selects last entry on the list
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
            KeyCode::Char('?') => self.show_help(), // shows keybindings popup
            _ => {}
        }
    }

    fn show_help(&mut self) {
        self.needs_redraw = true;
        let _ = show_info("Keybindings", popup::popup::KEYBINDINGS_INFO.to_string());
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

    /// opens the entry under the cursor on the current file picker
    fn open_selected_dir(&mut self) {
        let current_selected = self.curr_sel_entry();
        if current_selected.is_dir() {
            self.change_curr_dir(current_selected);
        }
    }

    /// adds/removes the item under the cursor to the buffer list for later use
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

    /// deletes with confirmation all directory entries passed as argument
    fn delete_files(&mut self, files: Vec<PathBuf>) -> bool {
        let mut files_string: String = "".to_string();

        // builds the list of files that will be deleted to display to the user
        for path in files.clone() {
            files_string = files_string
                + &Dir::get_entry_name(Dir::get_parent_dir(path.clone()).pathbuf)
                + "/"
                + &Dir::get_entry_name(path.clone())
                + if path.clone().is_dir() { "/" } else { "" }
                + "\n";
        }

        self.needs_redraw = true;

        //poor man try catch
        match (|| -> Result<bool> {
            if show_confirmation("Confirm deletion?", files_string) {
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
                        // puts the cursor on the item above the one deleted
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
                self.needs_redraw = true;
                false
            }
            Ok(res) => res,
        }
    }

    fn up_dir(&mut self) {
        let curr = Dir::get_cur_dir().pathbuf;
        let parent = Dir::get_parent_dir(curr.clone()).pathbuf;
        if parent != curr {
            self.change_curr_dir(Dir::get_parent_dir(Dir::get_cur_dir().pathbuf).pathbuf);
        }
    }

    /// returns current directory being displayed in the list
    /// *not* the apllication working directory
    fn get_curr_displaying_dir(&mut self) -> PathBuf {
        Dir::get_parent_dir(self.curr_sel_entry()).pathbuf
    }

    fn change_curr_dir(&mut self, path: PathBuf) {
        Dir::change_working_dir(path.clone());
        self.initialize(Some(path.clone()), None);
    }
}

/// handles the rendering of the widget
impl Widget for &FilePicker {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut style = Theme::default();

        // dims widget if not selected
        if !self.active {
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
