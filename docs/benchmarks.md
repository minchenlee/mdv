# mdv benchmarks

Hardware: MacBook Pro (Apple M2), macOS 26.1
Build: `cargo build --release` at commit `032c334c5f5d06381e2ce02af2938d4fa99bcf3f`
Date: 2026-05-10

## Cold start

Measured by `mdv --benchmark-startup`. Median of 5 runs.

| Checkpoint | Time |
|---|---|
| Process entry → `pre_run` (just before iced::application().run_with()) | ~14 µs |

The window appears before system fonts finish loading. `load_system_fonts` is deferred to the first `App::view()` call, so the window paints first and the (slower) full system-font scan runs lazily on the render thread.

## Parse + highlight

Criterion: `cargo bench --bench cold_start`.

| Workload | Median |
|---|---|
| `parse_10k_lines` (~1 MB synthetic markdown, parse only — highlight is lazy) | 8.1 ms |
| `font_system_load_system_fonts` (one-time, deferred to first view) | 7.7 ms |

Highlighting moved out of parser in v0.2 (`HlCache` LRU), making subsequent re-parses on hot reload reuse cached spans for unchanged code blocks.

## Reproducing

    cargo bench --bench cold_start
    ./target/release/mdv --benchmark-startup
