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

use super::file_picker::dir::{get_cur_dir, get_entry_name, get_entry_permissions_to_display, get_shortened_path};
use super::file_picker::widget::FilePicker;

#[derive(Eq, PartialEq)]
enum WhichPane {
    FilePicker,
    PreviewPane,
}

pub struct App<'a> {
    file_picker: FilePicker,
    preview_pane: FilePicker,
    curr_selected: WhichPane,
    exit: bool,
    term: &'a mut Terminal<CrosstermBackend<Stdout>>,
}

impl App<'_> {
    pub fn run(terminal: &mut Tui) -> Result<String> {
        let mut app = App {
            file_picker: FilePicker::new(true),
            preview_pane: FilePicker::new(false),
            curr_selected: WhichPane::FilePicker,
            exit: false,
            term: terminal,
        };

        app.file_picker.initialize(None, None);

        // WARN: remove after preview pane is fully implemented
        app.preview_pane.initialize(None, None);

        while !app.exit {
            // main render loop done inline to avoid borrows
            app.term.draw(|frame| {
                let _area = frame.size();
                let buf: &mut Buffer = frame.buffer_mut();

                // splits the screen into zones for each widget
                let layout_main_statusbar = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![Constraint::Fill(100), Constraint::Length(1)])
                    .split(*buf.area());
                let layout_picker_preview = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(layout_main_statusbar[0]);

                // block around preview pane
                let mut preview_pane_block_style = Style::default();
                if app.curr_selected == WhichPane::FilePicker {
                    preview_pane_block_style = preview_pane_block_style.add_modifier(Modifier::DIM);
                }

                let curr_selected_entry = app.file_picker.curr_sel_entry().clone();

                let curr_selected_name = get_entry_name(curr_selected_entry.clone())
                    + if curr_selected_entry.is_dir() {
                        "/"
                    } else {
                        ""
                    };

                let entry_permissions = get_entry_permissions_to_display(curr_selected_entry);

                let preview_pane_block = Block::bordered()
                    .title(curr_selected_name)
                    .title_bottom(Line::from(entry_permissions).centered())
                    .style(preview_pane_block_style);

                app.file_picker.render(layout_picker_preview[0], buf);

                preview_pane_block
                    .clone()
                    .render(layout_picker_preview[1], buf);
                // renders the preview pane inside the block
                app.preview_pane
                    .render(preview_pane_block.inner(layout_picker_preview[1]), buf);

                let status_bar_text = get_shortened_path(get_cur_dir().pathbuf);

                Paragraph::new(status_bar_text)
                    .add_modifier(Modifier::DIM)
                    .render(layout_main_statusbar[1], buf);
            })?;

            // handle key inputs
            app.handle_events()?;
        }

        Ok(get_cur_dir().display_name)
    }

    // hacky but reliable
    // on 1.0 the whole application will be rewritten with tokio
    fn redraw_if_needed(&mut self) {
        if self.file_picker.needs_redraw | self.preview_pane.needs_redraw {
            if let Ok(size) = self.term.size() {
                if let Ok(..) = self.term.resize(size) {};
            }
            self.file_picker.needs_redraw = false;
            self.preview_pane.needs_redraw = false;
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                if key_event.code == KeyCode::Char('q') {
                    self.exit();
                } else {
                    if let KeyModifiers::CONTROL = key_event.modifiers {
                        match key_event.code {
                            // [Ctrl+l] switch to preview pane
                            KeyCode::Char('l') => {
                                self.curr_selected = WhichPane::PreviewPane;
                                self.file_picker.selected = false;
                                self.preview_pane.selected = true;
                            }
                            // [Ctrl+h] switch to file picker
                            KeyCode::Char('h') => {
                                self.curr_selected = WhichPane::FilePicker;
                                self.file_picker.selected = true;
                                self.preview_pane.selected = false;
                            }
                            _ => (),
                        }
                    } else {
                        self.get_curr_sel_pane().handle_keys(key_event);

                        match key_event.code {
                            KeyCode::Char('h')
                            | KeyCode::Char('j')
                            | KeyCode::Char('k')
                            | KeyCode::Char('l') => {
                                if self.curr_selected == WhichPane::FilePicker {
                                    let curr = self.file_picker.curr_sel_entry();
                                    if curr.is_dir() {
                                        self.preview_pane.initialize(Some(curr), None);
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
            _ => {}
        };
        // checks if the applicattion needs to redraw itself
        self.redraw_if_needed();
        Ok(())
    }
    fn exit(&mut self) {
        self.exit = true;
    }

    fn get_curr_sel_pane(&mut self) -> &mut FilePicker {
        match self.curr_selected {
            WhichPane::FilePicker => &mut self.file_picker,
            WhichPane::PreviewPane => &mut self.preview_pane,
        }
    }
}
