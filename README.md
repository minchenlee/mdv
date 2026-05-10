# mdv

**Free, open-source, native, cross-platform markdown viewer.**

A fast, beautiful markdown reader for browsing folders of `.md` files — without spinning up Obsidian's vault, Typora's editor weight, or a static-site build. Just open a folder and read.

Built in Rust with [Iced](https://iced.rs/). ~16 MB binary, no Electron, no Chromium tax.

## Why mdv

| | mdv | Marked 2 | Glow | Obsidian | Typora |
|---|:-:|:-:|:-:|:-:|:-:|
| Free | ✅ | ❌ ($14) | ✅ | freemium | ❌ ($15) |
| Open source | ✅ | ❌ | ✅ | ❌ | ❌ |
| Native (no Electron) | ✅ | ✅ | ✅ | ❌ | partial |
| Cross-platform GUI | ✅ | ❌ (mac only) | terminal | ✅ | ✅ |
| Folder workspace | ✅ | ❌ | ❌ | ✅ | ❌ |
| Read-only focus | ✅ | ✅ | ✅ | ❌ | ❌ |

mdv is the only one that hits all six.

## Features

- **Workspace browser** — open a folder, navigate the file tree
- **Command palette** (`⌘K`) — every action one keystroke away
- **Quick file finder** (`⌘P`) — fuzzy jump to any `.md` in workspace
- **Live reload** — edits in your editor reflect instantly
- **Syntax highlighting** via tree-sitter — Rust, Python, JS, TS, Go, C, Bash, JSON, HTML, Markdown
- **Light / dark themes** with system follow
- **CJK-friendly** — bundled Inter + JetBrains Mono, system font fallback
- **Vim-style scrolling** — `j` / `k` / `g` / `G`
- **Drag and drop** files or folders

## Install

### macOS / Windows

Download the latest installer from [Releases](https://github.com/minchenlee/mdv/releases):

- **macOS Apple Silicon** — `mdv_*_aarch64.dmg`
- **macOS Intel** — `mdv_*_x64.dmg`
- **Windows** — `mdv_*_x64-setup.exe`

> Builds are unsigned. On macOS, right-click → Open the first time. On Windows, click "More info" → "Run anyway" past SmartScreen.

### From source

    cargo build --release
    ./target/release/mdv path/to/file.md

Requires Rust 1.80+.

## Keyboard shortcuts

| Key | Action |
|---|---|
| `⌘P` | Open file finder |
| `⌘K` | Open command palette |
| `⌘O` | Open folder |
| `⌘B` | Toggle sidebar |
| `⌘F` | Search in document |
| `⌘T` | Toggle theme |
| `j` / `k` | Scroll down / up |
| `g` / `G` | Top / bottom |
| `Space` / `Shift+Space` | Page down / up |
| `Esc` | Close overlay / search |

## Roadmap

- [ ] Code signing (mac notarization, Windows cert)
- [ ] Auto-update
- [ ] Export to PDF / HTML
- [ ] More tree-sitter grammars

## License

MIT
