use mdv::app::App;
use std::path::PathBuf;
use std::time::Instant;

fn main() -> iced::Result {
    let t0 = Instant::now();
    let bench = std::env::args().any(|a| a == "--benchmark-startup");
    if bench {
        // Set before any Iced threads spawn — set_var is unsound in multi-threaded contexts.
        std::env::set_var("MDV_BENCH_STARTUP", "1");
    }

    {
        let fs = iced::advanced::graphics::text::font_system();
        if let Ok(mut guard) = fs.write() {
            guard.raw().db_mut().load_system_fonts();
        }
    }
    let t_fonts = t0.elapsed();
    if bench {
        eprintln!("startup: fonts_loaded={:?}", t_fonts);
    }

    let initial: Option<PathBuf> = std::env::args()
        .skip(1)
        .find(|a| !a.starts_with("--"))
        .map(PathBuf::from);

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

    if bench {
        eprintln!("startup: pre_run={:?}", t0.elapsed());
    }

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
