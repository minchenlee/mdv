use mdv::app::App;
use std::path::PathBuf;

fn main() -> iced::Result {
    {
        let fs = iced::advanced::graphics::text::font_system();
        if let Ok(mut guard) = fs.write() {
            guard.raw().db_mut().load_system_fonts();
        }
    }
    let initial: Option<PathBuf> = std::env::args().nth(1).map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let platform_specific = iced::window::settings::PlatformSpecific {
        title_hidden: true,
        titlebar_transparent: true,
        fullsize_content_view: true,
    };
    #[cfg(not(target_os = "macos"))]
    let platform_specific = iced::window::settings::PlatformSpecific::default();
    let window = iced::window::Settings {
        platform_specific,
        ..Default::default()
    };
    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .subscription(App::subscription)
        .window(window)
        .font(include_bytes!("assets/fonts/Inter-Variable.ttf").as_slice())
        .font(include_bytes!("assets/fonts/JetBrainsMono-Regular.otf").as_slice())
        .font(include_bytes!("assets/fonts/lucide.ttf").as_slice())
        .default_font(iced::Font::with_name("Inter"))
        .run_with(move || App::new(initial.clone()))
}
