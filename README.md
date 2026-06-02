# Vista

A fast, cross-platform desktop gallery client for hitomi.la, built with Rust + Tauri and Svelte.

![CI](https://github.com/nyattic/Vista/actions/workflows/ci.yml/badge.svg)

## Features

- **Browse & search** — type, language, and sort filters, plus multi-term queries with namespaces (`artist:`, `female:`, `series:`, …) and live tag suggestions.
- **Reader** — continuous, paged, and two-page spread layouts with left-to-right / right-to-left direction and next-page prefetch.
- **Library** — favorites, view history, and per-gallery reading progress, stored locally in SQLite.
- **Downloads** — a download queue with pause, resume, cancel, and retry.
- **Caching** — on-disk image cache with a configurable size limit and automatic eviction.
- **Quality of life** — tag blacklist, light/dark themes, and built-in auto-update.

## Tech stack

- **Backend:** Rust, Tauri 2, reqwest, rusqlite
- **Frontend:** Svelte 5, TypeScript, Tailwind CSS, Vite

## Development

Requires [Node.js 24](https://nodejs.org), a [Rust toolchain](https://rustup.rs), and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform.

```bash
npm install
npm run tauri dev      # run the app in development
npm run tauri build    # produce a release bundle
npm run check          # type-check the frontend
```

Rust checks and tests live under `src-tauri`:

```bash
cd src-tauri
cargo clippy --all-targets
cargo test
```

## Releases

Pushing a `v*` tag (e.g. `v0.1.1`) triggers the release workflow, which builds signed bundles for Windows, macOS, and Linux and publishes them to GitHub Releases. Installed apps then update themselves through the Tauri updater.

Signing requires a `TAURI_SIGNING_PRIVATE_KEY` repository secret; the matching public key is committed in `src-tauri/tauri.conf.json`.

## License

[MIT](LICENSE)
