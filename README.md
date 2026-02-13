# syo

Lightweight clipboard manager for Linux. Keeps 12-hour history of text, links, and images.

## Features

- Background daemon monitors clipboard
- Auto-detects URLs
- Stores images (PNG, up to 5MB)
- SQLite storage with automatic cleanup
- Wayland & X11 support
- Global hotkey to open GUI popup
- Configurable via TOML

## Dependencies

**Runtime:**
- Wayland: `wl-clipboard` (`wl-paste`, `wl-copy`)
- X11: `xclip`

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

Add `~/.local/bin` to your PATH (add to `~/.bashrc` or `~/.zshrc`):
```bash
export PATH="$HOME/.local/bin:$PATH"
```

### From git
```bash
cargo install --git https://github.com/olajoao/sticky-one
```

### Arch Linux (AUR)
```bash
# Using an AUR helper
paru -S syo
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

syo popup           # open GUI popup
syo --version       # print version
```

## Systemd setup

Install the user service:
```bash
mkdir -p ~/.config/systemd/user
cp contrib/syo.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now syo
```

Check status:
```bash
systemctl --user status syo
```

## Configuration

Config file: `~/.config/sticky_one/config.toml`

A default config is created on first run. Example:

```toml
[hotkey]
modifiers = ["Alt", "Shift"]
key = "C"
```

### Hotkey options

**Modifiers:** `Alt`, `Shift`, `Ctrl`, `Super` (and `Right_Alt`, `Right_Shift`, `Right_Ctrl`, `Right_Meta`)

**Keys:** `A`-`Z`, `0`-`9`, `F1`-`F12`, `Space`, `Enter`, `Escape`, `Tab`, `Backspace`

## Shell completions

Completions are generated at build time inside `target/`. After `cargo build --release`:

```bash
OUT=$(find target/release/build -path '*/sticky_one-*/out' -type d | head -1)

# Bash
cp "$OUT/completions/syo.bash" ~/.local/share/bash-completion/completions/syo

# Zsh
cp "$OUT/completions/_syo" ~/.local/share/zsh/site-functions/_syo

# Fish
cp "$OUT/completions/syo.fish" ~/.config/fish/completions/syo.fish

# Man page
cp "$OUT/man/syo.1" ~/.local/share/man/man1/syo.1
```

## Storage

Data stored in `~/.local/share/sticky_one/`:
- `clipboard.db` — SQLite database (mode 600)
- `daemon.pid` — PID file
- `daemon.log` — daemon log file

## Troubleshooting

**"Missing dependency" error on `syo daemon`:**
Install the required clipboard tools for your display server:
```bash
# Wayland
sudo pacman -S wl-clipboard   # Arch
sudo apt install wl-clipboard  # Debian/Ubuntu

# X11
sudo pacman -S xclip           # Arch
sudo apt install xclip          # Debian/Ubuntu
```

**Daemon not starting:**
Check the log file at `~/.local/share/sticky_one/daemon.log`.

**Hotkey not working:**
- Ensure your user has access to `/dev/input/event*` devices (usually requires `input` group)
- Check `syo status` to confirm the daemon is running
- Verify your config at `~/.config/sticky_one/config.toml`

## License

MIT
