# install-rs

An interactive TUI installer for utilities in this repository.

## Usage

Run from the repository root:

```bash
./install
```

Or build and run directly:

```bash
cd install-rs
cargo run --release -- /path/to/scripts
```

## Controls

| Key | Action |
|---|---|
| `↑`/`↓` | Navigate utilities |
| `Space` | Toggle selection |
| `Enter` | Apply changes and quit |
| `Esc` / `q` | Quit without changes |

## How it works

- Scans the project root for utility directories (non-hidden subdirectories with an executable of the same name)
- Checks `$HOME/.local/bin` for existing symlinks to determine install status
- Already installed utilities start checked — unchecking removes the symlink, checking creates one
- Displays a color-coded summary after applying changes:
  - Green: installed
  - Yellow: uninstalled
  - Red: errors

## Building

```bash
cargo build --release
```

The binary is placed in `target/release/install-rs`.
