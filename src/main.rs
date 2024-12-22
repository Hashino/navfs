use std::fs::File;
use std::io::Result;
use std::io::Write;
use std::process::exit;
use ui::app::App;

mod parse_args;
mod theme;
mod tui;
mod ui;

fn main() -> Result<()> {
    let args = parse_args::parse_args();

    let mut terminal = tui::init()?;
    let app_result = App::run(&mut terminal);
    tui::restore()?;

    match app_result {
        Ok(path) => {
            // with this the user can use navfs to cd to the location last browsed
            args.get("file").map(|file| match File::create(file) {
                Ok(mut f) => match writeln!(f, "{}", path) {
                    Ok(_) => exit(0),
                    Err(e) => {
                        eprintln!("Error writing to file: {}", e);
                        exit(2)
                    }
                },
                Err(e) => {
                    eprintln!("Error creating file: {}", e);
                    exit(1)
                }
            });
            Ok(())
        }
        Err(e) => Err(e),
    }
}
