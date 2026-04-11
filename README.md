**English** | [中文](README_CN.md)

# me

A modern, context-aware replacement for `whoami`.

`me` keeps the Unix-tool shape: small, fast, local-first, and scriptable. It adds enough structure to answer "who am I here?" without becoming a system inventory tool.

## Quick Example

```txt
user@dev-machine  zsh

groups:     staff, admin, _developer (+3)
privilege:  user
ssh:        no
network:    192.168.0.10 (+2)

context:    rust (rustc 1.94.0)
```

## Installation

### macOS (Homebrew)

```bash
brew tap harveyTon/me
brew install me
```

### Download binary

Releases: [github.com/harveyTon/me/releases](https://github.com/harveyTon/me/releases)

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/harveyTon/me/main/scripts/install.sh)
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

### Verify the install

```bash
me
me --compact
```

## Why `me`

`whoami` is great when all you want is the username:

```console
$ whoami
user
```

`me` keeps that shape, but adds the bits I usually end up checking right after:

```console
$ me
user@dev-machine  zsh
...
ssh:        no
network:    192.168.0.10 (+2)
context:    rust (rustc 1.94.0)
```

The goal is still small and local: identity first, context second, and no drift into a general system inspector.

## Examples

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

`me` creates the default config on first run if it does not exist. If the file is invalid, `me` prints a warning and falls back to built-in defaults.

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

The current version is `v0.1.2`. Releases follow semantic versioning.

Build local release artifacts:

```bash
scripts/build.sh
```

The GitHub release workflow builds all release artifacts. The local script builds host-supported targets by default; pass a specific Rust target to build one artifact.

Linux ARM64 cross-compilation uses `cross` when the host is not Linux ARM64. Windows MSVC artifacts are built on Windows runners.

Create the release tag:

```bash
git tag -a v0.1.2 -m "Release v0.1.2"
git push origin v0.1.2
```

The GitHub Actions release workflow builds and uploads:

- `me-v0.1.2-macos-arm64.tar.gz`
- `me-v0.1.2-macos-x64.tar.gz`
- `me-v0.1.2-linux-x64.tar.gz`
- `me-v0.1.2-linux-arm64.tar.gz`
- `me-v0.1.2-windows-x64.zip`
- `me-v0.1.2-windows-arm64.zip`

See `RELEASE_CHECKLIST.md` for the full release checklist.

## Roadmap

- Refine the default block output to remove small duplications and keep the identity summary primary.
- Refine `--compact` for prompt usage and keep its output stable across common shell environments.
- Improve output consistency across local, SSH, container, and non-interactive sessions.
- Optimize startup time for prompt usage and add a lighter fast path for frequent invocation.
- Improve existing project detection and add lightweight Git branch context where it stays quiet and useful.
- Improve lightweight container detection without turning `me` into a general system inspector.
- Keep release binaries, Homebrew distribution, and the install script aligned across platforms.
- Expand snapshot coverage for output stability and keep CLI help and usage examples tight.

## License

[MIT](LICENSE)
