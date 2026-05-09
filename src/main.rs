use mdv::app::App;
use std::path::PathBuf;

fn main() -> iced::Result {
    let initial: Option<PathBuf> = std::env::args().nth(1).map(PathBuf::from);
    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .subscription(App::subscription)
        .font(include_bytes!("assets/fonts/Inter-Variable.ttf").as_slice())
        .font(include_bytes!("assets/fonts/JetBrainsMono-Regular.otf").as_slice())
        .default_font(iced::Font::with_name("Inter"))
        .run_with(move || App::new(initial.clone()))
}
