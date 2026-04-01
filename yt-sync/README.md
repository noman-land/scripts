# yt-sync

A utility to download YouTube videos with all available subtitles and comments and sync them to a given rclone remote (e.g., Cloudflare R2) using rclone.

## Usage

```bash
yt-sync <YouTube URL> <rclone remote:path>
```

Example:
```bash
yt-sync "https://www.youtube.com/@the-channel" cf:my-bucket/files
```

## How it works

1. **Download**: Uses `yt-dlp` to download the best quality video and audio, merges them into an MP4 file, writes all available subtitles (including auto-generated) as `.srt` files, and writes comments as a `.comments.json` file. The `--download-archive` option is used to keep track of already downloaded videos, so only new videos are downloaded on subsequent runs.
2. **Sync**: Uses `rclone sync` to upload all downloaded files (video, subtitles, comments) to the given rclone remote:path, making the remote a mirror of the local directory.

## Requirements

- [yt-dlp](https://github.com/yt-dlp/yt-dlp)
- [rclone](https://rclone.org/) configured with a remote for your Cloudflare R2 bucket (or any other supported remote).
- Standard Unix utilities (`bash`, etc.)

## Installation

You can install yt-sync (and any other utilities in this repository), use the root install script:

```bash
cd /path/to/scripts
./install
```

The install script will present an interactive TUI where you can navigate with arrow keys and select utilities with space bar. It creates symbolic links in your PATH (default: ~/.local/bin).

After installation, you can run the utility directly from your terminal.

Prerequisites: yt-dlp and rclone must be installed and configured.

## Notes

- The script downloads directly into the current working directory (where you run the script).
- An archive file named `yt-dlp-archive.txt` is created in the same directory to keep track of downloaded videos and avoid re-downloading.
- The sync operation will mirror the local directory to the remote, so any local changes (including new downloads) will be uploaded.

## License

MIT