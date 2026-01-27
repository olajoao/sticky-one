# syo

Lightweight clipboard manager for Linux. Keeps 12-hour history of text, links, and images.

## Features

- Background daemon monitors clipboard
- Auto-detects URLs
- Stores images (PNG, up to 5MB)
- SQLite storage
- Wayland & X11 support

## Dependencies

**Runtime:**
- `wl-paste`, `wl-copy` (Wayland)
- `xclip` (X11)

**Build:**
- Rust 1.70+

## Install

### From source
```bash
git clone https://github.com/olajoao/sticky-one.git
cd sticky_one
cargo build --release
mkdir -p ~/.local/bin
cp target/release/syo ~/.local/bin/
```

⚠️ **Add `~/.local/bin` to your PATH** (required for `syo` command to work):
```bash
# Add to ~/.bashrc or ~/.zshrc:
export PATH="$HOME/.local/bin:$PATH"
```
Then restart your terminal.

### From git
```bash
cargo install --git https://github.com/olajoao/sticky-one
```

## Usage

```bash
syo daemon          # start background monitor
syo stop            # stop daemon
syo status          # check if running

syo list            # show recent entries
syo list -l 50      # show last 50 entries
syo get <id>        # copy entry to clipboard
syo search <query>  # search text/links
syo clear           # wipe history
```

## Storage

Data stored in `~/.local/share/sticky_one/`:
- `clipboard.db` - SQLite database
- `daemon.pid` - PID file

## License

MIT
