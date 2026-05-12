use crate::ast::{Block, BlockId};
use crate::parser;
use crate::icon::{self, ic};
use crate::picker::{self, Picker, PickerMode};
use crate::render::Highlight;
use crate::search::{self, MatchPos};
use crate::theme::{self, Palette, ThemeMode, ThemePreset, Typography};
use crate::tree::{self, Node};
use iced::widget::{
    button, column, container, mouse_area, row as irow, scrollable, text, text_input, Column, Space,
};
use iced::{Background, Border, Color, Element, Length, Padding, Task, Theme};
use std::collections::HashSet;
use std::path::PathBuf;

const SIDEBAR_WIDTH: f32 = 280.0;
const READING_MAX: f32 = 780.0;
const TREE_INDENT: f32 = 14.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    None,
    FolderPicker,
    FileFinder,
    Command,
    ThemePicker,
}

#[derive(Debug, Clone)]
pub enum Message {
    Open(PathBuf),
    OpenWorkspace(PathBuf),
    OpenFolderPicker,
    OpenFileFinder,
    OpenCommandPalette,
    OpenThemePicker,
    CloseOverlay,
    PickerNavigate(PathBuf),
    PickerParent,
    PickerHome,
    PickerSelectFolderHere,
    OverlayQueryChanged(String),
    OverlayMove(isize),
    OverlayConfirm,
    OverlayDescend,
    FileLoaded(Result<(PathBuf, String), String>),
    FileChanged(PathBuf),
    OpenLink(String),
    ToggleTheme,
    SetTheme(ThemePreset),
    ToggleSidebar,
    TreeToggle(PathBuf),
    TreeMove(isize),
    TreeActivate,
    TreeToggleAtCursor,
    ScrollBy(f32),
    ScrollToTop,
    ScrollToBottom,
    ToggleSearch,
    QueryChanged(String),
    NextMatch,
    PrevMatch,
    TreeScrolled(iced::widget::scrollable::Viewport),
    OverlayScrolled(iced::widget::scrollable::Viewport),
    BodyScrolled(iced::widget::scrollable::Viewport),
    Noop,
}

pub struct App {
    pub file: Option<PathBuf>,
    pub source: String,
    pub ast: Vec<(BlockId, Block)>,
    pub theme_mode: ThemeMode,
    pub theme_preset: ThemePreset,
    pub palette: Palette,
    pub typography: Typography,
    pub error: Option<String>,
    pub query: String,
    pub matches: Vec<MatchPos>,
    pub match_idx: usize,
    pub search_open: bool,
    pub workspace: Option<PathBuf>,
    pub workspace_files: Vec<PathBuf>,
    pub workspace_tree: Option<Node>,
    pub expanded: HashSet<PathBuf>,
    pub sidebar_open: bool,
    pub tree_cursor: usize,
    pub overlay: Overlay,
    pub overlay_query: String,
    pub overlay_selected: usize,
    pub picker: Option<Picker>,
    pub tree_viewport: Option<iced::widget::scrollable::Viewport>,
    pub overlay_viewport: Option<iced::widget::scrollable::Viewport>,
    pub body_viewport: Option<iced::widget::scrollable::Viewport>,
    pub last_body_range: std::cell::Cell<(usize, usize)>,
    #[allow(dead_code)]
    pub first_frame_at: Option<std::time::Instant>,
    pub(crate) hl_cache: crate::highlight::HlCache,
    pub(crate) height_cache: crate::virt::HeightCache,
}

impl Default for App {
    fn default() -> Self {
        let mode = ThemeMode::System;
        let preset = theme::resolve_mode(mode);
        Self {
            file: None,
            source: String::new(),
            ast: Vec::new(),
            theme_mode: mode,
            theme_preset: preset,
            palette: theme::palette_for(preset),
            typography: Typography::DEFAULT,
            error: None,
            query: String::new(),
            matches: Vec::new(),
            match_idx: 0,
            search_open: false,
            workspace: None,
            workspace_files: Vec::new(),
            workspace_tree: None,
            expanded: HashSet::new(),
            sidebar_open: false,
            tree_cursor: 0,
            overlay: Overlay::None,
            overlay_query: String::new(),
            overlay_selected: 0,
            picker: None,
            tree_viewport: None,
            overlay_viewport: None,
            body_viewport: None,
            last_body_range: std::cell::Cell::new((0, 0)),
            first_frame_at: None,
            hl_cache: crate::highlight::HlCache::default(),
            height_cache: crate::virt::HeightCache::default(),
        }
    }
}

impl App {
    fn scroll_id() -> iced::widget::scrollable::Id {
        iced::widget::scrollable::Id::new("body")
    }
    fn tree_scroll_id() -> iced::widget::scrollable::Id {
        iced::widget::scrollable::Id::new("tree")
    }
    fn overlay_scroll_id() -> iced::widget::scrollable::Id {
        iced::widget::scrollable::Id::new("overlay")
    }

    fn scroll_tree_to_cursor(&self) -> Task<Message> {
        const ROW_H: f32 = 26.0;
        let Some(root) = &self.workspace_tree else { return Task::none() };
        let total = tree::flatten(root, &self.expanded).len();
        if total == 0 {
            return Task::none();
        }
        edge_scroll(
            Self::tree_scroll_id(),
            self.tree_viewport.as_ref(),
            self.tree_cursor,
            total,
            ROW_H,
        )
    }

    fn scroll_overlay_to_cursor(&self) -> Task<Message> {
        let (total, row_h) = match self.overlay {
            Overlay::FileFinder => (self.filtered_files().len().min(80), 32.0),
            Overlay::Command => (self.filtered_commands().len(), 32.0),
            Overlay::ThemePicker => (self.filtered_themes().len(), 32.0),
            Overlay::FolderPicker => (
                self.picker
                    .as_ref()
                    .map(|p| p.entries.len())
                    .unwrap_or(0),
                33.0,
            ),
            Overlay::None => (0, 32.0),
        };
        if total == 0 {
            return Task::none();
        }
        edge_scroll(
            Self::overlay_scroll_id(),
            self.overlay_viewport.as_ref(),
            self.overlay_selected,
            total,
            row_h,
        )
    }

    pub fn new(initial: Option<PathBuf>) -> (Self, Task<Message>) {
        let app = Self::default();
        let task = match initial {
            Some(p) => {
                if p.is_dir() {
                    Task::done(Message::OpenWorkspace(p))
                } else {
                    Task::perform(load_file(p), Message::FileLoaded)
                }
            }
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
        if self.theme_preset.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn scroll_to_current_match(&self) -> Task<Message> {
        if self.matches.is_empty() || self.ast.is_empty() {
            return Task::none();
        }
        let m = self.matches[self.match_idx];
        let total = self.ast.len().max(1) as f32;
        // Center the matched block roughly in viewport by biasing slightly down.
        let y = ((m.block as f32) / total - 0.15).clamp(0.0, 1.0);
        iced::widget::scrollable::snap_to(
            Self::scroll_id(),
            iced::widget::scrollable::RelativeOffset { x: 0.0, y },
        )
    }

    fn rebuild_matches(&mut self) {
        self.matches = search::find_in_blocks(&self.ast, &self.query);
        self.match_idx = 0;
    }

    pub fn blocks(&self) -> impl Iterator<Item = &Block> {
        self.ast.iter().map(|(_, b)| b)
    }

    fn open_overlay(&mut self, kind: Overlay) {
        self.overlay = kind;
        self.overlay_query.clear();
        self.overlay_selected = 0;
        self.overlay_viewport = None;
        if kind == Overlay::FolderPicker {
            let start = self
                .workspace
                .clone()
                .or_else(|| self.file.as_ref().and_then(|p| p.parent().map(|x| x.to_path_buf())));
            self.picker = Some(Picker::new(start, PickerMode::Folder));
        } else {
            self.picker = None;
        }
    }

    fn command_items(&self) -> Vec<(&'static str, Message)> {
        vec![
            ("Open Folder…  ⌘O", Message::OpenFolderPicker),
            ("Find File in Workspace…  ⌘P", Message::OpenFileFinder),
            ("Toggle Sidebar  ⌘B", Message::ToggleSidebar),
            ("Find in Document  ⌘F", Message::ToggleSearch),
            ("Cycle Theme  ⌘T", Message::ToggleTheme),
            ("Pick Theme…", Message::OpenThemePicker),
            ("Scroll to Top  g", Message::ScrollToTop),
            ("Scroll to Bottom  G", Message::ScrollToBottom),
        ]
    }

    fn filtered_files(&self) -> Vec<(PathBuf, String, i32)> {
        let root = self.workspace.as_ref();
        let mut scored: Vec<(PathBuf, String, i32)> = self
            .workspace_files
            .iter()
            .filter_map(|p| {
                let rel = root
                    .and_then(|r| p.strip_prefix(r).ok())
                    .map(|x| x.to_string_lossy().into_owned())
                    .unwrap_or_else(|| p.to_string_lossy().into_owned());
                let s = picker::fuzzy_score(&self.overlay_query, &rel)?;
                Some((p.clone(), rel, s))
            })
            .collect();
        scored.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.1.cmp(&b.1)));
        scored.truncate(200);
        scored
    }

    fn filtered_commands(&self) -> Vec<(&'static str, Message, i32)> {
        let mut scored: Vec<(&'static str, Message, i32)> = self
            .command_items()
            .into_iter()
            .filter_map(|(label, msg)| {
                let s = picker::fuzzy_score(&self.overlay_query, label)?;
                Some((label, msg, s))
            })
            .collect();
        scored.sort_by(|a, b| b.2.cmp(&a.2));
        scored
    }

    fn filtered_themes(&self) -> Vec<ThemePreset> {
        ThemePreset::ALL
            .into_iter()
            .filter(|t| {
                if self.overlay_query.is_empty() {
                    true
                } else {
                    picker::fuzzy_score(&self.overlay_query, t.label()).is_some()
                }
            })
            .collect()
    }

    fn reveal_current_file(&mut self) {
        let (Some(ws), Some(file)) = (self.workspace.as_ref(), self.file.as_ref()) else {
            return;
        };
        for a in tree::ancestors_of(ws, file) {
            self.expanded.insert(a);
        }
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Open(p) => Task::perform(load_file(p), Message::FileLoaded),
            Message::OpenWorkspace(p) => {
                self.workspace_files = picker::walk_markdown(&p, 8, 5000);
                self.workspace_tree = Some(tree::build(&p));
                self.expanded.clear();
                if let Some(t) = &self.workspace_tree {
                    self.expanded.insert(t.path.clone());
                }
                self.workspace = Some(p);
                self.sidebar_open = true;
                self.tree_cursor = 0;
                self.overlay = Overlay::None;
                self.picker = None;
                Task::none()
            }
            Message::OpenFolderPicker => {
                self.open_overlay(Overlay::FolderPicker);
                Task::none()
            }
            Message::OpenFileFinder => {
                if self.workspace.is_some() {
                    self.open_overlay(Overlay::FileFinder);
                } else {
                    self.open_overlay(Overlay::FolderPicker);
                }
                Task::none()
            }
            Message::OpenCommandPalette => {
                self.open_overlay(Overlay::Command);
                Task::none()
            }
            Message::OpenThemePicker => {
                self.open_overlay(Overlay::ThemePicker);
                Task::none()
            }
            Message::CloseOverlay => {
                self.overlay = Overlay::None;
                self.picker = None;
                Task::none()
            }
            Message::PickerNavigate(p) => {
                if let Some(pk) = self.picker.as_mut() {
                    if p.is_dir() {
                        pk.navigate_to(p);
                    }
                }
                Task::none()
            }
            Message::PickerParent => {
                if let Some(pk) = self.picker.as_mut() {
                    pk.parent();
                    self.overlay_selected = 0;
                }
                Task::none()
            }
            Message::PickerHome => {
                if let Some(home) = Picker::home() {
                    if let Some(pk) = self.picker.as_mut() {
                        pk.navigate_to(home);
                    }
                }
                Task::none()
            }
            Message::PickerSelectFolderHere => {
                if let Some(pk) = &self.picker {
                    let p = pk.cwd.clone();
                    return Task::done(Message::OpenWorkspace(p));
                }
                Task::none()
            }
            Message::OverlayQueryChanged(q) => {
                self.overlay_query = q;
                self.overlay_selected = 0;
                Task::none()
            }
            Message::OverlayMove(d) => {
                let len = match self.overlay {
                    Overlay::FileFinder => self.filtered_files().len(),
                    Overlay::Command => self.filtered_commands().len(),
                    Overlay::ThemePicker => self.filtered_themes().len(),
                    Overlay::FolderPicker => self
                        .picker
                        .as_ref()
                        .map(|p| p.entries.len())
                        .unwrap_or(0),
                    Overlay::None => 0,
                };
                if len == 0 {
                    return Task::none();
                }
                let len = len as isize;
                self.overlay_selected =
                    ((self.overlay_selected as isize + d).rem_euclid(len)) as usize;
                self.scroll_overlay_to_cursor()
            }
            Message::OverlayConfirm => match self.overlay {
                Overlay::FileFinder => {
                    let files = self.filtered_files();
                    if let Some((p, _, _)) = files.get(self.overlay_selected).cloned() {
                        self.overlay = Overlay::None;
                        return Task::perform(load_file(p), Message::FileLoaded);
                    }
                    Task::none()
                }
                Overlay::Command => {
                    let cmds = self.filtered_commands();
                    if let Some((_, msg, _)) = cmds.get(self.overlay_selected).cloned() {
                        self.overlay = Overlay::None;
                        return Task::done(msg);
                    }
                    Task::none()
                }
                Overlay::ThemePicker => {
                    let themes = self.filtered_themes();
                    if let Some(t) = themes.get(self.overlay_selected).copied() {
                        self.overlay = Overlay::None;
                        return Task::done(Message::SetTheme(t));
                    }
                    Task::none()
                }
                Overlay::FolderPicker => {
                    if let Some(pk) = self.picker.as_ref() {
                        if let Some(e) = pk.entries.get(self.overlay_selected).cloned() {
                            if e.is_dir {
                                self.overlay = Overlay::None;
                                self.picker = None;
                                return Task::done(Message::OpenWorkspace(e.path));
                            } else if e.is_md {
                                self.overlay = Overlay::None;
                                self.picker = None;
                                return Task::perform(load_file(e.path), Message::FileLoaded);
                            }
                        }
                    }
                    Task::none()
                }
                Overlay::None => Task::none(),
            },
            Message::OverlayDescend => {
                if self.overlay == Overlay::FolderPicker {
                    if let Some(pk) = self.picker.as_mut() {
                        if let Some(e) = pk.entries.get(self.overlay_selected).cloned() {
                            if e.is_dir {
                                pk.navigate_to(e.path);
                                self.overlay_selected = 0;
                                return self.scroll_overlay_to_cursor();
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::FileLoaded(Ok((path, src))) => {
                crate::recent::add(&path);
                let mut parsed = parser::parse(&src);
                for (_id, b) in parsed.iter_mut() {
                    if let Block::CodeBlock { lang: Some(l), code, spans } = b {
                        if spans.is_empty() {
                            *spans = self.hl_cache.highlight(l, code);
                        }
                    }
                }
                self.ast = parsed;
                self.source = src;
                self.error = None;
                self.file = Some(path);
                self.rebuild_matches();
                self.reveal_current_file();
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
                self.theme_preset = self.theme_preset.next();
                self.palette = theme::palette_for(self.theme_preset);
                Task::none()
            }
            Message::SetTheme(t) => {
                self.theme_preset = t;
                self.palette = theme::palette_for(t);
                Task::none()
            }
            Message::ToggleSidebar => {
                self.sidebar_open = !self.sidebar_open;
                Task::none()
            }
            Message::TreeToggle(p) => {
                if !self.expanded.remove(&p) {
                    self.expanded.insert(p);
                }
                Task::none()
            }
            Message::TreeMove(d) => {
                let Some(root) = &self.workspace_tree else { return Task::none() };
                let len = tree::flatten(root, &self.expanded).len();
                if len == 0 {
                    return Task::none();
                }
                let len_i = len as isize;
                self.tree_cursor =
                    ((self.tree_cursor as isize + d).rem_euclid(len_i)) as usize;
                self.scroll_tree_to_cursor()
            }
            Message::TreeActivate => {
                let Some(root) = &self.workspace_tree else { return Task::none() };
                let rows = tree::flatten(root, &self.expanded);
                let Some(r) = rows.get(self.tree_cursor) else { return Task::none() };
                if r.node.is_dir {
                    let p = r.node.path.clone();
                    if !self.expanded.remove(&p) {
                        self.expanded.insert(p);
                    }
                    Task::none()
                } else {
                    let p = r.node.path.clone();
                    Task::perform(load_file(p), Message::FileLoaded)
                }
            }
            Message::TreeToggleAtCursor => {
                let Some(root) = &self.workspace_tree else { return Task::none() };
                let rows = tree::flatten(root, &self.expanded);
                let Some(r) = rows.get(self.tree_cursor) else { return Task::none() };
                if r.node.is_dir {
                    let p = r.node.path.clone();
                    if !self.expanded.remove(&p) {
                        self.expanded.insert(p);
                    }
                }
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
            Message::ToggleSearch => {
                self.search_open = !self.search_open;
                if !self.search_open {
                    self.query.clear();
                    self.matches.clear();
                    self.match_idx = 0;
                }
                Task::none()
            }
            Message::QueryChanged(q) => {
                self.query = q;
                self.rebuild_matches();
                self.scroll_to_current_match()
            }
            Message::NextMatch => {
                if !self.matches.is_empty() {
                    self.match_idx = (self.match_idx + 1) % self.matches.len();
                }
                self.scroll_to_current_match()
            }
            Message::PrevMatch => {
                if !self.matches.is_empty() {
                    self.match_idx =
                        (self.match_idx + self.matches.len() - 1) % self.matches.len();
                }
                self.scroll_to_current_match()
            }
            Message::TreeScrolled(v) => {
                self.tree_viewport = Some(v);
                Task::none()
            }
            Message::OverlayScrolled(v) => {
                self.overlay_viewport = Some(v);
                Task::none()
            }
            Message::BodyScrolled(v) => {
                // Only mutate state (→ trigger view rebuild) when the visible
                // virt-range actually changed. Tiny scroll deltas within the
                // already-rendered window become no-ops, eliminating per-frame
                // rich_text rebuild jank during smooth scrolling.
                let new_range = crate::virt::visible_range(
                    &self.ast,
                    &self.height_cache,
                    v.absolute_offset().y,
                    v.bounds().height,
                    5,
                );
                let prev = self.last_body_range.get();
                let need_search_update = self.search_open && !self.query.is_empty();
                if new_range != prev || need_search_update {
                    self.body_viewport = Some(v);
                    self.last_body_range.set(new_range);
                }
                Task::none()
            }
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
        let focused = self.search_open;
        let overlay_open = self.overlay != Overlay::None;
        let tree_active = self.sidebar_open && self.workspace.is_some();
        let keys = iced::event::listen()
            .with((focused, overlay_open, tree_active))
            .map(|((focused, overlay_open, tree_active), ev)| {
                use iced::keyboard::{key::Named, Event as KEv, Key};
                let (key, mods) = match ev {
                    iced::Event::Keyboard(KEv::KeyPressed { key, modifiers, .. }) => {
                        (key, modifiers)
                    }
                    _ => return Message::Noop,
                };
                let cmd = mods.command() || mods.control();
                if let Key::Character(c) = &key {
                    match c.as_str() {
                        "p" if cmd => return Message::OpenFileFinder,
                        "k" if cmd => return Message::OpenCommandPalette,
                        "o" if cmd => return Message::OpenFolderPicker,
                        "b" if cmd => return Message::ToggleSidebar,
                        "f" if cmd => return Message::ToggleSearch,
                        "t" if cmd => return Message::ToggleTheme,
                        _ => {}
                    }
                }
                if matches!(&key, Key::Named(Named::Escape)) {
                    if overlay_open {
                        return Message::CloseOverlay;
                    }
                    if focused {
                        return Message::ToggleSearch;
                    }
                }
                if overlay_open {
                    return match key {
                        Key::Named(Named::ArrowDown) => Message::OverlayMove(1),
                        Key::Named(Named::ArrowUp) => Message::OverlayMove(-1),
                        Key::Named(Named::Enter) => Message::OverlayConfirm,
                        Key::Named(Named::Space) => Message::OverlayDescend,
                        Key::Named(Named::ArrowRight) => Message::OverlayDescend,
                        Key::Named(Named::ArrowLeft) => Message::PickerParent,
                        _ => Message::Noop,
                    };
                }
                if focused {
                    if matches!(&key, Key::Named(Named::Enter)) {
                        return if mods.shift() {
                            Message::PrevMatch
                        } else {
                            Message::NextMatch
                        };
                    }
                    return Message::Noop;
                }
                let m: Option<Message> = match key {
                    Key::Named(Named::ArrowDown) if tree_active => Some(Message::TreeMove(1)),
                    Key::Named(Named::ArrowUp) if tree_active => Some(Message::TreeMove(-1)),
                    Key::Named(Named::Enter) if tree_active => Some(Message::TreeActivate),
                    Key::Named(Named::Space) if tree_active => {
                        Some(Message::TreeToggleAtCursor)
                    }
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
                        _ => None,
                    },
                    _ => None,
                };
                m.unwrap_or(Message::Noop)
            });
        iced::Subscription::batch([dnd, watcher, keys])
    }

    pub fn view(&self) -> Element<'_, Message> {
        {
            use std::sync::OnceLock;
            // Print first_view BEFORE the font-load block so the timing reflects
            // when the window can actually paint (font load runs lazily after).
            static BENCH: OnceLock<bool> = OnceLock::new();
            if *BENCH.get_or_init(|| std::env::var_os("MDV_BENCH_STARTUP").is_some()) {
                static FIRST: OnceLock<()> = OnceLock::new();
                FIRST.get_or_init(|| {
                    if let Some(d) = crate::bench::since_process_start() {
                        eprintln!("startup: first_view={:?}", d);
                    }
                });
            }
            // Deferred from main(): first view pays ~270ms font scan instead of blocking window paint.
            static FONTS_LOADED: OnceLock<()> = OnceLock::new();
            FONTS_LOADED.get_or_init(|| {
                let fs = iced::advanced::graphics::text::font_system();
                if let Ok(mut guard) = fs.write() {
                    guard.raw().db_mut().load_system_fonts();
                }
                if std::env::var_os("MDV_BENCH_STARTUP").is_some() {
                    if let Some(d) = crate::bench::since_process_start() {
                        eprintln!("startup: fonts_loaded={:?}", d);
                    }
                }
            });
        }
        let pal = self.palette;

        let reader: Element<'_, Message> = if let Some(err) = &self.error {
            centered_card(
                column![
                    text("Couldn't open file").size(20).color(pal.fg),
                    text(err.clone()).color(pal.muted).size(13),
                    Space::with_height(8),
                    primary_button("Open Folder", pal).on_press(Message::OpenFolderPicker),
                ]
                .spacing(10)
                .align_x(iced::Alignment::Center)
                .into(),
                pal,
            )
        } else if self.file.is_none() {
            welcome_view(pal)
        } else {
            let hl = Highlight {
                query: self.query.clone(),
                current_block: self.matches.get(self.match_idx).map(|m| m.block),
                current_in_block: self
                    .matches
                    .get(self.match_idx)
                    .map(|m| m.in_block)
                    .unwrap_or(0),
            };
            let body = crate::render::render(
                &self.ast,
                &pal,
                &self.typography,
                &hl,
                self.body_viewport.as_ref(),
                &self.height_cache,
            );
            scrollable(
                container(
                    container(body)
                        .max_width(READING_MAX)
                        .width(Length::Shrink),
                )
                .padding(Padding::from([56, 32]))
                .center_x(Length::Fill)
                .width(Length::Fill),
            )
            .id(Self::scroll_id())
            .height(Length::Fill)
            .direction(slim_scroll_direction())
            .style(move |_, status| sleek_scrollable_style(status, pal))
            .into()
        };

        let reader_with_search: Element<'_, Message> = if self.search_open {
            column![
                search_bar_view(&self.query, &self.matches, self.match_idx, pal),
                reader,
            ]
            .into()
        } else {
            reader.into()
        };

        let main_area: Element<'_, Message> = if self.sidebar_open && self.workspace.is_some() {
            irow![
                sidebar_view(self, pal),
                vertical_rule(pal),
                container(reader_with_search)
                    .width(Length::Fill)
                    .height(Length::Fill),
            ]
            .into()
        } else {
            container(reader_with_search)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        let main = container(main_area)
            .style(move |_| container::Style {
                background: Some(pal.bg.into()),
                ..Default::default()
            })
            .width(Length::Fill)
            .height(Length::Fill);

        match self.overlay {
            Overlay::None => main.into(),
            Overlay::FolderPicker => {
                let ov = folder_picker_overlay(self.picker.as_ref(), self.overlay_selected, pal);
                iced::widget::stack![main, ov].into()
            }
            Overlay::FileFinder => {
                let files = self.filtered_files();
                let ov = file_finder_overlay(
                    &self.overlay_query,
                    files,
                    self.overlay_selected,
                    pal,
                );
                iced::widget::stack![main, ov].into()
            }
            Overlay::Command => {
                let cmds = self.filtered_commands();
                let ov =
                    command_overlay(&self.overlay_query, cmds, self.overlay_selected, pal);
                iced::widget::stack![main, ov].into()
            }
            Overlay::ThemePicker => {
                let themes = self.filtered_themes();
                let ov = theme_overlay(
                    &self.overlay_query,
                    themes,
                    self.overlay_selected,
                    self.theme_preset,
                    pal,
                );
                iced::widget::stack![main, ov].into()
            }
        }
    }
}

fn edge_scroll(
    id: iced::widget::scrollable::Id,
    viewport: Option<&iced::widget::scrollable::Viewport>,
    cursor: usize,
    total: usize,
    row_h: f32,
) -> Task<Message> {
    // List inside scrollable has small top/bottom padding (~6-8px each). Pad cur_bot
    // so the bottom edge of the *last* row is fully revealed instead of clipped.
    const PAD: f32 = 8.0;
    let Some(v) = viewport else {
        if total <= 1 {
            return Task::none();
        }
        let y = (cursor as f32 / (total - 1) as f32).clamp(0.0, 1.0);
        return iced::widget::scrollable::snap_to(
            id,
            iced::widget::scrollable::RelativeOffset { x: 0.0, y },
        );
    };
    let cur_top = cursor as f32 * row_h;
    let cur_bot = cur_top + row_h + PAD;
    let off = v.absolute_offset();
    let view_top = off.y;
    let view_h = v.bounds().height;
    let view_bot = view_top + view_h;
    let new_y = if cur_top < view_top {
        cur_top
    } else if cur_bot > view_bot {
        cur_bot - view_h
    } else {
        return Task::none();
    };
    iced::widget::scrollable::scroll_to(
        id,
        iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: new_y.max(0.0) },
    )
}

fn vertical_rule<'a>(pal: Palette) -> Element<'a, Message> {
    container(Space::with_width(1.0))
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(pal.rule.into()),
            ..Default::default()
        })
        .into()
}

fn welcome_view<'a>(pal: Palette) -> Element<'a, Message> {
    let kbd = |label: &'static str, key: &'static str| {
        irow![
            container(text(key).size(11).color(pal.muted).font(iced::Font::with_name("JetBrains Mono")))
                .padding(Padding::from([2, 7]))
                .style(move |_| container::Style {
                    background: Some(pal.surface_alt.into()),
                    border: Border {
                        color: pal.rule,
                        width: 1.0,
                        radius: 5.0.into(),
                    },
                    ..Default::default()
                }),
            text(label).size(13).color(pal.muted),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
    };
    centered_card(
        column![
            text("mdv").size(40).color(pal.fg),
            text("Lightweight, beautiful, native markdown viewer").size(14).color(pal.muted),
            Space::with_height(22),
            kbd("Open Folder", "⌘O"),
            kbd("Find File in Workspace", "⌘P"),
            kbd("Command Palette", "⌘K"),
            kbd("Toggle Sidebar", "⌘B"),
            kbd("Find in Document", "⌘F"),
            kbd("Cycle Theme", "⌘T"),
        ]
        .spacing(8)
        .align_x(iced::Alignment::Start)
        .into(),
        pal,
    )
}

fn search_bar_view<'a>(
    query: &'a str,
    matches: &'a [MatchPos],
    idx: usize,
    pal: Palette,
) -> Element<'a, Message> {
    let counter = if matches.is_empty() {
        if query.is_empty() { String::new() } else { "0/0".into() }
    } else {
        format!("{}/{}", idx + 1, matches.len())
    };
    container(
        irow![
            text("Find").size(12).color(pal.subtle),
            text_input("type to search…", query)
                .on_input(Message::QueryChanged)
                .on_submit(Message::NextMatch)
                .padding(Padding::from([6, 10]))
                .size(13)
                .style(move |_, _| iced::widget::text_input::Style {
                    background: pal.surface_alt.into(),
                    border: Border {
                        color: pal.rule,
                        width: 1.0,
                        radius: 999.0.into(),
                    },
                    icon: pal.muted,
                    placeholder: pal.subtle,
                    value: pal.fg,
                    selection: pal.selection,
                })
                .width(Length::Fill),
            text(counter).color(pal.muted).size(12),
            ghost_lu(ic::CHEVRON_LEFT, pal).on_press(Message::PrevMatch),
            ghost_lu(ic::CHEVRON_RIGHT, pal).on_press(Message::NextMatch),
            ghost_lu(ic::X, pal).on_press(Message::ToggleSearch),
        ]
        .padding(Padding::from([8, 14]))
        .spacing(10)
        .align_y(iced::Alignment::Center),
    )
    .style(move |_| container::Style {
        background: Some(pal.surface.into()),
        border: Border {
            color: pal.rule,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

fn sidebar_view<'a>(app: &'a App, pal: Palette) -> Element<'a, Message> {
    let ws = app.workspace.as_ref().unwrap();
    let ws_name = ws
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("workspace");
    let header = container(
        irow![
            text(ws_name.to_string().to_uppercase())
                .size(11)
                .color(pal.muted),
            Space::with_width(Length::Fill),
            ghost_lu(ic::ARROW_UP_FROM_LINE, pal).on_press(Message::OpenFolderPicker),
        ]
        .padding(Padding::from([10, 14]))
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill);

    let mut list = Column::new().spacing(0).padding(Padding::from([4, 4]));
    if let Some(tree_root) = &app.workspace_tree {
        let rows = tree::flatten(tree_root, &app.expanded);
        let current = app.file.as_ref();
        let cursor = app.tree_cursor;
        for (i, r) in rows.iter().enumerate() {
            let row_el = tree_row(r.node, r.depth, &app.expanded, current, i == cursor, pal);
            list = list.push(row_el);
        }
    }
    let body = scrollable(list)
        .id(App::tree_scroll_id())
        .height(Length::Fill)
        .on_scroll(Message::TreeScrolled)
        .direction(slim_scroll_direction())
            .style(move |_, status| sleek_scrollable_style(status, pal));

    container(column![header, body])
        .width(Length::Fixed(SIDEBAR_WIDTH))
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(pal.sidebar.into()),
            ..Default::default()
        })
        .into()
}

fn tree_row<'a>(
    node: &'a Node,
    depth: usize,
    expanded: &HashSet<PathBuf>,
    current: Option<&'a PathBuf>,
    is_cursor: bool,
    pal: Palette,
) -> Element<'a, Message> {
    let is_current = !node.is_dir && current.map(|c| c == &node.path).unwrap_or(false);
    let path = node.path.clone();

    // Indent area with vertical guides per ancestor level.
    let mut indent = iced::widget::Row::new();
    for _ in 0..depth {
        indent = indent.push(indent_guide(pal));
    }

    let chevron: Element<'a, Message> = if node.is_dir {
        let open = expanded.contains(&node.path);
        let g = if open { ic::CHEVRON_DOWN } else { ic::CHEVRON_RIGHT };
        icon::glyph(g, 12.0, pal.subtle).into()
    } else {
        Space::with_width(12.0).into()
    };

    let label_color = if is_current {
        pal.fg
    } else if node.is_dir {
        pal.fg
    } else {
        pal.muted
    };
    let label_weight = if node.is_dir {
        iced::font::Weight::Medium
    } else {
        iced::font::Weight::Normal
    };
    let mut label_font = iced::Font::with_name("Inter");
    label_font.weight = label_weight;
    let label = text(node.name.as_str())
        .size(13)
        .color(label_color)
        .font(label_font)
        .wrapping(text::Wrapping::None);

    let leaf_icon: Element<'a, Message> = if node.is_dir {
        let open = expanded.contains(&node.path);
        let g = if open { ic::FOLDER_OPEN } else { ic::FOLDER };
        icon::glyph(g, 13.0, pal.subtle).into()
    } else {
        icon::glyph(ic::FILE_TEXT, 13.0, pal.subtle).into()
    };
    let content = irow![
        indent,
        container(chevron).width(Length::Fixed(14.0)),
        Space::with_width(4.0),
        leaf_icon,
        Space::with_width(7.0),
        label,
    ]
    .align_y(iced::Alignment::Center)
    .spacing(0);

    let on_press = if node.is_dir {
        Message::TreeToggle(path)
    } else {
        Message::Open(path)
    };

    button(content)
        .padding(Padding::from([4, 8]))
        .width(Length::Fill)
        .height(Length::Fixed(26.0))
        .style(move |_, status| {
            let bg = if is_current {
                Some(Background::Color(pal.tree_selected_bg))
            } else if is_cursor {
                Some(Background::Color(pal.surface_alt))
            } else {
                match status {
                    button::Status::Hovered => Some(Background::Color(pal.surface_alt)),
                    _ => None,
                }
            };
            let show_border = is_current || is_cursor;
            button::Style {
                background: bg,
                text_color: pal.fg,
                border: Border {
                    color: if show_border { pal.tree_selected_border } else { Color::TRANSPARENT },
                    width: if show_border { 1.0 } else { 0.0 },
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        })
        .on_press(on_press)
        .into()
}

fn indent_guide<'a>(pal: Palette) -> Element<'a, Message> {
    container(
        container(Space::with_height(Length::Fill))
            .width(Length::Fixed(1.0))
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(pal.indent_guide.into()),
                ..Default::default()
            }),
    )
    .width(Length::Fixed(TREE_INDENT))
    .height(Length::Fixed(26.0))
    .center_x(Length::Fixed(TREE_INDENT))
    .into()
}

fn primary_button<'a>(label: &'a str, pal: Palette) -> button::Button<'a, Message> {
    button(text(label).size(13))
        .padding(Padding::from([8, 14]))
        .style(move |_, status| {
            let bg = match status {
                button::Status::Hovered => Color { a: 0.92, ..pal.accent },
                button::Status::Pressed => Color { a: 0.80, ..pal.accent },
                _ => pal.accent,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: pal.accent_fg,
                border: Border { radius: 999.0.into(), ..Default::default() },
                ..Default::default()
            }
        })
}

fn ghost_lu<'a>(code: char, pal: Palette) -> button::Button<'a, Message> {
    button(icon::glyph(code, 14.0, pal.muted))
        .padding(Padding::from([4, 8]))
        .style(move |_, status| button::Style {
            background: match status {
                button::Status::Hovered => Some(Background::Color(pal.surface_alt)),
                _ => None,
            },
            text_color: pal.muted,
            border: Border { radius: 999.0.into(), ..Default::default() },
            ..Default::default()
        })
}

fn centered_card<'a>(content: Element<'a, Message>, pal: Palette) -> Element<'a, Message> {
    container(
        container(content)
            .padding(Padding::from([40, 56]))
            .style(move |_| container::Style {
                background: Some(pal.surface.into()),
                border: Border {
                    color: pal.rule,
                    width: 1.0,
                    radius: 16.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.18),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 30.0,
                },
                ..Default::default()
            }),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

fn folder_picker_overlay<'a>(
    pk: Option<&'a Picker>,
    selected: usize,
    pal: Palette,
) -> Element<'a, Message> {
    let panel: Element<'a, Message> = if let Some(pk) = pk {
        let crumbs = pk.breadcrumbs();
        let mut crumb_row = iced::widget::Row::new()
            .spacing(2)
            .align_y(iced::Alignment::Center);
        crumb_row = crumb_row.push(ghost_lu(ic::HOME, pal).on_press(Message::PickerHome));
        crumb_row = crumb_row.push(ghost_lu(ic::ARROW_UP, pal).on_press(Message::PickerParent));
        crumb_row = crumb_row.push(Space::with_width(8));
        for (i, (label, path)) in crumbs.iter().enumerate() {
            if i > 0 {
                crumb_row = crumb_row.push(text("/").color(pal.subtle).size(12));
            }
            let label = label.clone();
            let path = path.clone();
            crumb_row = crumb_row.push(
                button(text(label).size(12).color(pal.fg))
                    .padding(Padding::from([3, 6]))
                    .style(move |_, status| button::Style {
                        background: match status {
                            button::Status::Hovered => Some(Background::Color(pal.surface_alt)),
                            _ => None,
                        },
                        text_color: pal.fg,
                        border: Border { radius: 6.0.into(), ..Default::default() },
                        ..Default::default()
                    })
                    .on_press(Message::PickerNavigate(path)),
            );
        }
        let header = container(crumb_row)
            .padding(Padding::from([10, 14]))
            .width(Length::Fill);

        let mut list = Column::new().spacing(1).padding(Padding::from([6, 8]));
        if let Some(err) = &pk.error {
            list = list.push(text(err.clone()).color(pal.muted).size(13));
        } else if pk.entries.is_empty() {
            list = list
                .push(container(text("Empty folder").color(pal.subtle).size(13)).padding(14));
        } else {
            for (i, e) in pk.entries.iter().enumerate() {
                let is_sel = i == selected;
                let path_clone = e.path.clone();
                let name = e.name.clone();
                let row = button(
                    irow![
                        icon::glyph(ic::FOLDER, 13.0, pal.subtle),
                        text(name).size(13).color(pal.fg),
                    ]
                    .spacing(10)
                    .align_y(iced::Alignment::Center),
                )
                .padding(Padding::from([7, 12]))
                .width(Length::Fill)
                .height(Length::Fixed(32.0))
                .style(move |_, status| button::Style {
                    background: match (is_sel, status) {
                        (true, _) => Some(Background::Color(pal.surface_alt)),
                        (_, button::Status::Hovered) => Some(Background::Color(pal.surface_alt)),
                        _ => None,
                    },
                    text_color: pal.fg,
                    border: Border { radius: 6.0.into(), ..Default::default() },
                    ..Default::default()
                })
                .on_press(Message::PickerNavigate(path_clone));
                list = list.push(row);
            }
        }
        let body = scrollable(list)
            .id(App::overlay_scroll_id())
            .height(Length::Fill)
            .on_scroll(Message::OverlayScrolled)
            .direction(slim_scroll_direction())
            .style(move |_, status| sleek_scrollable_style(status, pal));

        column![header, body].into()
    } else {
        text("No picker").into()
    };

    overlay_frame(panel, pal, 640.0, 560.0)
}

fn file_finder_overlay<'a>(
    query: &'a str,
    files: Vec<(PathBuf, String, i32)>,
    selected: usize,
    pal: Palette,
) -> Element<'a, Message> {
    let input = container(
        text_input("Find file… (fuzzy)", query)
            .on_input(Message::OverlayQueryChanged)
            .on_submit(Message::OverlayConfirm)
            .padding(Padding::from([10, 14]))
            .size(14)
            .style(move |_, _| iced::widget::text_input::Style {
                background: Color::TRANSPARENT.into(),
                border: Border::default(),
                icon: pal.muted,
                placeholder: pal.subtle,
                value: pal.fg,
                selection: pal.selection,
            }),
    )
    .style(move |_| container::Style {
        background: Some(pal.surface.into()),
        border: Border {
            color: pal.rule,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    });

    let mut list = Column::new().spacing(0).padding(Padding::from([6, 8]));
    if files.is_empty() {
        list = list.push(
            container(text("No matches").color(pal.subtle).size(13)).padding(14),
        );
    } else {
        for (i, (p, rel, _)) in files.into_iter().enumerate().take(80) {
            let is_sel = i == selected;
            let path_clone = p.clone();
            let parent = std::path::Path::new(&rel)
                .parent()
                .map(|x| x.to_string_lossy().into_owned())
                .unwrap_or_default();
            let name = std::path::Path::new(&rel)
                .file_name()
                .map(|x| x.to_string_lossy().into_owned())
                .unwrap_or_else(|| rel.clone());
            let inner = irow![
                text(name).size(13).color(pal.fg),
                Space::with_width(8),
                text(parent).size(12).color(pal.subtle),
            ]
            .align_y(iced::Alignment::Center);
            let row = button(inner)
                .padding(Padding::from([7, 12]))
                .width(Length::Fill)
                .height(Length::Fixed(32.0))
                .style(move |_, status| button::Style {
                    background: match (is_sel, status) {
                        (true, _) => Some(Background::Color(pal.surface_alt)),
                        (_, button::Status::Hovered) => Some(Background::Color(pal.surface_alt)),
                        _ => None,
                    },
                    text_color: pal.fg,
                    border: Border { radius: 6.0.into(), ..Default::default() },
                    ..Default::default()
                })
                .on_press(Message::Open(path_clone));
            list = list.push(row);
        }
    }
    let body = scrollable(list)
        .height(Length::Fill)
        .direction(slim_scroll_direction())
            .style(move |_, status| sleek_scrollable_style(status, pal));

    let divider = container(Space::with_height(1.0))
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(pal.rule.into()),
            ..Default::default()
        });

    overlay_frame(column![input, divider, body].into(), pal, 600.0, 460.0)
}

fn command_overlay<'a>(
    query: &'a str,
    cmds: Vec<(&'static str, Message, i32)>,
    selected: usize,
    pal: Palette,
) -> Element<'a, Message> {
    let input = container(
        text_input("Run a command…", query)
            .on_input(Message::OverlayQueryChanged)
            .on_submit(Message::OverlayConfirm)
            .padding(Padding::from([10, 14]))
            .size(14)
            .style(move |_, _| iced::widget::text_input::Style {
                background: Color::TRANSPARENT.into(),
                border: Border::default(),
                icon: pal.muted,
                placeholder: pal.subtle,
                value: pal.fg,
                selection: pal.selection,
            }),
    );

    let mut list = Column::new().spacing(0).padding(Padding::from([6, 8]));
    if cmds.is_empty() {
        list = list
            .push(container(text("No commands").color(pal.subtle).size(13)).padding(14));
    } else {
        for (i, (label, msg, _)) in cmds.into_iter().enumerate() {
            let is_sel = i == selected;
            let row = button(text(label).size(13).color(pal.fg))
                .padding(Padding::from([7, 12]))
                .width(Length::Fill)
                .height(Length::Fixed(32.0))
                .style(move |_, status| button::Style {
                    background: match (is_sel, status) {
                        (true, _) => Some(Background::Color(pal.surface_alt)),
                        (_, button::Status::Hovered) => Some(Background::Color(pal.surface_alt)),
                        _ => None,
                    },
                    text_color: pal.fg,
                    border: Border { radius: 6.0.into(), ..Default::default() },
                    ..Default::default()
                })
                .on_press(msg);
            list = list.push(row);
        }
    }

    let body = scrollable(list)
        .height(Length::Fill)
        .direction(slim_scroll_direction())
            .style(move |_, status| sleek_scrollable_style(status, pal));

    let divider = container(Space::with_height(1.0))
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(pal.rule.into()),
            ..Default::default()
        });

    overlay_frame(column![input, divider, body].into(), pal, 560.0, 420.0)
}

fn theme_overlay<'a>(
    query: &'a str,
    themes: Vec<ThemePreset>,
    selected: usize,
    current: ThemePreset,
    pal: Palette,
) -> Element<'a, Message> {
    let input = container(
        text_input("Pick theme…", query)
            .on_input(Message::OverlayQueryChanged)
            .on_submit(Message::OverlayConfirm)
            .padding(Padding::from([10, 14]))
            .size(14)
            .style(move |_, _| iced::widget::text_input::Style {
                background: Color::TRANSPARENT.into(),
                border: Border::default(),
                icon: pal.muted,
                placeholder: pal.subtle,
                value: pal.fg,
                selection: pal.selection,
            }),
    );

    let mut list = Column::new().spacing(0).padding(Padding::from([6, 8]));
    for (i, t) in themes.into_iter().enumerate() {
        let is_sel = i == selected;
        let is_current = t == current;
        let swatch_pal = theme::palette_for(t);
        let swatch = container(Space::new(Length::Fixed(14.0), Length::Fixed(14.0)))
            .style(move |_| container::Style {
                background: Some(swatch_pal.accent.into()),
                border: Border {
                    color: swatch_pal.rule,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });
        let bg_swatch = container(Space::new(Length::Fixed(14.0), Length::Fixed(14.0)))
            .style(move |_| container::Style {
                background: Some(swatch_pal.bg.into()),
                border: Border {
                    color: swatch_pal.rule,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });
        let label = t.label();
        let marker: Element<'a, Message> = if is_current {
            icon::glyph(ic::CHECK, 12.0, pal.accent).into()
        } else {
            Space::with_width(12.0).into()
        };
        let row = button(
            irow![
                marker,
                Space::with_width(4),
                bg_swatch,
                Space::with_width(2),
                swatch,
                Space::with_width(8),
                text(label).size(13).color(pal.fg),
            ]
            .align_y(iced::Alignment::Center),
        )
        .padding(Padding::from([7, 12]))
        .width(Length::Fill)
        .style(move |_, status| button::Style {
            background: match (is_sel, status) {
                (true, _) => Some(Background::Color(pal.surface_alt)),
                (_, button::Status::Hovered) => Some(Background::Color(pal.surface_alt)),
                _ => None,
            },
            text_color: pal.fg,
            border: Border { radius: 6.0.into(), ..Default::default() },
            ..Default::default()
        })
        .on_press(Message::SetTheme(t));
        list = list.push(row);
    }

    let body = scrollable(list)
        .height(Length::Fill)
        .direction(slim_scroll_direction())
            .style(move |_, status| sleek_scrollable_style(status, pal));

    let divider = container(Space::with_height(1.0))
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(pal.rule.into()),
            ..Default::default()
        });

    overlay_frame(column![input, divider, body].into(), pal, 480.0, 420.0)
}

fn overlay_frame<'a>(
    content: Element<'a, Message>,
    pal: Palette,
    max_w: f32,
    max_h: f32,
) -> Element<'a, Message> {
    let panel = container(content)
        .max_width(max_w)
        .max_height(max_h)
        .width(Length::Fill)
        .height(Length::Fill)
        .clip(true)
        .style(move |_| container::Style {
            background: Some(pal.surface.into()),
            border: Border {
                color: pal.rule,
                width: 1.0,
                radius: 14.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.28),
                offset: iced::Vector::new(0.0, 14.0),
                blur_radius: 50.0,
            },
            ..Default::default()
        });

    let scrim = mouse_area(
        container(Space::new(Length::Fill, Length::Fill))
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.18))),
                ..Default::default()
            })
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(Message::CloseOverlay);

    let centered = container(panel)
        .padding(Padding::from([80, 40]))
        .center_x(Length::Fill)
        .align_y(iced::alignment::Vertical::Top);

    iced::widget::stack![scrim, centered].into()
}

fn slim_scroll_direction() -> scrollable::Direction {
    scrollable::Direction::Vertical(
        scrollable::Scrollbar::new()
            .width(6.0)
            .scroller_width(6.0)
            .margin(2.0),
    )
}

fn sleek_scrollable_style(
    status: scrollable::Status,
    pal: Palette,
) -> scrollable::Style {
    let visible = matches!(
        status,
        scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. }
    );
    let scroller_color = if visible {
        pal.scroller_hover
    } else {
        Color::TRANSPARENT
    };
    let rail = scrollable::Rail {
        background: None,
        border: Border { radius: 8.0.into(), ..Default::default() },
        scroller: scrollable::Scroller {
            color: scroller_color,
            border: Border { radius: 8.0.into(), ..Default::default() },
        },
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
    }
}

async fn load_file(p: PathBuf) -> Result<(PathBuf, String), String> {
    let bytes = tokio::fs::read(&p).await.map_err(|e| e.to_string())?;
    let s = String::from_utf8_lossy(&bytes).into_owned();
    Ok((p, s))
}
