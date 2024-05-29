use ratatui::prelude::*;

pub struct Theme {
    pub normal: Style,
    pub selected: Style,
}

// TODO: ability to configure colors and styles in a .file

impl Theme {
    pub fn default() -> Theme {
        Theme {
            normal: Style::default(),
            selected: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        }
    }
}
