# scripts

A collection of utility scripts and commands for various tasks.

## Contents

- [yt-sync](./yt-sync/README.md): A utility to download YouTube videos with subtitles and comments (using an archive to avoid re-downloads) and sync them to a given `rclone` remote (e.g., Cloudflare R2).

## Usage

Each utility may have its own instructions. Refer to the README in the respective utility's directory.

## Installation

You can install the utilities using the provided `install` script:

```bash
./install
```

This will launch an interactive TUI that allows you to:
- Navigate with `↑`/`↓` arrow keys
- Select/deselect utilities with `Space`
- Apply changes with `Enter`
- Quit with `Escape` or `q`

Utilities are installed as symlinks to `$HOME/.local/bin`. Make sure that directory is in your `PATH`.

## Requirements

- Rust toolchain (to build the installer from source)

The installer is written in Rust and will be built automatically on first run if `cargo` is available. Install Rust from <https://rustup.rs>

## Contributing

To add a new utility/command:

1. Create a new folder for the utility (e.g., `mytool/`).
2. Place the executable script inside that folder (e.g., `mytool/mytool`).
3. Add a `README.md` inside the folder describing usage, requirements, installation notes, etc.
4. Ensure the script is executable (`chmod +x mytool/mytool`).
5. Update this root `README.md` to list the new utility under "## Contents" with a link to its README.
6. Commit your changes.

The installer will automatically detect the new utility.

## License

MIT
