use crate::{theme::Theme, tui};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, widgets::*};
use std::{
    env::set_current_dir,
    io::{self, Result},
    path::PathBuf,
};

use self::utils::{get_cur_dir_entries_ordered, get_cur_dir_path, get_directory_display_entries};

#[derive(Debug, Default)]
pub struct FilePicker {
    items: Vec<PathBuf>,
    index: usize,
    exit: bool,
}

mod utils;

impl FilePicker {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<String> {
        match get_cur_dir_entries_ordered() {
            Ok(ret) => self.items = ret,
            Err(error) => println!("Error while reading directory: {:?}", error),
        }

        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }

        return get_cur_dir_path();
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
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
        match set_current_dir(&self.items[self.index]) {
            Ok(_) => (),
            Err(error) => println!("Error opening directory/file: {:?}", error),
        }
        self.items = get_cur_dir_entries_ordered().unwrap_or(<Vec<PathBuf>>::new());
        self.index = 0;
    }

    fn exit(&mut self) {

        self.exit = true;
    }
}

impl Widget for &FilePicker {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let list =
            List::new(get_directory_display_entries(self)).highlight_style(Theme::new().selected);

        let mut state = ListState::default().with_selected(Some(self.index));

        StatefulWidget::render(list, area, buf, &mut state);
    }
}
