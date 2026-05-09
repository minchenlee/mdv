use crate::app::Message;
use crate::ast::{Block, Inline, ListItem};
use crate::theme::{Palette, Typography};
use iced::widget::{container, image as image_widget, rich_text, row, span, text, Column, Space};
use iced::{Color, Element, Length, Padding};

pub fn render<'a>(blocks: &'a [Block], pal: &Palette, typ: &Typography) -> Element<'a, Message> {
    let mut col = Column::new().spacing(12);
    for b in blocks {
        col = col.push(render_block(b, pal, typ));
    }
    container(col)
        .max_width((typ.measure_ch as f32) * (typ.body_size * 0.55))
        .into()
}

fn render_block<'a>(b: &'a Block, pal: &Palette, typ: &Typography) -> Element<'a, Message> {
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
            let spans = inline_spans(inlines, pal, size);
            rich_text(spans).into()
        }
        Block::Paragraph(inlines) => {
            let spans = inline_spans(inlines, pal, typ.body_size);
            rich_text(spans).into()
        }
        Block::CodeBlock { code, .. } => {
            let pal_c = *pal;
            container(
                text(code.as_str())
                    .size(typ.code_size)
                    .font(iced::Font::MONOSPACE)
                    .color(pal_c.fg),
            )
            .padding(Padding::from(12))
            .style(move |_| container::Style {
                background: Some(pal_c.code_bg.into()),
                border: iced::Border {
                    color: pal_c.code_border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .width(Length::Fill)
            .into()
        }
        Block::Blockquote(blocks) => {
            let inner = blocks
                .iter()
                .fold(Column::new().spacing(8), |c, b| c.push(render_block(b, pal, typ)));
            let pal_q = *pal;
            row![
                container(Space::with_width(3.0))
                    .height(Length::Fill)
                    .style(move |_| container::Style {
                        background: Some(pal_q.muted.into()),
                        ..Default::default()
                    }),
                container(inner).padding(Padding::from([0, 12]))
            ]
            .spacing(8)
            .into()
        }
        Block::List { ordered, items } => render_list(*ordered, items, pal, typ),
        Block::Table { headers, rows } => render_table(headers, rows, pal, typ),
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
            container(Space::with_height(1.0))
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

fn inline_spans<'a>(inlines: &'a [Inline], pal: &Palette, size: f32) -> Vec<RtSpan<'a>> {
    let mut out = Vec::new();
    for i in inlines {
        push_span(i, &mut out, pal, size, Style::default());
    }
    out
}

#[derive(Clone, Copy, Default)]
struct Style {
    italic: bool,
    bold: bool,
    strike: bool,
    code: bool,
    link: bool,
    color: Option<Color>,
}

fn push_span<'a>(
    i: &'a Inline,
    out: &mut Vec<RtSpan<'a>>,
    pal: &Palette,
    size: f32,
    st: Style,
) {
    match i {
        Inline::Text(t) => {
            let mut s = span(t.as_str()).size(size);
            if st.italic {
                s = s.font(iced::Font {
                    style: iced::font::Style::Italic,
                    ..iced::Font::DEFAULT
                });
            }
            if st.bold {
                s = s.font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..iced::Font::DEFAULT
                });
            }
            if st.strike {
                s = s.strikethrough(true);
            }
            if st.code {
                s = s.font(iced::Font::MONOSPACE).background(pal.code_bg);
            }
            if st.link {
                s = s.color(pal.accent).underline(true);
            } else if let Some(c) = st.color {
                s = s.color(c);
            } else {
                s = s.color(pal.fg);
            }
            out.push(s);
        }
        Inline::Code(t) => {
            let mut child = st;
            child.code = true;
            // Use static text by leaking? No — use Inline::Text path with original string borrow.
            // We need 'a lifetime; clone and convert via owned String requires leak. Instead build span directly.
            let mut s = span(t.as_str()).size(size).font(iced::Font::MONOSPACE).background(pal.code_bg);
            if child.strike { s = s.strikethrough(true); }
            if child.link { s = s.color(pal.accent).underline(true); } else { s = s.color(pal.fg); }
            out.push(s);
        }
        Inline::Emph(c) => {
            for x in c {
                let mut child = st;
                child.italic = true;
                push_span(x, out, pal, size, child);
            }
        }
        Inline::Strong(c) => {
            for x in c {
                let mut child = st;
                child.bold = true;
                push_span(x, out, pal, size, child);
            }
        }
        Inline::Strike(c) => {
            for x in c {
                let mut child = st;
                child.strike = true;
                push_span(x, out, pal, size, child);
            }
        }
        Inline::Link { children, .. } => {
            for x in children {
                let mut child = st;
                child.link = true;
                push_span(x, out, pal, size, child);
            }
        }
    }
}

fn render_list<'a>(
    ordered: bool,
    items: &'a [ListItem],
    pal: &Palette,
    typ: &Typography,
) -> Element<'a, Message> {
    let mut col = Column::new().spacing(6);
    for (idx, it) in items.iter().enumerate() {
        let bullet = match (ordered, it.task) {
            (_, Some(true)) => "[x]".to_string(),
            (_, Some(false)) => "[ ]".to_string(),
            (true, _) => format!("{}.", idx + 1),
            (false, _) => "•".to_string(),
        };
        let inner = it
            .blocks
            .iter()
            .fold(Column::new().spacing(6), |c, b| c.push(render_block(b, pal, typ)));
        col = col.push(
            row![
                container(text(bullet).color(pal.muted).size(typ.body_size))
                    .width(Length::Fixed(28.0)),
                inner
            ]
            .spacing(6),
        );
    }
    col.into()
}

fn render_table<'a>(
    headers: &'a [Vec<Inline>],
    rows: &'a [Vec<Vec<Inline>>],
    pal: &Palette,
    typ: &Typography,
) -> Element<'a, Message> {
    let mut grid = Column::new().spacing(0);
    let pal_h = *pal;
    let header_row = headers
        .iter()
        .fold(iced::widget::Row::new().spacing(0), |acc, cell| {
            acc.push(
                container(rich_text(inline_spans(cell, pal, typ.body_size)))
                    .padding(Padding::from(8))
                    .style(move |_| container::Style {
                        background: Some(pal_h.code_bg.into()),
                        border: iced::Border {
                            color: pal_h.code_border,
                            width: 1.0,
                            radius: 0.0.into(),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fill),
            )
        });
    grid = grid.push(header_row);
    for row_cells in rows {
        let pal_r = *pal;
        let r = row_cells
            .iter()
            .fold(iced::widget::Row::new().spacing(0), |acc, cell| {
                acc.push(
                    container(rich_text(inline_spans(cell, pal, typ.body_size)))
                        .padding(Padding::from(8))
                        .style(move |_| container::Style {
                            border: iced::Border {
                                color: pal_r.code_border,
                                width: 1.0,
                                radius: 0.0.into(),
                            },
                            ..Default::default()
                        })
                        .width(Length::Fill),
                )
            });
        grid = grid.push(r);
    }
    grid.into()
}

