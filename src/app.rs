use crate::ast::Block;
use crate::parser;
use crate::theme::{self, Palette, ThemeMode, Typography};
use iced::widget::{column, container, scrollable, text};
use iced::{Element, Length, Task, Theme};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    Open(PathBuf),
    FileLoaded(Result<(PathBuf, String), String>),
    ToggleTheme,
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
            Message::FileLoaded(Ok((path, src))) => {
                self.file = Some(path);
                self.ast = parser::parse(&src);
                self.source = src;
                self.error = None;
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
            Message::Noop => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let body: Element<'_, Message> = if let Some(err) = &self.error {
            text(format!("Error: {err}")).into()
        } else if self.file.is_none() {
            text("Drop a .md file or pass one on the command line").into()
        } else {
            crate::render::render(&self.ast, &self.palette, &self.typography)
        };

        let content = column![body].padding(24).spacing(16);
        scrollable(container(content).width(Length::Fill).center_x(Length::Fill)).into()
    }
}

async fn load_file(p: PathBuf) -> Result<(PathBuf, String), String> {
    let bytes = tokio::fs::read(&p).await.map_err(|e| e.to_string())?;
    let s = String::from_utf8_lossy(&bytes).into_owned();
    Ok((p, s))
}
