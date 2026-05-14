use crate::app::Message;
use crate::ast::{Block, BlockId, Inline, ListItem};
use crate::theme::{Palette, Typography};
use iced::widget::{container, image as image_widget, rich_text, row, span, text, Column, Space};
use iced::{Element, Length, Padding};

#[derive(Clone, Default)]
pub struct Highlight {
    pub query: String,
    pub current_block: Option<usize>,
    pub current_in_block: usize,
}

pub fn render<'a>(
    blocks: &'a [(BlockId, Block)],
    pal: &Palette,
    typ: &Typography,
    hl: &Highlight,
    viewport: Option<&iced::widget::scrollable::Viewport>,
    cache: &crate::virt::HeightCache,
) -> Element<'a, Message> {
    // Virt scroll disabled: rebuilding the visible-window Element tree on
    // every scroll event causes per-frame rich_text reflow jank in Iced 0.13.
    // Full render lets Iced's scrollable handle scrolling internally without
    // re-emitting the body tree per delta.
    let _ = (viewport, cache);
    let mut col = Column::new().spacing(14);
    for (idx, (_id, b)) in blocks.iter().enumerate() {
        let local = if hl.current_block == Some(idx) {
            Some(hl.current_in_block)
        } else {
            None
        };
        col = col.push(render_block(b, pal, typ, &hl.query, local));
    }

    // Reading column cap: 780px (mdv design system READING_MAX, render.rs).
    let _ = typ.measure_ch;
    container(col).max_width(780.0).into()
}

fn render_block<'a>(
    b: &'a Block,
    pal: &Palette,
    typ: &Typography,
    query: &str,
    current_in_block: Option<usize>,
) -> Element<'a, Message> {
    let mut ctx = HlCtx { query, counter: 0, current_in_block, pal: *pal };
    match b {
        Block::Heading { level, inlines, .. } => {
            let size = match level {
                1 => typ.h1_size,
                2 => typ.h2_size,
                3 => typ.h3_size,
                4 => typ.h4_size,
                5 => typ.h5_size,
                _ => typ.h6_size,
            };
            let spans = inline_spans(inlines, pal, size, &mut ctx);
            rich_text(spans).into()
        }
        Block::Paragraph(inlines) => {
            let spans = inline_spans(inlines, pal, typ.body_size, &mut ctx);
            rich_text(spans).into()
        }
        Block::CodeBlock { code, spans, .. } => {
            let pal_c = *pal;
            let mut out: Vec<RtSpan<'a>> = Vec::new();
            let mut cursor = 0usize;
            for s in spans {
                if s.range.start < cursor || s.range.end > code.len() {
                    continue;
                }
                if s.range.start > cursor {
                    let slice = &code[cursor..s.range.start];
                    push_code_with_hl(slice, pal_c.fg, pal, typ.code_size, &mut out, &mut ctx);
                }
                let color = style_color(s.style, pal);
                let slice = &code[s.range.start..s.range.end];
                push_code_with_hl(slice, color, pal, typ.code_size, &mut out, &mut ctx);
                cursor = s.range.end;
            }
            if cursor < code.len() {
                push_code_with_hl(&code[cursor..], pal_c.fg, pal, typ.code_size, &mut out, &mut ctx);
            }
            container(rich_text(out))
                .padding(Padding::from(14))
                .style(move |_| container::Style {
                    background: Some(pal_c.code_bg.into()),
                    border: iced::Border {
                        color: pal_c.code_border,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                })
                .width(Length::Fill)
                .into()
        }
        Block::Blockquote(blocks) => {
            let inner = blocks
                .iter()
                .fold(Column::new().spacing(8), |c, b| {
                    c.push(render_block(b, pal, typ, query, current_in_block))
                });
            let pal_q = *pal;
            container(inner)
                .padding(Padding { top: 2.0, right: 14.0, bottom: 2.0, left: 17.0 })
                .width(Length::Fill)
                .style(move |_| container::Style {
                    border: iced::Border {
                        color: pal_q.accent,
                        width: 0.0,
                        radius: 0.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: pal_q.accent,
                        offset: iced::Vector::new(-1.5, 0.0),
                        blur_radius: 0.0,
                    },
                    ..Default::default()
                })
                .into()
        }
        Block::List { ordered, items } => render_list(*ordered, items, pal, typ, &mut ctx),
        Block::Table { headers, rows } => render_table(headers, rows, pal, typ, &mut ctx),
        Block::Image { url, alt } => {
            if url.starts_with("http://") || url.starts_with("https://") {
                text(format!("[image: {alt} ({url})]"))
                    .color(pal.muted)
                    .into()
            } else {
                image_widget(url).into()
            }
        }
        Block::Rule => {
            let pal_r = *pal;
            container(Space::new().height(1.0))
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(pal_r.rule.into()),
                    ..Default::default()
                })
                .into()
        }
    }
}

type RtSpan<'a> = iced::advanced::text::Span<'a, Message, iced::Font>;

struct HlCtx<'a> {
    query: &'a str,
    counter: usize,
    current_in_block: Option<usize>,
    pal: Palette,
}

fn inline_spans<'a>(
    inlines: &'a [Inline],
    pal: &Palette,
    size: f32,
    ctx: &mut HlCtx<'_>,
) -> Vec<RtSpan<'a>> {
    let mut out = Vec::new();
    for i in inlines {
        push_span(i, &mut out, pal, size, Style::default(), ctx);
    }
    out
}

#[derive(Clone, Default)]
struct Style {
    italic: bool,
    bold: bool,
    strike: bool,
    link: Option<String>,
}

fn make_span<'a>(
    text_str: &'a str,
    pal: &Palette,
    size: f32,
    st: &Style,
    monospace: bool,
    bg: Option<iced::Color>,
) -> RtSpan<'a> {
    let mut font = if monospace {
        iced::Font::MONOSPACE
    } else {
        iced::Font::with_name("Inter")
    };
    if st.italic {
        font.style = iced::font::Style::Italic;
    }
    if st.bold {
        font.weight = iced::font::Weight::Bold;
    }
    let mut s = span(text_str).size(size).font(font);
    if st.strike {
        s = s.strikethrough(true);
    }
    if let Some(c) = bg {
        s = s.background(c);
    } else if monospace {
        s = s.background(pal.code_bg);
    }
    if let Some(url) = &st.link {
        s = s
            .color(pal.accent)
            .underline(true)
            .link(Message::OpenLink(url.clone()));
    } else {
        s = s.color(pal.fg);
    }
    s
}

fn push_text_with_hl<'a>(
    text_str: &'a str,
    pal: &Palette,
    size: f32,
    st: &Style,
    monospace: bool,
    out: &mut Vec<RtSpan<'a>>,
    ctx: &mut HlCtx<'_>,
) {
    if ctx.query.is_empty() {
        out.push(make_span(text_str, pal, size, st, monospace, None));
        return;
    }
    let lower_text = text_str.to_lowercase();
    let lower_q = ctx.query.to_lowercase();
    let mut cursor = 0usize;
    while let Some(rel) = lower_text[cursor..].find(&lower_q) {
        let abs = cursor + rel;
        if abs > cursor {
            out.push(make_span(&text_str[cursor..abs], pal, size, st, monospace, None));
        }
        let end = abs + lower_q.len();
        let is_current = ctx.current_in_block == Some(ctx.counter);
        let bg = if is_current { ctx.pal.match_current_bg } else { ctx.pal.match_bg };
        out.push(make_span(&text_str[abs..end], pal, size, st, monospace, Some(bg)));
        ctx.counter += 1;
        cursor = end;
    }
    if cursor < text_str.len() {
        out.push(make_span(&text_str[cursor..], pal, size, st, monospace, None));
    }
}

fn push_code_with_hl<'a>(
    text_str: &'a str,
    color: iced::Color,
    pal: &Palette,
    size: f32,
    out: &mut Vec<RtSpan<'a>>,
    ctx: &mut HlCtx<'_>,
) {
    if ctx.query.is_empty() {
        out.push(span(text_str).font(iced::Font::MONOSPACE).size(size).color(color));
        return;
    }
    let lower_text = text_str.to_lowercase();
    let lower_q = ctx.query.to_lowercase();
    let mut cursor = 0usize;
    while let Some(rel) = lower_text[cursor..].find(&lower_q) {
        let abs = cursor + rel;
        if abs > cursor {
            out.push(span(&text_str[cursor..abs]).font(iced::Font::MONOSPACE).size(size).color(color));
        }
        let end = abs + lower_q.len();
        let is_current = ctx.current_in_block == Some(ctx.counter);
        let bg = if is_current { ctx.pal.match_current_bg } else { ctx.pal.match_bg };
        out.push(
            span(&text_str[abs..end])
                .font(iced::Font::MONOSPACE)
                .size(size)
                .color(color)
                .background(bg),
        );
        ctx.counter += 1;
        cursor = end;
    }
    if cursor < text_str.len() {
        out.push(span(&text_str[cursor..]).font(iced::Font::MONOSPACE).size(size).color(color));
    }
    let _ = pal;
}

fn push_span<'a>(
    i: &'a Inline,
    out: &mut Vec<RtSpan<'a>>,
    pal: &Palette,
    size: f32,
    st: Style,
    ctx: &mut HlCtx<'_>,
) {
    match i {
        Inline::Text(t) => push_text_with_hl(t.as_str(), pal, size, &st, false, out, ctx),
        Inline::Code(t) => push_text_with_hl(t.as_str(), pal, size, &st, true, out, ctx),
        Inline::Emph(c) => {
            for x in c {
                let mut child = st.clone();
                child.italic = true;
                push_span(x, out, pal, size, child, ctx);
            }
        }
        Inline::Strong(c) => {
            for x in c {
                let mut child = st.clone();
                child.bold = true;
                push_span(x, out, pal, size, child, ctx);
            }
        }
        Inline::Strike(c) => {
            for x in c {
                let mut child = st.clone();
                child.strike = true;
                push_span(x, out, pal, size, child, ctx);
            }
        }
        Inline::Link { url, children } => {
            for x in children {
                let mut child = st.clone();
                child.link = Some(url.clone());
                push_span(x, out, pal, size, child, ctx);
            }
        }
    }
}

fn render_list<'a>(
    ordered: bool,
    items: &'a [ListItem],
    pal: &Palette,
    typ: &Typography,
    ctx: &mut HlCtx<'_>,
) -> Element<'a, Message> {
    let mut col = Column::new().spacing(8);
    for (idx, it) in items.iter().enumerate() {
        let bullet = match (ordered, it.task) {
            (_, Some(true)) => "✓".to_string(),
            (_, Some(false)) => "○".to_string(),
            (true, _) => format!("{}.", idx + 1),
            (false, _) => "•".to_string(),
        };
        let inner = it.blocks.iter().fold(Column::new().spacing(6), |c, b| {
            c.push(render_block_inner(b, pal, typ, ctx))
        });
        col = col.push(
            row![
                container(text(bullet).color(pal.accent).size(typ.body_size))
                    .width(Length::Fixed(28.0)),
                inner
            ]
            .spacing(6),
        );
    }
    col.into()
}

fn render_block_inner<'a>(
    b: &'a Block,
    pal: &Palette,
    typ: &Typography,
    ctx: &mut HlCtx<'_>,
) -> Element<'a, Message> {
    match b {
        Block::Paragraph(inlines) => {
            let spans = inline_spans(inlines, pal, typ.body_size, ctx);
            rich_text(spans).into()
        }
        _ => render_block(b, pal, typ, ctx.query, ctx.current_in_block),
    }
}

fn render_table<'a>(
    headers: &'a [Vec<Inline>],
    rows: &'a [Vec<Vec<Inline>>],
    pal: &Palette,
    typ: &Typography,
    ctx: &mut HlCtx<'_>,
) -> Element<'a, Message> {
    let cols = headers
        .len()
        .max(rows.iter().map(|r| r.len()).max().unwrap_or(0))
        .max(1);
    let pal_t = *pal;

    let make_cell = |content: Element<'a, Message>, is_header: bool| -> Element<'a, Message> {
        container(content)
            .padding(Padding::from([8, 12]))
            .width(Length::FillPortion(1))
            .style(move |_| container::Style {
                background: if is_header {
                    Some(pal_t.surface_alt.into())
                } else {
                    None
                },
                ..Default::default()
            })
            .into()
    };

    let mut header_row = iced::widget::Row::new().spacing(0);
    for i in 0..cols {
        let content: Element<'a, Message> = if let Some(cell) = headers.get(i) {
            let spans = inline_spans(cell, pal, typ.body_size, ctx);
            rich_text(spans).into()
        } else {
            text("").into()
        };
        header_row = header_row.push(make_cell(content, true));
    }

    let mut grid = Column::new().spacing(0);
    grid = grid.push(
        container(header_row)
            .style(move |_| container::Style {
                border: iced::Border {
                    color: pal_t.code_border,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .width(Length::Fill),
    );

    for row_cells in rows {
        let mut r = iced::widget::Row::new().spacing(0);
        for i in 0..cols {
            let content: Element<'a, Message> = if let Some(cell) = row_cells.get(i) {
                let spans = inline_spans(cell, pal, typ.body_size, ctx);
                rich_text(spans).into()
            } else {
                text("").into()
            };
            r = r.push(make_cell(content, false));
        }
        grid = grid.push(
            container(r)
                .style(move |_| container::Style {
                    border: iced::Border {
                        color: pal_t.code_border,
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                })
                .width(Length::Fill),
        );
    }

    container(grid)
        .width(Length::Fill)
        .style(move |_| container::Style {
            border: iced::Border {
                color: pal_t.code_border,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn style_color(s: crate::ast::HlStyle, pal: &Palette) -> iced::Color {
    use crate::ast::HlStyle::*;
    match s {
        // Universal hl palette (mdv design system: --hl-*)
        Keyword => iced::Color::from_rgb(199.0 / 255.0, 92.0 / 255.0, 140.0 / 255.0),  // #c75c8c
        Type => iced::Color::from_rgb(92.0 / 255.0, 166.0 / 255.0, 199.0 / 255.0),     // #5ca6c7
        Function => iced::Color::from_rgb(103.0 / 255.0, 140.0 / 255.0, 217.0 / 255.0),// #678cd9
        String => iced::Color::from_rgb(107.0 / 255.0, 166.0 / 255.0, 107.0 / 255.0),  // #6ba66b
        Number => iced::Color::from_rgb(199.0 / 255.0, 140.0 / 255.0, 77.0 / 255.0),   // #c78c4d
        Comment => pal.muted,
        Operator => pal.fg,
        Constant => iced::Color::from_rgb(199.0 / 255.0, 140.0 / 255.0, 77.0 / 255.0), // #c78c4d
        Variable => pal.fg,
        Punctuation => pal.muted,
        Plain => pal.fg,
    }
}
