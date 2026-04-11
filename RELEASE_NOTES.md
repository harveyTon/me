# Release Notes

## v0.2.3

Location and color-mode refinement for `me`, a modern, context-aware replacement for `whoami`.

### Highlights

- **`pwd` is now a first-class signal**: block output includes a dedicated `pwd:` row between the main identity/runtime/state fields and the context summary.
- **Compact output now ends with the current directory name**: the final segment uses the basename of the working directory, keeping prompt output short and location-aware.
- **Structured `pwd` in JSON**: JSON output now includes a `pwd` object with `raw` and `display` paths when the current directory is available.
- **Config-style output includes `pwd`**: `--format config` now emits the display path as `pwd = ...`.
- **Color mode is formalized**: config now supports `color: auto | on | off`, with `--no-color` and `NO_COLOR` still taking precedence.
- **Fast mode keeps location available**: `pwd` remains present in block, compact, and JSON output even when `--fast` is used.
- **Windows test stability improved**: release validation no longer assumes one exact path normalization format on Windows runners.

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

Binary release artifacts are archives. Unix archives contain `me`; Windows archives contain `me.exe`.

Checksums are published with the release as `SHA256SUMS.txt`.

### Release Artifacts

Supported platforms:

- macOS arm64
- macOS x64
- Linux x64
- Linux arm64
- Windows x64
- Windows arm64

Expected binary artifact names:

- `me-v0.2.3-macos-arm64.tar.gz`
- `me-v0.2.3-macos-x64.tar.gz`
- `me-v0.2.3-linux-x64.tar.gz`
- `me-v0.2.3-linux-arm64.tar.gz`
- `me-v0.2.3-windows-x64.zip`
- `me-v0.2.3-windows-arm64.zip`

## v0.2.1

JSON cleanup update for `me`, a modern, context-aware replacement for `whoami`.

### Highlights

- Human-readable default block output.
- Compact output for prompts and quick checks.
- JSON output for scripts.
- Fast-mode JSON now omits absent project versions instead of emitting `"version": null`.
- Config-style output for simple shell parsing.
- Field selectors for common identity, runtime, state, and network fields.
- Local context detection for SSH sessions, containers, Rust projects, Node projects, and lightweight Git branch context.
- SSH session detection remains meaningful under common `sudo` execution paths.
- Default block output keeps the network summary visible, including in `--fast` mode.
- `--fast` is available for prompt usage and skips slower context version checks without dropping explicit or default network output.
- Watch mode for lightweight refreshes.
- Copy mode for local clipboard workflows.
- Man page included at `man/man1/me.1`.
- `me` now creates `~/.config/me/config.yaml` on first run when it does not exist.
- Invalid config files now emit a warning and fall back to defaults instead of blocking normal use.
- Theme settings now participate in the render path rather than being ignored.

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

- `me-v0.2.1-macos-arm64.tar.gz`
- `me-v0.2.1-macos-x64.tar.gz`
- `me-v0.2.1-linux-x64.tar.gz`
- `me-v0.2.1-linux-arm64.tar.gz`
- `me-v0.2.1-windows-x64.zip`
- `me-v0.2.1-windows-arm64.zip`
