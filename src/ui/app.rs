use std::io::Result;
use std::path::PathBuf;

use crate::tui;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::{
    buffer::Buffer,
    layout::{Direction, Layout, Rect},
    widgets::Widget,
    Frame,
};

use super::file_picker::{dir::get_cur_dir, widget::FilePicker};

enum WhichPane {
    FilePicker,
    PreviewPane,
}

impl Default for WhichPane {
    fn default() -> Self {
        WhichPane::FilePicker
    }
}

#[derive(Default)]
pub struct App {
    file_picker: FilePicker,
    preview_pane: FilePicker,
    curr_selected: WhichPane,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<String> {
        self.file_picker.initialize(None);
        self.file_picker.selected = true;

        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            // terminal.draw(|frame| self.file_picker.render_frame(frame))?;
            self.handle_events()?;
        }

        match get_cur_dir() {
            Ok(path) => Ok(path.display_name),
            Err(e) => Err(e),
        }
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }
    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Char('q') => self.exit(),
                    _ => {
                        if key_event.modifiers == KeyModifiers::CONTROL {
                            match key_event.code {
                                KeyCode::Char('l') => {
                                    self.curr_selected = WhichPane::PreviewPane;
                                    self.file_picker.selected = false;
                                    self.preview_pane.selected = true;
                                }
                                KeyCode::Char('h') => { 
                                    self.curr_selected = WhichPane::FilePicker;
                                    self.file_picker.selected = true;
                                    self.preview_pane.selected = false;
                                }
                                _ => (),
                            }
                        }
                        match self.curr_selected {
                            WhichPane::FilePicker => {
                                self.file_picker.handle_keys(key_event);
                                match key_event.code {
                                    KeyCode::Char('j') | KeyCode::Char('k') => {
                                        let curr = self.file_picker.curr_sel_entry();
                                        if curr.is_dir() {
                                            self.preview_pane.initialize(Some(curr.to_path_buf()));
                                        }
                                    }
                                    KeyCode::Enter => {
                                        self.preview_pane.initialize(None);
                                    }
                                    _ => (),
                                }
                            }
                            WhichPane::PreviewPane => self.preview_pane.handle_keys(key_event), //TODO
                        }
                    }
                }
            }
            _ => {}
        };
        Ok(())
    }
    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, _area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(*buf.area());

        let preview_pane_block = Block::bordered().title("Preview");

        self.file_picker.render(layout[0], buf);
        preview_pane_block.clone().render(layout[1], buf);

        self.preview_pane
            .render(preview_pane_block.inner(layout[1]), buf);
    }
}
