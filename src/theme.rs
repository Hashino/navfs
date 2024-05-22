use ratatui::prelude::*;

#[derive(Debug)]
pub struct Theme {
    pub normal: Style,
    pub selected: Style,
}

impl Theme {
    pub fn new() -> Theme {
        Theme {
            normal: Style::default(),
            selected: Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
        }
    }
}
