**English** | [中文](README_CN.md)

# me

A better `whoami`, with context.

`me` keeps the Unix-tool shape: small, fast, local-first, and scriptable. It adds enough structure to answer "who am I here?" without becoming a system inventory tool.

## Quick Example

```txt
user@dev-machine  zsh

uid:        501
gid:        20
groups:     staff, admin, _developer (+2)
privilege:  user
ssh:        no
network:    192.168.0.10 (+2)

pwd:        /Users/user/dev/me

context:    rust 1.94.1 · git(main)
```

## Installation

The fastest way on macOS is Homebrew. For other platforms, use a release binary.

### macOS (Homebrew, recommended)

```bash
brew tap harveyTon/me
brew install me
```

### macOS / Linux (one-line install)

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/harveyTon/me/main/scripts/install.sh)
```

### Download a release binary

Releases: [github.com/harveyTon/me/releases](https://github.com/harveyTon/me/releases)
Prebuilt archives are available for macOS, Linux, and Windows.

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

`me` keeps that shape, but answers the next questions:

- who am I?
- where am I (`pwd`)?
- am I in SSH, `sudo`, or a container?
- what project am I working on (`rust`, `node`, `python`, ...)?

The goal stays narrow: identity first, context second, location included, with no drift into a general system inspector.

## Examples

### SSH Session

```txt
user@server-01  bash

uid:        1000
gid:        1000
groups:     user, deploy
pid:        24811
ppid:       24803
tty:        pts/0
privilege:  user
sudo:       no
ssh:        yes
network:    10.0.0.5

pwd:        /srv/app

context:    rust 1.94.1 · git(main)
```

### sudo

```txt
root@server-01  bash

uid:        0
gid:        0
groups:     root
pid:        24902
ppid:       24890
tty:        pts/0
privilege:  root
sudo:       yes
ssh:        yes
network:    10.0.0.5

pwd:        /srv/app

context:    rust 1.94.1 · git(main)
```

### Compact Mode

```txt
user@dev-machine · local · rust 1.94.1 · git:main · me
```

### JSON

```json
{
  "user": "user",
  "pwd": {
    "raw": "/Users/user/dev/me",
    "display": "/Users/user/dev/me"
  },
  "context": {
    "projects": [
      {
        "kind": "rust",
        "version": "1.94.1"
      }
    ],
    "git": {
      "branch": "main"
    }
  }
}
```

## Usage

```bash
me
me --compact
me --json
me update --check
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
me --compact --fast
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
user@dev-machine · local · rust 1.94.1 · git:main · me
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
  "privilege": "user",
  "sudo": false,
  "ssh": false,
  "pwd": {
    "raw": "/Users/user/dev/me",
    "display": "/Users/user/dev/me"
  }
}
```

## Context Awareness

`me` detects a small set of local context signals automatically:

- SSH sessions
- Docker/container environments
- project context for Rust, Node, Python, Go, Java, Ruby, C/C++, PHP, Lua, Swift, R, C#, and Docker Compose
- Git branches when the current directory is inside a Git work tree

Multiple project signals can coexist, but default output stays bounded and quiet.
Context stays a secondary signal in the default text output, even when several detectors match.

```txt
context:    node 24.14.1 (pnpm, turbo) · python 3.12 (.venv) · git(main) (+1)
```

The default command stays local-first. It does not query cloud identity, inspect remote services, or perform public IP lookups.

## Shell Integration

Set up `me` in your shell:

```bash
me install
```

Remove it later:

```bash
me uninstall
```

Update `me` itself:

```bash
me update
```

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
  git: true
```

CLI flags override environment variables, which override config, which overrides defaults.

## Philosophy

`me` is identity-first.

The default output should answer the common question quickly: who am I, on which machine, in what session? Context is useful, but it should not dominate the output. It appears as a quiet secondary signal after the main identity and state fields.

There is no heavy UI, no daemon, and no plugin system. The output is plain text by default and should remain readable in any terminal, with or without color.

## Release

The current version is tracked in `Cargo.toml`. Releases follow semantic versioning.

The GitHub release workflow builds artifacts for:

- macOS arm64
- macOS x64
- Linux x64
- Linux arm64
- Windows x64
- Windows arm64

Local release artifacts can be built with:

```bash
scripts/build.sh
```

See [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) for the detailed release procedure.

## Roadmap

- Refine multi-project context density so the default text output stays quiet and useful.
- Improve detector coverage where project signals remain strong, cheap, and local-first.
- Continue stabilizing block, compact, JSON, and prompt-oriented output behavior.
- Keep startup time predictable for prompt usage, especially in `--fast` mode.
- Keep release artifacts, Homebrew distribution, and install paths aligned across platforms.

## License

[MIT](LICENSE)
