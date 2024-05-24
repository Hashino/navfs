use std::io::Result;

use ui::app::App;

mod theme;
mod tui;
mod ui;

fn main() -> Result<()> {
    let mut terminal = tui::init()?;
    let app_result = App::default().run(&mut terminal);
    tui::restore()?;

    match app_result {
        Ok(path) => {
            // with this the user can use navfs to cd to the location last browsed
            println!("{}", path);
            Ok(())
        }
        Err(e) => Err(e),
    }
}
