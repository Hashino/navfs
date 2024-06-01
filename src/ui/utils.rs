use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct Utils {}

impl Utils {
    /// helper function to create a centered rect
    /// * [width](u16) in characters of text
    /// * [height](u16) in lines of text
    pub fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
        // Cut the given rectangle into three vertical pieces
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((r.height - height) / 2),
                Constraint::Length(height),
                Constraint::Length((r.height - height) / 2),
            ])
            .split(r);

        // Then cut the middle vertical piece into three width-wise pieces
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length((r.width - width) / 2),
                Constraint::Length(width),
                Constraint::Length((r.width - width) / 2),
            ])
            .split(popup_layout[1])[1] // Return the middle chunk
    }
}
