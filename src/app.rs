use crate::ast::Block;
use crate::parser;
use crate::theme::{self, Palette, ThemeMode, Typography};
use iced::widget::{button, column, container, row as irow, scrollable, text, Space};
use iced::{Element, Length, Task, Theme};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    Open(PathBuf),
    OpenDialog,
    FileLoaded(Result<(PathBuf, String), String>),
    FileChanged(PathBuf),
    OpenLink(String),
    ToggleTheme,
    ScrollBy(f32),
    ScrollToTop,
    ScrollToBottom,
    Noop,
}

pub struct App {
    pub file: Option<PathBuf>,
    pub source: String,
    pub ast: Vec<Block>,
    pub theme_mode: ThemeMode,
    pub palette: Palette,
    pub typography: Typography,
    pub error: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        let mode = ThemeMode::System;
        Self {
            file: None,
            source: String::new(),
            ast: Vec::new(),
            theme_mode: mode,
            palette: theme::resolve(mode),
            typography: Typography::DEFAULT,
            error: None,
        }
    }
}

impl App {
    fn scroll_id() -> iced::widget::scrollable::Id {
        iced::widget::scrollable::Id::new("body")
    }

    pub fn new(initial: Option<PathBuf>) -> (Self, Task<Message>) {
        let app = Self::default();
        let task = match initial {
            Some(p) => Task::perform(load_file(p), Message::FileLoaded),
            None => Task::none(),
        };
        (app, task)
    }

    pub fn title(&self) -> String {
        match &self.file {
            Some(p) => format!(
                "mdv — {}",
                p.file_name().and_then(|n| n.to_str()).unwrap_or("?")
            ),
            None => "mdv".into(),
        }
    }

    pub fn theme(&self) -> Theme {
        match self.theme_mode {
            ThemeMode::Dark => Theme::Dark,
            ThemeMode::Light => Theme::Light,
            ThemeMode::System => match dark_light::detect() {
                dark_light::Mode::Dark => Theme::Dark,
                _ => Theme::Light,
            },
        }
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Open(p) => Task::perform(load_file(p), Message::FileLoaded),
            Message::OpenDialog => Task::perform(pick_file(), |opt| match opt {
                Some(p) => Message::Open(p),
                None => Message::Noop,
            }),
            Message::FileLoaded(Ok((path, src))) => {
                crate::recent::add(&path);
                self.ast = parser::parse(&src);
                self.source = src;
                self.error = None;
                self.file = Some(path);
                Task::none()
            }
            Message::FileChanged(p) => Task::perform(load_file(p), Message::FileLoaded),
            Message::OpenLink(url) => {
                let _ = open::that_detached(&url);
                Task::none()
            }
            Message::FileLoaded(Err(e)) => {
                self.error = Some(e);
                Task::none()
            }
            Message::ToggleTheme => {
                self.theme_mode = match self.theme_mode {
                    ThemeMode::Light => ThemeMode::Dark,
                    ThemeMode::Dark => ThemeMode::Light,
                    ThemeMode::System => ThemeMode::Dark,
                };
                self.palette = theme::resolve(self.theme_mode);
                Task::none()
            }
            Message::ScrollBy(dy) => iced::widget::scrollable::scroll_by(
                Self::scroll_id(),
                iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: dy },
            ),
            Message::ScrollToTop => iced::widget::scrollable::scroll_to(
                Self::scroll_id(),
                iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
            ),
            Message::ScrollToBottom => iced::widget::scrollable::scroll_to(
                Self::scroll_id(),
                iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: f32::MAX },
            ),
            Message::Noop => Task::none(),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        let dnd = iced::event::listen_with(|ev, _status, _id| match ev {
            iced::Event::Window(iced::window::Event::FileDropped(path)) => {
                Some(Message::Open(path))
            }
            _ => None,
        });
        let watcher =
            crate::watch::watch_subscription(self.file.clone()).map(Message::FileChanged);
        let keys = iced::keyboard::on_key_press(|key, mods| {
            use iced::keyboard::{key::Named, Key};
            match key {
                Key::Named(Named::ArrowDown) => Some(Message::ScrollBy(40.0)),
                Key::Named(Named::ArrowUp) => Some(Message::ScrollBy(-40.0)),
                Key::Named(Named::Space) if mods.shift() => Some(Message::ScrollBy(-400.0)),
                Key::Named(Named::Space) => Some(Message::ScrollBy(400.0)),
                Key::Named(Named::PageDown) => Some(Message::ScrollBy(400.0)),
                Key::Named(Named::PageUp) => Some(Message::ScrollBy(-400.0)),
                Key::Named(Named::Home) => Some(Message::ScrollToTop),
                Key::Named(Named::End) => Some(Message::ScrollToBottom),
                Key::Character(c) => match c.as_str() {
                    "j" => Some(Message::ScrollBy(40.0)),
                    "k" => Some(Message::ScrollBy(-40.0)),
                    "g" => Some(Message::ScrollToTop),
                    "G" => Some(Message::ScrollToBottom),
                    "t" if mods.command() || mods.control() => Some(Message::ToggleTheme),
                    "o" if mods.command() || mods.control() => Some(Message::OpenDialog),
                    _ => None,
                },
                _ => None,
            }
        });
        iced::Subscription::batch([dnd, watcher, keys])
    }

    pub fn view(&self) -> Element<'_, Message> {
        let theme_label = match self.theme_mode {
            ThemeMode::Dark => "☀",
            _ => "🌙",
        };

        let top = irow![
            button("Open").on_press(Message::OpenDialog),
            Space::with_width(Length::Fill),
            button(theme_label).on_press(Message::ToggleTheme),
        ]
        .padding(8)
        .spacing(8);

        let body: Element<'_, Message> = if let Some(err) = &self.error {
            text(format!("Error: {err}")).into()
        } else if self.file.is_none() {
            text("Drop a .md file or pass one on the command line").into()
        } else {
            crate::render::render(&self.ast, &self.palette, &self.typography)
        };

        let scrollable_body = scrollable(
            container(column![body].padding(24).spacing(16))
                .width(Length::Fill)
                .center_x(Length::Fill),
        )
        .id(Self::scroll_id());

        column![top, scrollable_body].into()
    }
}

async fn load_file(p: PathBuf) -> Result<(PathBuf, String), String> {
    let bytes = tokio::fs::read(&p).await.map_err(|e| e.to_string())?;
    let s = String::from_utf8_lossy(&bytes).into_owned();
    Ok((p, s))
}

async fn pick_file() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter("Markdown", &["md", "markdown"])
        .pick_file()
        .await
        .map(|h| h.path().to_path_buf())
}
