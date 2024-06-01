//! Should handle only ui elements leaving actual file system implementations to the `Dir` type
use std::io::{Result, Stdout};

use crate::tui::Tui;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};
use ratatui::{
    buffer::Buffer,
    layout::{Direction, Layout},
    widgets::Widget,
};

use super::file_picker::{dir::Dir, file_picker::FilePicker};
use super::preview_pane::preview_pane::PreviewPane;

// to determine which panel is currently selected
#[derive(Eq, PartialEq)]
enum WhichPane {
    FilePicker,
    PreviewPane,
}

/// the App module exists to manage states between child widgets
/// it alsos handles global keybindings
///
/// [file_picker](FilePicker): left side widget
/// [preview_pane](PreviewPane): right side widget
/// [curr_selected](WhichPane): enum to which panel is currently selected
/// [exit](bool): if the app is done executing
/// [term](Terminal<CrosstermBackend<Stdout>>): the virtual terminal running the app
pub struct App<'a> {
    file_picker: FilePicker,
    preview_pane: PreviewPane,
    curr_selected: WhichPane,
    exit: bool,
    term: &'a mut Terminal<CrosstermBackend<Stdout>>,
}

impl App<'_> {
    /// executes the app and returns the path to the final working directory
    pub fn run(terminal: &mut Tui) -> Result<String> {
        let mut app = App {
            file_picker: FilePicker::new(true),
            preview_pane: PreviewPane::new(),
            curr_selected: WhichPane::FilePicker,
            exit: false,
            term: terminal,
        };

        app.file_picker.initialize(None, None);

        app.preview_pane.initialize(None);

        while !app.exit {
            // main render loop done inline to avoid borrows
            // handles rendering and constructing of Widgets
            // TODO: modularize Widget construction for better readability
            app.term.draw(|frame| {
                let _area = frame.size();
                let buf: &mut Buffer = frame.buffer_mut();

                // splits the screen into zones for each widget
                let layout_main_statusbar = Layout::default()
                    .direction(Direction::Vertical)
                    // the bottom space has exactly 1 line of space the rest is filled by the panels
                    .constraints(vec![Constraint::Fill(100), Constraint::Length(1)])
                    .split(*buf.area());
                let layout_picker_preview = Layout::default()
                    .direction(Direction::Horizontal)
                    // divides the panels area in 2
                    .constraints(vec![Constraint::Percentage(40), Constraint::Percentage(60)])
                    .split(layout_main_statusbar[0]);

                // block around preview pane
                let mut preview_pane_block_style = Style::default();
                if app.curr_selected == WhichPane::FilePicker {
                    preview_pane_block_style = preview_pane_block_style.add_modifier(Modifier::DIM);
                }

                let curr_selected_entry = app.file_picker.curr_sel_entry().clone();

                let curr_selected_name = Dir::get_entry_name(curr_selected_entry.clone())
                    + if curr_selected_entry.is_dir() {
                        "/"
                    } else {
                        ""
                    };

                app.file_picker.render(layout_picker_preview[0], buf);

                let entry_permissions = Dir::get_entry_metadata_to_display(curr_selected_entry);

                let preview_pane_block = Block::bordered()
                    .title(curr_selected_name)
                    .title_bottom(Line::from(entry_permissions).centered())
                    .style(preview_pane_block_style);

                preview_pane_block
                    .clone()
                    .render(layout_picker_preview[1], buf);
                // renders the preview pane inside the block
                app.preview_pane
                    .render(preview_pane_block.inner(layout_picker_preview[1]), buf);

                let status_bar_text = Dir::get_shortened_path(Dir::get_cur_dir().pathbuf);

                Paragraph::new(status_bar_text)
                    .add_modifier(Modifier::DIM)
                    .render(layout_main_statusbar[1], buf);
            })?;

            // handle key inputs
            app.handle_events()?;
        }

        // once the app finishes executing it returns the internal current directory
        Ok(Dir::get_cur_dir().pathbuf.display().to_string())
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                if key_event.code == KeyCode::Char('q') {
                    self.exit();
                } else if let KeyModifiers::CONTROL = key_event.modifiers {
                    match key_event.code {
                        // [Ctrl+l] switch to preview pane
                        KeyCode::Char('l') => {
                            self.curr_selected = WhichPane::PreviewPane;
                            self.file_picker.active = false;
                            self.preview_pane.active = true;
                        }
                        // [Ctrl+h] switch to file picker
                        KeyCode::Char('h') => {
                            self.curr_selected = WhichPane::FilePicker;
                            self.file_picker.active = true;
                            self.preview_pane.active = false;
                        }
                        _ => (),
                    }
                } else {
                    match self.curr_selected {
                        WhichPane::FilePicker => {
                            self.file_picker.handle_keys(key_event);
                            match key_event.code {
                                KeyCode::Char('h')
                                | KeyCode::Char('j')
                                | KeyCode::Char('k')
                                | KeyCode::Char('l') => {
                                    let curr = self.file_picker.curr_sel_entry();
                                    self.preview_pane.initialize(Some(curr));
                                }
                                _ => (),
                            }
                        }
                        WhichPane::PreviewPane => self.preview_pane.handle_keys(key_event),
                    }

                    // if the user pressses a key that makes it necessary to update the preview
                    // panel it does so. important that this happens after the child panel has
                    // handled their events.
                }
            }
            _ => {}
        };
        // checks if the applicattion needs to redraw itself
        self.redraw_if_needed();
        Ok(())
    }

    /// since the inbuilt StateFul widget only triggers a redraw when a property changes, we call it
    /// manually by using the resize method. specially important after a popup.
    /// TODO: investigate a better way of handling this
    fn redraw_if_needed(&mut self) {
        if self.file_picker.needs_redraw | self.preview_pane.needs_redraw {
            if let Ok(size) = self.term.size() {
                if let Ok(..) = self.term.resize(size) {};
            }
            self.file_picker.needs_redraw = false;
            self.preview_pane.needs_redraw = false;
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
