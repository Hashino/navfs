use crossterm::event::KeyEvent;
use rascii_art::RenderOptions;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

pub struct PreviewPane {
    curr_entry: PathBuf,
    pub active: bool,
    pub needs_redraw: bool,
    file_picker: FilePicker,
}
use std::path::PathBuf;

use crate::ui::file_picker::{dir::Dir, file_picker::FilePicker};

impl PreviewPane {
    pub fn new() -> PreviewPane {
        PreviewPane {
            curr_entry: PathBuf::new(),
            active: false,
            needs_redraw: false,
            file_picker: FilePicker::new(false),
        }
    }

    pub fn initialize(&mut self, dir: Option<PathBuf>) {
        match dir.clone() {
            Some(value) => {
                self.curr_entry = value;
                if self.curr_entry.is_dir() {
                    self.file_picker.initialize(dir, Some(0));
                }
            }
            None => self.curr_entry = Dir::get_cur_dir().pathbuf,
        }
    }
    pub fn handle_keys(&mut self, key: KeyEvent) {
        if self.curr_entry.is_dir() {
            self.file_picker.handle_keys(key);
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let preview_inner_area = self.render_preview_pane_block(area, buf);

        // if its a directory shows another file picker
        if self.curr_entry.is_dir() {
            self.file_picker.active = self.active;
            self.file_picker.render(preview_inner_area, buf);
        } else if self.curr_entry.is_file() {
            match self.curr_entry.extension() {
                Some(extension) => match extension.to_str() {
                    Some("png") | Some("jpg") | Some("jpeg") => {
                        let mut buffer = String::new();

                        rascii_art::render_to(
                            self.curr_entry.to_str().unwrap(),
                            &mut buffer,
                            &RenderOptions::new()
                                .width(area.width as u32)
                                .height(area.height as u32)
                                .colored(false)
                                .charset(&[" ", "🞗", "🞘", "🞙", "🞚", "◈", "🞛", "❖", "⯁", "■"]),
                        )
                        .unwrap();

                        Paragraph::new(
                            buffer
                                .lines()
                                .map(|line| Line::raw(line))
                                .collect::<Vec<Line>>(),
                        )
                        .render(preview_inner_area, buf)
                    }
                    _ => {
                        let content = std::fs::read_to_string(self.curr_entry.clone())
                            .unwrap_or_else(|_| "Could not read file".to_string());
                        Paragraph::new(content).render(preview_inner_area, buf)
                    }
                },
                None => {
                    let content = std::fs::read_to_string(self.curr_entry.clone())
                        .unwrap_or_else(|_| "Could not read file".to_string());
                    Paragraph::new(content).render(preview_inner_area, buf);
                }
            }
        }
    }

    fn render_preview_pane_block(&mut self, area: Rect, buf: &mut Buffer) -> Rect {
        // block around preview pane
        let mut preview_pane_block_style = Style::default();

        if !self.active {
            preview_pane_block_style = preview_pane_block_style.add_modifier(Modifier::DIM);
        }

        let curr_selected_name = Dir::get_entry_name(self.curr_entry.clone())
            + if self.curr_entry.is_dir() { "/" } else { "" };

        let entry_permissions = Dir::get_entry_metadata_to_display(self.curr_entry.clone());

        let preview_pane_block = Block::bordered()
            .title(curr_selected_name)
            .title_bottom(Line::from(entry_permissions).centered())
            .style(preview_pane_block_style);

        preview_pane_block.clone().render(area, buf);
        // renders the preview pane inside the block
        return preview_pane_block.inner(area);
    }
}
