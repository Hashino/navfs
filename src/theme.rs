use ratatui::prelude::*;

#[derive(Debug)]
pub struct Theme {
    pub normal: Style,
    pub selected: Style,
}

// TODO: ability to configure colors and styles in a .file

impl Theme {
    pub fn new() -> Theme {
        Theme {
            normal: Style::default(),
            selected: Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
        }
    }
}
