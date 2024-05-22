use std::io::Result;
use ui_elements::FilePicker;

mod theme;
mod tui;
mod ui_elements;

fn main() -> Result<()> {
    let mut terminal = tui::init()?;
    let app_result = FilePicker::default().run(&mut terminal);
    tui::restore()?;

    match app_result {
        Ok(path) => {
            println!("{:?}", path);
            Ok(())
        }
        Err(e) => {
            Err(e)
        }
    }
}
