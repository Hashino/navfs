use std::io::Result;

use crate::tui;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};
use ratatui::{
    buffer::Buffer,
    layout::{Direction, Layout, Rect},
    widgets::Widget,
    Frame,
};

use super::file_picker::{dir::get_cur_dir_path, widget::FilePicker};

enum WhichPane {
    FilePicker,
    PreviewPane,
}

#[derive(Default)]
pub struct App {
    file_picker: FilePicker,
    // preview_pane: PreviewPane,
    curr_selected: Option<WhichPane>,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<String> {
        self.file_picker.initialize();

        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            // terminal.draw(|frame| self.file_picker.render_frame(frame))?;
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
                match key_event.code {
                    KeyCode::Char('q') => self.exit(),
                    _ => {
                        match self.curr_selected {
                            Some(WhichPane::FilePicker) | None => {
                                self.file_picker.handle_keys(key_event)?
                            }
                            Some(WhichPane::PreviewPane) => (), //TODO
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
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(*buf.area());

        self.file_picker.render(layout[0], buf);

        let b = Block::default().title("TODO").borders(Borders::ALL);

        b.render(layout[1], buf)
    }
}
