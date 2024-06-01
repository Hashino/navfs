use crossterm::event::KeyEvent;
use image::DynamicImage;
use image::{imageops::FilterType, io::Reader as ImageReader};
use image_ascii::TextGenerator;
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

use crate::ui::{
    file_picker::{dir::Dir, file_picker::FilePicker},
    utils::Utils,
};

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
                        render_image_as_ascii(preview_inner_area, buf, self.curr_entry.clone());
                    }
                    _ => {
                        let text = Paragraph::new("Not yet implementd");
                        text.render(area, buf)
                    }
                },
                None => (),
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

// TODO: optmize this mess
fn render_image_as_ascii(area: Rect, buf: &mut Buffer, image_file: PathBuf) {
    let drawing_characters = &[' ', '🞌', '🞍', '🞐', '🞑', '🞒', '🞓', '🞕'];

    let dynamic_image: DynamicImage = ImageReader::open(image_file).unwrap().decode().unwrap();

    let fixed_widht_image = dynamic_image.resize_exact(
        (dynamic_image.width() as f32 * 2.5) as u32,
        dynamic_image.height(),
        FilterType::Nearest,
    );

    let resized_image = fixed_widht_image.thumbnail(area.width.into(), area.height.into());

    let ascii: String = TextGenerator::new(&resized_image)
        .set_density_chars(drawing_characters)
        .unwrap()
        .generate();

    let result: Vec<Line> = ascii.lines().map(|line| Line::raw(line)).collect();

    let centered_area = Utils::centered_rect(
        result[0].width().try_into().unwrap_or_default(),
        result.len().try_into().unwrap_or_default(),
        area,
    );

    Paragraph::new(result).render(centered_area, buf)
}
