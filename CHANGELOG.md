# Changelog

## 0.1.0 — 2026-02-13

Initial release.

- Background daemon with clipboard polling
- Text, link (auto-detected), and image (PNG) support
- 12-hour retention with automatic cleanup
- SQLite storage
- Wayland (`wl-paste`/`wl-copy`) and X11 (`xclip`) support
- Global hotkey to open GUI popup (configurable)
- CLI: `list`, `get`, `search`, `clear`, `status`, `stop`
- Config file at `~/.config/sticky_one/config.toml`
