use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use std::{env::set_current_dir, io::Result};

use super::dir::{get_cur_dir_entries_ordered, Dir};

#[derive(Default)]
pub struct FilePicker {
    items: Vec<Dir>,
    index: usize,
    pub selected: bool,
}

impl FilePicker {
    /// runs the application's main loop until the user quits
    pub fn initialize(&mut self) {
        match get_cur_dir_entries_ordered() {
            Ok(ret) => self.items = ret,
            Err(error) => println!("Error while reading directory: {:?}", error),
        }
    }

    pub fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    pub fn handle_keys(&mut self, key: KeyEvent) -> Result<()> {
        // it's important to check that the event is a key press event as
        // crossterm also emits key release and repeat events on Windows.
        match key.code {
            KeyCode::Char('j') => self.select_next(),
            KeyCode::Char('k') => self.select_prev(),
            KeyCode::Enter => self.open_dir(),
            _ => {}
        }
        Ok(())
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
            Err(error) => println!("Error opening directory/file: {:?}", error),
        }
        self.initialize();
        self.index = 0;
    }
}

impl Widget for &FilePicker {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let list = List::new(
            self.items
                .iter()
                .map(|dir| dir.display_name.clone())
                .collect::<Vec<String>>(),
        )
        .highlight_style(Theme::new().selected);

        let mut state = ListState::default().with_selected(Some(self.index));

        StatefulWidget::render(list, area, buf, &mut state);
    }
}
