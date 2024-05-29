//! Handles showing popups to the user:
//!
//! # Example
//! ```rust
//! let res: bool = show_confirmation("Confirmation", "Are you sure?");
//! if res {
//!    match function_that_returns_result() {
//!      Ok(..) => show_info("Success!", "Action done successfully".to_string()),
//!      Err(e) => show_error("Failed while doing action", e),
//!    }
//! }
//! ```

use std::io::{Error, Result};

use crossterm::event::{self, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{
        block::{self, Position},
        Block, Borders, Paragraph, Wrap,
    },
};

use crate::tui;

pub static KEYBINDINGS_INFO: &str = "[?]         - Show this window
[j/k]       - Navigate up/down in list
[l]         - Open directory/file
[h]         - Go to parent directory
[Space]     - Adds/Removes directory/files to/from buffer
[d]         - Delete directory/file
[bc]        - Clears buffer, ie: unselects all
[bd]        - Deletes all files in buffer
[Ctrl+ h/l] - Switch selected panel";

pub fn show_error(title: &str, error: Error) {
    show_info(title, error.to_string().clone());
}

/// shows a floating window in the center of screen
///
/// +---title---+
/// ∣           ∣
/// ∣ info      ∣
/// ∣           ∣
/// +-----------+
pub fn show_info(title: &str, info: String) {
    // try catch
    if let Err(error) = (|| -> Result<()> {
        //try block
        let mut term = tui::init()?;
        term.draw(|frame| {
            let block =
                Block::default() // block to wrap around message
                    .title(block::Title::from(title).alignment(Alignment::Center))
                    .title(
                        block::Title::from("Press any key to close")
                            .alignment(Alignment::Center)
                            .position(Position::Bottom),
                    )
                    .borders(Borders::ALL)
                    .on_blue()
                    .title_style(Style::default().add_modifier(Modifier::BOLD));

            let info_size = info.clone().chars().filter(|c| *c == '\n').count();
            let paragraph = Paragraph::new(info.clone()).wrap(Wrap { trim: false }); //message body

            let inner = block.inner(centered_rect(65, 5 + info_size as u16, frame.size()));

            frame.render_widget(paragraph.clone().block(block), inner);
        })?;

        loop {
            if event::poll(std::time::Duration::from_millis(16))? {
                if let event::Event::Key(..) = event::read()? {
                    break; // closes popup on any keypress
                }
            }
        }

        Ok(())
    })() {
        // catch block
        println!("Error displaying information: {error:?}\n Original information: {title}:{info}");
    }
}

/// shows a floating popup with Yes and No options and returns `true` if the user selects Yes
/// defaults to `false` if an error occur while rendering the popup
///
/// +---title---+
/// ∣           ∣
/// ∣ info      ∣
/// ∣           ∣
/// +-Y-------N-+
pub fn show_confirmation(title: &str, info: String) -> bool {
    // try catch
    match (|| -> Result<bool> {
        let mut term = tui::init()?;
        term.draw(|frame| {
            //outside block with the Yes/No options
            let block = Block::default()
                .title(block::Title::from(title).alignment(Alignment::Center))
                .title(
                    block::Title::from("[Y]es")
                        .alignment(Alignment::Left)
                        .position(Position::Bottom),
                )
                .title(
                    block::Title::from("[N]o")
                        .alignment(Alignment::Right)
                        .position(Position::Bottom),
                )
                .borders(Borders::ALL)
                .on_blue()
                .title_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::LightCyan),
                );

            // inside text
            let paragraph = Paragraph::new(info.clone()).style(Style::default().fg(Color::Cyan));
            let info_size = info.clone().chars().filter(|c| *c == '\n').count();

            let inner = block.inner(centered_rect(25, 5 + info_size as u16, frame.size()));

            frame.render_widget(paragraph.clone().block(block), inner);
        })?;

        loop {
            // stays in loop until user press one of the keys
            if event::poll(std::time::Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('y') => return Ok(true),
                        KeyCode::Char('n') => return Ok(false),
                        _ => continue,
                    };
                }
            }
        }
    })() {
        Ok(res) => res,
        Err(error) => {
            println!(
                "Error displaying information: {error:?}\n Original information: {title}:{info}"
            );
            false
        }
    }
}
/// helper function to create a centered rect
/// * [width](u16) in characters of text
/// * [height](u16) in lines of text
fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
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
