# Vista

![Release](https://img.shields.io/github/v/release/nyattic/Vista?style=flat&color=6366f1)
![Downloads](https://img.shields.io/github/downloads/nyattic/Vista/total?style=flat&color=10b981)
![Last Commit](https://img.shields.io/github/last-commit/nyattic/Vista?style=flat&color=f59e0b)
![License](https://img.shields.io/badge/license-MIT-8b5cf6?style=flat)

A fast, cross-platform desktop gallery client for hitomi.la.

Browse in a small, focused window — no web browser, no ads. Favorites,
history, reading progress, and downloads all stay on your machine.

## Features

- Browse by type, language, and time period, with popular and latest sorts
- Multi-term search with namespaces (`artist:`, `female:`, `series:`, …),
  combined with AND, plus live tag suggestions and search history
- Built-in reader with continuous, paged, and two-page spread layouts,
  left-to-right / right-to-left direction, and next-page prefetch
- Favorites, view history, and per-gallery reading progress, stored
  locally in SQLite
- Download queue with pause, resume, cancel, and retry
- On-disk image cache with a configurable size limit and automatic eviction
- Tag blacklist, light and dark themes, and built-in auto-update

## Installation

Download the latest build for your platform from the
[Releases](https://github.com/nyattic/Vista/releases) page:

- **Windows** — `.exe` installer
- **macOS** — `.dmg`
- **Linux** — `.AppImage`

## Usage

1. Launch Vista.
2. Filter by type, language, and time period from the toolbar, or search
   with the bar at the top.
3. Click a gallery to inspect it in the sidebar; double-click (or press
   Enter) to open the reader.
4. Favorite, download, or copy a link from the sidebar. Track downloads
   from the panel in the header.
5. Open Settings to change the reader layout, theme, default language,
   blacklist, download folder, and cache size.

## Search syntax

Separate terms with spaces; every term must match (AND). Use namespaces to
target a field, and underscores for multi-word values:

```
artist:naoki_urasawa language:korean female:sole_female
```

Supported prefixes: `tag:`, `female:`, `male:`, `artist:`, `group:`,
`series:`, `character:`, `type:`, `language:`. A term with no prefix is
treated as a tag.

## Keyboard shortcuts

Browsing:

| Key | Action |
| --- | --- |
| `←` `→` | Move selection left / right (flips the page at the edge) |
| `↑` `↓` | Move selection up / down a row |
| `Enter` | Open the selected gallery in the reader |
| `Esc` | Close the reader, settings, or a dialog |

Search box:

| Key | Action |
| --- | --- |
| `↑` `↓` | Move through tag suggestions and history |
| `Tab` | Insert the highlighted tag |
| `Enter` | Run the search (or pick a history entry) |
| `Esc` | Dismiss the suggestions |

Reader:

| Key | Action |
| --- | --- |
| `←` `→` | Previous / next page (paged and spread; respects direction) |
| `Space` | Next page |
| `Home` `End` | Jump to the first / last page |
| `Esc` | Close the reader (or exit zoom) |

## Building from source

Requires [Node.js 24+](https://nodejs.org/), [Rust](https://rustup.rs/),
and the platform dependencies for [Tauri 2](https://v2.tauri.app/start/prerequisites/).

```sh
npm install
npm run tauri dev      # run in development
npm run tauri build    # produce a release bundle
```

## License

[MIT](LICENSE). Not affiliated with or endorsed by hitomi.la.
