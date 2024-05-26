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

pub fn show_error(title: &str, error: Error) {
     show_info(title, error.to_string().clone());
}

// shows a floating window in the center of screen
pub fn show_info(title: &str, info: String) {
    // try catch
    if let Err(error) = (|| -> Result<()> { //try block
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
    })() { // catch block
        println!(
            "Error displaying error: {error:?}\n Original error: {:?}:{:?}",
            title, error
        );
    }
}

pub fn show_confirmation(title: &str, info: String) -> Result<bool> {
    let mut term = tui::init()?;
    term.draw(|frame| {
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
}

/// helper function to create a centered rect
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
