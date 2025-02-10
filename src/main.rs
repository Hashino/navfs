use std::fs::File;
use std::io::Write;
use std::process::exit;
use ui::app::App;

mod parse_args;
mod theme;
mod tui;
mod ui;

enum ExitCode {
    Success = 0,
    ErrorCreatingFile = 1,
    ErrorWritingToFile = 2,
    GenericError = 3,
}

#[tokio::main]
async fn main() {
    let args = parse_args::parse_args();

    let mut terminal = tui::init().unwrap();
    let app_result = App::run(&mut terminal).await;
    tui::restore().unwrap();

    match app_result {
        Ok(path) => {
            // with this the user can use navfs to cd to the location last browsed
            args.get("file").map(|file| match File::create(file) {
                Ok(mut f) => match writeln!(f, "{}", path) {
                    Ok(_) => exit(0),
                    Err(e) => {
                        eprintln!("Error writing to file: {}", e);
                        exit(ExitCode::ErrorWritingToFile as i32);
                    }
                },
                Err(e) => {
                    eprintln!("Error creating file: {}", e);
                    exit(ExitCode::ErrorCreatingFile as i32);
                }
            });
        }
        Err(e) => {
            eprintln!("Error running app: {}", e);
            exit(ExitCode::GenericError as i32);
        }
    }

    exit(ExitCode::Success as i32);
}
