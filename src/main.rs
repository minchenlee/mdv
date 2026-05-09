use mdv::app::App;
use std::path::PathBuf;

fn main() -> iced::Result {
    let initial: Option<PathBuf> = std::env::args().nth(1).map(PathBuf::from);
    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .run_with(move || App::new(initial.clone()))
}
