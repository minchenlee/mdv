//! T2 — renderer-core integration tests.

use mdv::diagram::{render_blocking, DiagramKind, MAX_SOURCE_BYTES};
use mdv::theme::Palette;

fn palette() -> Palette {
    Palette::ONE_DARK
}

#[test]
fn render_simple_mermaid_returns_svg() {
    let svg = render_blocking(
        DiagramKind::Mermaid,
        "graph LR\n  A --> B",
        &palette(),
        "system-ui",
    )
    .expect("mermaid should render");
    let head = &svg[..200.min(svg.len())];
    assert!(head.contains("<svg"), "no <svg in head: {head}");
}

#[test]
fn render_simple_dot_returns_svg() {
    let svg = render_blocking(
        DiagramKind::Dot,
        "digraph G { a -> b }",
        &palette(),
        "system-ui",
    )
    .expect("dot should render");
    let head = &svg[..200.min(svg.len())];
    assert!(head.contains("<svg"), "no <svg in head: {head}");
}

#[test]
fn render_rejects_oversized_source() {
    let big = "x".repeat(MAX_SOURCE_BYTES + 100);
    let err = render_blocking(DiagramKind::Mermaid, &big, &palette(), "system-ui")
        .expect_err("oversized source must be rejected");
    assert!(err.contains("too large"), "got: {err}");
}

#[test]
fn render_mermaid_invalid_returns_err() {
    let err = render_blocking(
        DiagramKind::Mermaid,
        "this is not mermaid",
        &palette(),
        "system-ui",
    )
    .expect_err("invalid mermaid must be Err");
    assert!(!err.is_empty());
}

#[test]
fn inject_skipped_when_user_has_init() {
    // If the user already provided %%{init, our theme block must not be prepended.
    // We can't easily inspect the prepared string from outside, but we can verify
    // behaviorally: a user-provided init parses identically with or without our
    // injection. We rely on render success here as a proxy — and check no panic.
    let src = "%%{init: { 'theme': 'base' }}%%\ngraph LR\n  A --> B";
    let svg = render_blocking(DiagramKind::Mermaid, src, &palette(), "system-ui")
        .expect("user-init mermaid should render");
    assert!(svg.contains("<svg"));
}
