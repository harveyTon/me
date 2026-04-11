# Release Notes

## v0.2.2

Output refinement and Git detection update for `me`, a modern, context-aware replacement for `whoami`.

### Highlights

- **Block output de-duplicated**: `shell` field removed from body rows; already shown in header line (`user@host  shell`).
- **Compact output redesigned for prompt usage**: new 3-part format `user@host · env · project` with deterministic env priority (ssh > docker > local), root shown as `root@host`, no separate privilege segment.
- **Project and Git context coexist**: block and compact show both project kind and git branch when both are detected (e.g. `rust 1.94.0 · git(main)`).
- **Shortened project version rendering**: toolchain output trimmed to semver only (e.g. `rust 1.94.1` instead of `rustc 1.94.1 (e408947bf 2026-03-25) (Homebrew)`).
- **`--fast` flag**: skips slower context version checks for prompt usage; git detection remains active (file-based, fast).
- **Git detached head support**: resolves tags for detached HEAD state via loose refs and packed-refs; fast mode falls back to short OID.
- **Git detection boundary**: bounded to 8 directory levels, stops at home directory.
- **Root identity**: `root@host` in both block header and compact output when uid == 0.
- **Consistent cross-environment output**: field order, context position, and structure remain stable across local, SSH, container, and sudo sessions.
- **Expanded test coverage**: 56 tests including golden snapshots, git detection, and CLI behavior.

### Breaking Changes (output format)

- Default block output no longer includes a `shell:` body row (still in header).
- Compact output format changed from `user@host · shell · privilege · env · project` to `user@host · env · project`.

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
