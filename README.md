# me

A modern, context-aware replacement for `whoami`.

`me` keeps the Unix-tool shape: small, fast, local-first, and scriptable. It adds enough structure to answer "who am I here?" without becoming a system inventory tool.

## Quick Example

```txt
user@dev-machine  zsh

uid:        501
gid:        20
groups:     staff, admin, _developer (+3)
shell:      zsh
pid:        12345
ppid:       6789
tty:        ttys001
privilege:  user
sudo:       no
ssh:        no
network:    192.168.0.10 (+2)

context:    rust (rustc 1.94.0)
```

## Why `me`

`whoami` answers one question:

```console
$ whoami
user
```

`me` answers the same question with local context:

```console
$ me
user@dev-machine  zsh

uid:        501
gid:        20
groups:     staff, admin, _developer (+3)
shell:      zsh
pid:        12345
ppid:       6789
tty:        ttys001
privilege:  user
sudo:       no
ssh:        no
network:    192.168.0.10 (+2)

context:    rust (rustc 1.94.0)
```

The goal is not to list everything about the system. The goal is to show the identity and session details that matter most in a calm, readable form.

## Examples

### Local Rust Project

```txt
user@dev-machine  zsh

uid:        501
gid:        20
groups:     staff, admin, _developer (+3)
shell:      zsh
pid:        12345
ppid:       6789
tty:        ttys001
privilege:  user
sudo:       no
ssh:        no
network:    192.168.0.10 (+2)

context:    rust (rustc 1.94.0)
```

### SSH Session

```txt
user@server-01  bash

uid:        1000
gid:        1000
groups:     user, deploy
shell:      bash
pid:        24811
ppid:       24803
tty:        pts/0
privilege:  user
sudo:       no
ssh:        yes
network:    10.0.0.5

context:    rust (rustc 1.94.0)
```

### sudo

```txt
root@server-01  bash

uid:        0
gid:        0
groups:     root
shell:      bash
pid:        24902
ppid:       24890
tty:        pts/0
privilege:  root
sudo:       yes
ssh:        yes
network:    10.0.0.5

context:    rust (rustc 1.94.0)
```

### Compact Mode

```txt
user@dev-machine · zsh · user · local · rust
```

## Installation

### Homebrew (recommended on macOS)

```bash
brew tap harveyTon/me
brew install me
```

Tap repository: `harveyTon/homebrew-me`  
Formula path: `Formula/me.rb`

### Download a prebuilt binary

Supported release artifacts:

- `me-v0.1.0-macos-arm64.tar.gz`
- `me-v0.1.0-macos-x64.tar.gz`
- `me-v0.1.0-linux-x64.tar.gz`
- `me-v0.1.0-linux-arm64.tar.gz`
- `me-v0.1.0-windows-x64.zip`
- `me-v0.1.0-windows-arm64.zip`

One-line installer for macOS and Linux:

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/harveyTon/me/main/scripts/install.sh)
```

The installer prefers `/usr/local/bin` when it is writable, otherwise it installs to `~/.local/bin`.

Pinned version:

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/harveyTon/me/main/scripts/install.sh) -- v0.1.0
```

Manual install on macOS:

```bash
curl -fsSL -o me.tar.gz https://github.com/harveyTon/me/releases/download/v0.1.0/me-v0.1.0-macos-arm64.tar.gz
tar -xzf me.tar.gz
install -d ~/.local/bin
install -m 0755 me ~/.local/bin/me
```

Manual install on Linux:

```bash
curl -L -o me.tar.gz https://github.com/harveyTon/me/releases/download/v0.1.0/me-v0.1.0-linux-x64.tar.gz
tar -xzf me.tar.gz
mkdir -p ~/.local/bin
install -m 0755 me ~/.local/bin/me
```

Windows:

```powershell
Invoke-WebRequest -Uri https://github.com/harveyTon/me/releases/download/v0.1.0/me-v0.1.0-windows-x64.zip -OutFile me.zip
Expand-Archive .\me.zip -DestinationPath .
New-Item -ItemType Directory -Force "$HOME\AppData\Local\Programs\me" | Out-Null
Move-Item .\me.exe "$HOME\AppData\Local\Programs\me\me.exe" -Force
```

Add `%USERPROFILE%\AppData\Local\Programs\me` to the user `Path` environment variable if `me` is not found in a new terminal.

If `me` is not found after install, add the binary directory to `PATH`:

- `/usr/local/bin`
- `~/.local/bin`
- Windows: `%USERPROFILE%\AppData\Local\Programs\me`

Checksums are published with each GitHub release in `SHA256SUMS.txt`.

### Cargo

Current tag install:

```bash
cargo install --git https://github.com/harveyTon/me --tag v0.1.0
```

After the crate is published to crates.io:

```bash
cargo install me
```

### From Source

```bash
git clone https://github.com/harveyTon/me.git
cd me
cargo build --locked --release
```

Binary:

```bash
target/release/me
```

Optional man page install:

```bash
install -d ~/.local/share/man/man1
install -m 0644 man/man1/me.1 ~/.local/share/man/man1/me.1
mandb ~/.local/share/man 2>/dev/null || true
man me
```

### Verify the install

```bash
me
me --compact
```

## Usage

```bash
me
me --compact
me --json
me --help
```

Field filters:

```bash
me -u
me -u -h
me -n
```

Other modes:

```bash
me --plain
me --format config
me --watch
me --full
```

## Output Modes

### Default

Human-friendly block output:

```bash
me
```

### Compact

One-line output for prompts and quick checks:

```bash
me --compact
```

```txt
user@dev-machine · zsh · user · local · rust
```

### JSON

Machine-readable output for scripts:

```bash
me --json
```

```json
{
  "user": "user",
  "host": "dev-machine",
  "uid": 501,
  "gid": 20,
  "sudo": false,
  "ssh": false
}
```

## Context Awareness

`me` detects a small set of local context signals automatically:

- SSH sessions
- Docker/container environments
- Rust projects via `Cargo.toml`
- Node projects via `package.json`

Context is summarized as a secondary signal:

```txt
context:    rust (rustc 1.94.0)
```

The default command stays local-first. It does not query cloud identity, inspect remote services, or perform public IP lookups.

## Shell Integration

### zsh

```zsh
PROMPT='$(me --compact) %~ %# '
```

### bash

```bash
PS1='$(me --compact) \w \$ '
```

Prompt commands run often. If your prompt feels slow, keep `me --compact` out of the hot path or cache its output in your shell configuration.

## Configuration

Config file:

```txt
~/.config/me/config.yaml
```

Minimal example:

```yaml
view: block
icons: auto

context:
  enabled: true
  project: true
  container: true
  ssh: true
```

CLI flags override environment variables, which override config, which overrides defaults.

## Philosophy

`me` is identity-first.

The default output should answer the common question quickly: who am I, on which machine, in what session?

Context is useful, but it should not dominate the output. It appears as a quiet secondary signal after the main identity and state fields.

There is no heavy UI, no daemon, and no plugin system. The output is plain text by default and should remain readable in any terminal, with or without color.

## Release

The initial version is `v0.1.0`. Releases follow semantic versioning.

Build local release artifacts:

```bash
scripts/build.sh
```

The GitHub release workflow builds all release artifacts. The local script builds host-supported targets by default; pass a specific Rust target to build one artifact.

Linux ARM64 cross-compilation uses `cross` when the host is not Linux ARM64. Windows MSVC artifacts are built on Windows runners.

Create the release tag:

```bash
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

The GitHub Actions release workflow builds and uploads:

- `me-v0.1.0-macos-arm64.tar.gz`
- `me-v0.1.0-macos-x64.tar.gz`
- `me-v0.1.0-linux-x64.tar.gz`
- `me-v0.1.0-linux-arm64.tar.gz`
- `me-v0.1.0-windows-x64.zip`
- `me-v0.1.0-windows-arm64.zip`

See `RELEASE_CHECKLIST.md` for the full release checklist.

## Roadmap

- Refine the default block output to remove small duplications and keep the identity summary primary.
- Refine `--compact` for prompt usage and keep its output stable across common shell environments.
- Improve output consistency across local, SSH, container, and non-interactive sessions.
- Optimize startup time for prompt usage and add a lighter fast path for frequent invocation.
- Improve existing project detection and add lightweight Git branch context where it stays quiet and useful.
- Improve lightweight container detection without turning `me` into a general system inspector.
- Finalize Homebrew distribution, keep release binaries aligned across platforms, and maintain a simple install script.
- Expand snapshot coverage for output stability and keep CLI help and usage examples tight.

## License

MIT
