# Release Notes

## v0.1.1

Linux compatibility update for `me`, a modern, context-aware replacement for `whoami`.

### Highlights

- Human-readable default block output.
- Compact output for prompts and quick checks.
- JSON output for scripts.
- Config-style output for simple shell parsing.
- Field selectors for common identity, runtime, state, and network fields.
- Local context detection for SSH sessions, containers, Rust projects, and Node projects.
- SSH session detection remains meaningful under common `sudo` execution paths.
- Watch mode for lightweight refreshes.
- Copy mode for local clipboard workflows.
- Man page included at `man/man1/me.1`.
- Linux release artifacts are now built against an older userspace baseline for better runtime compatibility.
- The installer verifies a staged binary before replacing an existing `me` on disk.

### Install

Recommended on macOS:

```bash
brew tap harveyTon/me
brew install me
```

One-line installer on macOS and Linux:

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/harveyTon/me/main/scripts/install.sh)
```

Homebrew tap, after publishing `harveyTon/homebrew-me`:

```bash
brew tap harveyTon/me
brew install me
```

Binary release artifacts are archives. Unix archives contain `me`; Windows archives contain `me.exe`.

Checksums are published with the release as `SHA256SUMS.txt`.

### Build From Source

```bash
git clone https://github.com/harveyTon/me.git
cd me
cargo build --locked --release
```

### Release Artifacts

Supported platforms:

- macOS arm64
- macOS x64
- Linux x64
- Linux arm64
- Windows x64
- Windows arm64

Expected binary artifact names:

- `me-v0.1.1-macos-arm64.tar.gz`
- `me-v0.1.1-macos-x64.tar.gz`
- `me-v0.1.1-linux-x64.tar.gz`
- `me-v0.1.1-linux-arm64.tar.gz`
- `me-v0.1.1-windows-x64.zip`
- `me-v0.1.1-windows-arm64.zip`
