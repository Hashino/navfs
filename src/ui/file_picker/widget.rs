use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use std::{env::set_current_dir, io::Result, path::PathBuf};

use super::dir::{get_cur_dir, get_dir_entries_ordered, Dir};

#[derive(Default)]
pub struct FilePicker {
    items: Vec<Dir>,
    index: usize,
    pub selected: bool,
}

impl FilePicker {
    /// runs the application's main loop until the user quits
    pub fn initialize(&mut self, dir: Option<PathBuf>) {
        // poor man try catch
        if let Err(error) = (|| -> Result<()> {
            self.items = get_dir_entries_ordered(dir.unwrap_or(get_cur_dir()?.pathbuf))?;

            self.index = if self.items.len() > 0 { 1 } else { 0 };

            Ok(())
        })() {
            println!("Error while reading directory: {:?}", error); // TODO: replac with a popup
        }
    }

    pub fn handle_keys(&mut self, key: KeyEvent) {
        // it's important to check that the event is a key press event as
        // crossterm also emits key release and repeat events on Windows.
        match key.code {
            KeyCode::Char('j') => self.select_next(),
            KeyCode::Char('k') => self.select_prev(),
            KeyCode::Enter => self.open_dir(),
            _ => {}
        }
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

    fn open_dir(&mut self) {
        match set_current_dir(&self.items[self.index].pathbuf) {
            Ok(_) => (),
            Err(error) => println!("Error opening directory/file: {:?}", error), // TODO: replace
                                                                                 // with popup
        }

        self.initialize(None);
    }

    pub fn curr_sel_entry(&mut self) -> &PathBuf {
        &self.items[self.index].pathbuf
    }

    pub fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
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
