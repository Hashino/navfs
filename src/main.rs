use std::io::Result;

use ui::app::App;

mod theme;
mod tui;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = tui::init()?;
    let app_result = App::run(&mut terminal);
    tui::restore()?;

    match app_result {
        Ok(path) => {
            // with this the user can use navfs to cd to the location last browsed
            print!("{}", path);
            Ok(())
        }
        Err(e) => Err(e),
    }
}
