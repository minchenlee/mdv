//! Diagram rendering core — Mermaid + Graphviz/DOT pipelines.
//!
//! Pure-Rust, blocking renderer plus an LRU cache. Higher layers (T3 render path,
//! T4 app integration) wrap the blocking call in `iced::Task::perform` and feed
//! the resulting SVG into `iced::widget::svg::Handle`.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;

pub use crate::ast::DiagramKind;
use crate::theme::Palette;

use iced::widget::svg;
use iced::Color;

/// Maximum accepted source size (bytes). Anything larger is rejected before
/// the parser/renderer runs.
pub const MAX_SOURCE_BYTES: usize = 64 * 1024;

/// Maximum accepted rendered SVG size (bytes).
pub const MAX_SVG_BYTES: usize = 4 * 1024 * 1024;

/// Default LRU cache capacity.
pub const DEFAULT_CACHE_CAP: usize = 64;

/// State of a diagram in the cache.
#[derive(Debug, Clone)]
pub enum DiagramState {
    /// A render task is in flight.
    Pending,
    /// Render completed successfully. `source_bytes` is the SVG payload.
    Ready {
        handle: svg::Handle,
        source_bytes: Arc<Vec<u8>>,
    },
    /// Render failed — held so we don't retry on every redraw.
    Err(String),
}

/// LRU cache of rendered diagrams, keyed by `(content_hash, theme_id)`.
#[derive(Debug)]
pub struct DiagramCache {
    inner: lru::LruCache<(u64, u32), DiagramState>,
}

impl DiagramCache {
    pub fn new(cap: usize) -> Self {
        let cap = NonZeroUsize::new(cap.max(1)).expect("cap >= 1");
        Self {
            inner: lru::LruCache::new(cap),
        }
    }

    pub fn get(&mut self, key: &(u64, u32)) -> Option<&DiagramState> {
        self.inner.get(key)
    }

    pub fn put(&mut self, key: (u64, u32), value: DiagramState) {
        self.inner.put(key, value);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for DiagramCache {
    fn default() -> Self {
        Self::new(DEFAULT_CACHE_CAP)
    }
}

/// Stable `u32` digest of a palette. Used as a cache-bust key so theme changes
/// invalidate cached SVGs without explicit pruning.
pub fn theme_id(palette: &Palette) -> u32 {
    let mut h = DefaultHasher::new();
    // Hash every color-bearing field. We only need monotonic determinism — the
    // exact mixing isn't load-bearing.
    for c in [
        palette.bg,
        palette.surface,
        palette.surface_alt,
        palette.sidebar,
        palette.fg,
        palette.muted,
        palette.subtle,
        palette.accent,
        palette.accent_fg,
        palette.code_bg,
        palette.code_border,
        palette.rule,
        palette.selection,
        palette.match_bg,
        palette.match_current_bg,
        palette.scroller,
        palette.scroller_hover,
        palette.indent_guide,
        palette.tree_selected_bg,
        palette.tree_selected_border,
    ] {
        hash_color(&c, &mut h);
    }
    let full = h.finish();
    (full ^ (full >> 32)) as u32
}

fn hash_color<H: Hasher>(c: &Color, h: &mut H) {
    // Color isn't Hash; fold bytes manually.
    c.r.to_bits().hash(h);
    c.g.to_bits().hash(h);
    c.b.to_bits().hash(h);
    c.a.to_bits().hash(h);
}

fn color_to_hex(c: Color) -> String {
    let r = (c.r.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (c.g.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (c.b.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Build the mermaid `%%{init}%%` directive that carries our palette.
///
/// Spec maps:
///   background         <- palette.bg
///   primaryColor       <- palette.surface
///   primaryTextColor   <- palette.fg          (Palette has `fg`, not `text`)
///   primaryBorderColor <- palette.code_border (closest analogue to `border`)
///   lineColor          <- palette.muted
///   secondaryColor     <- palette.surface_alt
///   tertiaryColor      <- palette.accent
///   fontFamily         <- caller-supplied font family
fn mermaid_init_block(palette: &Palette, font_family: &str) -> String {
    format!(
        "%%{{init: {{ 'theme': 'base', 'themeVariables': {{ \
         'background': '{bg}', \
         'primaryColor': '{primary}', \
         'primaryTextColor': '{text}', \
         'primaryBorderColor': '{border}', \
         'lineColor': '{line}', \
         'secondaryColor': '{secondary}', \
         'tertiaryColor': '{tertiary}', \
         'fontFamily': '{font}' \
         }} }}}}%%\n",
        bg = color_to_hex(palette.bg),
        primary = color_to_hex(palette.surface),
        text = color_to_hex(palette.fg),
        border = color_to_hex(palette.code_border),
        line = color_to_hex(palette.muted),
        secondary = color_to_hex(palette.surface_alt),
        tertiary = color_to_hex(palette.accent),
        font = font_family,
    )
}

/// True if `source` already opens with a `%%{init` directive — caller wins.
pub(crate) fn has_user_init(source: &str) -> bool {
    source.trim_start().starts_with("%%{init")
}

/// Build the DOT preamble that injects our palette as default graph/node/edge
/// attributes. Inserted immediately after the user's opening `{`.
fn dot_preamble(palette: &Palette, font_family: &str) -> String {
    format!(
        "  graph [bgcolor=\"transparent\" fontname=\"{font}\"];\n  \
         node  [fontcolor=\"{text}\" color=\"{border}\" fillcolor=\"{fill}\" fontname=\"{font}\"];\n  \
         edge  [color=\"{edge}\" fontname=\"{font}\"];\n",
        font = font_family,
        text = color_to_hex(palette.fg),
        border = color_to_hex(palette.code_border),
        fill = color_to_hex(palette.surface),
        edge = color_to_hex(palette.muted),
    )
}

/// Insert `preamble` immediately after the first `{` in `source`. If no `{`
/// is found, returns `source` unchanged (the parser will error out anyway,
/// and we don't want to mangle malformed input).
fn inject_dot_preamble(source: &str, preamble: &str) -> String {
    if let Some(idx) = source.find('{') {
        let (head, tail) = source.split_at(idx + 1);
        let mut out = String::with_capacity(source.len() + preamble.len() + 1);
        out.push_str(head);
        out.push('\n');
        out.push_str(preamble);
        out.push_str(tail);
        out
    } else {
        source.to_string()
    }
}

/// Blocking renderer. Wraps the inner work in `catch_unwind` so a panic in a
/// third-party crate doesn't take down the UI thread.
pub fn render_blocking(
    kind: DiagramKind,
    source: &str,
    palette: &Palette,
    font_family: &str,
) -> Result<String, String> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err("diagram too large".to_string());
    }

    // Clone inputs we need inside the unwind boundary.
    let source = source.to_string();
    let palette = *palette;
    let font_family = font_family.to_string();

    let result = std::panic::catch_unwind(move || -> Result<String, String> {
        match kind {
            DiagramKind::Mermaid => render_mermaid(&source, &palette, &font_family),
            DiagramKind::Dot => render_dot(&source, &palette, &font_family),
        }
    });

    let svg = match result {
        Ok(Ok(svg)) => svg,
        Ok(Err(msg)) => return Err(msg),
        Err(_) => return Err("internal renderer error".to_string()),
    };

    if svg.len() > MAX_SVG_BYTES {
        return Err("rendered diagram too large".to_string());
    }

    Ok(svg)
}

fn render_mermaid(source: &str, palette: &Palette, font_family: &str) -> Result<String, String> {
    let prepared = if has_user_init(source) {
        source.to_string()
    } else {
        let mut s = mermaid_init_block(palette, font_family);
        s.push_str(source);
        s
    };

    mermaid_rs_renderer::render_with_options(
        &prepared,
        mermaid_rs_renderer::RenderOptions::default(),
    )
    .map_err(|e| e.to_string())
}

fn render_dot(source: &str, palette: &Palette, font_family: &str) -> Result<String, String> {
    let preamble = dot_preamble(palette, font_family);
    let prepared = inject_dot_preamble(source, &preamble);

    use layout::backends::svg::SVGWriter;
    use layout::gv::{DotParser, GraphBuilder};

    let mut parser = DotParser::new(&prepared);
    let graph = parser.process().map_err(|e| e.to_string())?;

    let mut builder = GraphBuilder::new();
    builder.visit_graph(&graph);
    let mut vg = builder.get();

    let mut svg = SVGWriter::new();
    vg.do_it(false, false, false, &mut svg);
    Ok(svg.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Palette;

    fn palette() -> Palette {
        Palette::ONE_DARK
    }

    #[test]
    fn init_skipped_when_user_provides_it() {
        assert!(has_user_init("%%{init: { 'theme': 'dark' }}%%\ngraph LR\nA-->B"));
        assert!(!has_user_init("graph LR\nA-->B"));
    }

    #[test]
    fn dot_preamble_inserted_after_brace() {
        let injected = inject_dot_preamble("digraph G { a -> b }", "PREAMBLE\n");
        assert!(injected.starts_with("digraph G {"));
        assert!(injected.contains("PREAMBLE"));
        // Preamble appears before the edge.
        let pre = injected.find("PREAMBLE").unwrap();
        let edge = injected.find("a -> b").unwrap();
        assert!(pre < edge);
    }

    #[test]
    fn theme_id_changes_with_palette() {
        let a = theme_id(&Palette::ONE_DARK);
        let b = theme_id(&Palette::ONE_LIGHT);
        assert_ne!(a, b);
    }

    #[test]
    fn cache_basic() {
        let mut cache = DiagramCache::new(2);
        cache.put((1, 0), DiagramState::Pending);
        cache.put((2, 0), DiagramState::Err("boom".into()));
        assert_eq!(cache.len(), 2);
        assert!(matches!(cache.get(&(1, 0)), Some(DiagramState::Pending)));
    }

    #[test]
    fn oversized_source_rejected() {
        let big = "x".repeat(MAX_SOURCE_BYTES + 1);
        let err =
            render_blocking(DiagramKind::Mermaid, &big, &palette(), "system-ui").unwrap_err();
        assert!(err.contains("too large"), "got: {err}");
    }
}
