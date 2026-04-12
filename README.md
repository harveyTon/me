**English** | [中文](README_CN.md)

# me

A better `whoami` for your shell — shows who you are, where you are, and what you're working on.

## Install

### macOS (Homebrew)

```bash
brew tap harveyTon/me
brew install me
```

### Other platforms

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/harveyTon/me/main/scripts/install.sh)
```

Releases: [github.com/harveyTon/me/releases](https://github.com/harveyTon/me/releases)

### From source

```bash
git clone https://github.com/harveyTon/me.git
cd me
cargo build --locked --release
```

```bash
target/release/me
```

## Example

```txt
tiger@TigerdeMac-mini  zsh

uid:        501
gid:        20
groups:     staff, admin, _developer (+2)
pid:        18420
ppid:       18398
tty:        ttys001
privilege:  user
sudo:       no
ssh:        no
network:    192.168.0.10 (+2)

pwd:        /Users/tiger/dev/me

context:    rust 1.94.1 · git(main)
```

`whoami` tells you *who you are*.

`me` tells you:
- who you are
- where you are (`pwd`)
- what environment you're in (`ssh` / `sudo`)
- what project you're working on (`rust` / `node` / `python` / ...)

## Project context

`me` automatically detects common project types:

- Rust
- Node (`pnpm` / `yarn` / `npm` / `turbo` / `nx`)
- Python (with virtualenv)
- Go
- Java (`Maven` / `Gradle`)
- Ruby
- C / C++
- PHP
- Lua
- Swift
- R
- C#

Multiple project signals can coexist.

Default output stays minimal, but full context is available in JSON.

## Other modes

Compact:

```bash
me --compact
```

JSON:

```bash
me --json
```

A small tool, but surprisingly useful in daily shell work.
