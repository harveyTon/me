# Release Notes

## v0.4.0

Block-output presentation update for the current `me` CLI surface.

### Highlights

- Reworked block-style text output into fixed `identity`, `system`, `session`, `network`, and `location` groups without a standalone header line.
- Kept `--fast` and `--full` on the same grouped layout, with only field density changing by mode.
- Refined block network rendering so default and fast modes show separate IPv4 and IPv6 summary rows, while `--full` keeps expanded address lists.
- Kept compact, JSON, config, install, update, and detector behavior unchanged.

## v0.3.4

Focused reliability and release-hygiene hardening.

### Highlights

- Added SHA256 verification for release-binary self-update before replacement.
- Made shell integration writes atomic, with rollback on failure and optional backup support in the write path.
- Hardened config loading with a size limit and shorter user-facing fallback warnings.
- Kept network collection failure silent while adding separate IPv4 and IPv6 collection and display.
- Added lightweight version consistency checks to the release validation flow.

## v0.3.3

Release-prep alignment for the current `me` CLI surface.

### Highlights

- Kept multi-project context coverage and bounded output density aligned across README, man page, and release docs.
- Carried forward shell integration install and uninstall behavior, including managed block markers and non-interactive safeguards.
- Polished `me update` install-source reporting so Homebrew installs on macOS are identified as `homebrew`, release binaries remain `release binary`, and unclear cases fall back to `unknown`.
- Tightened release-facing documentation and checklist text so install, update, artifact, and verification steps match the current repository state.

## v0.3.1

Shell integration polish for `me install` and `me uninstall`.

### Highlights

- Added safe non-interactive uninstall support with `--file` or explicit `--yes` for global removal.
- Improved install and uninstall summaries so affected modes and files are easier to scan.
- Refined interactive install wording for login and interactive shell choices.
- Updated README and man page documentation for shell integration commands.

## v0.3.0

Expanded project context detection for `me`, a better `whoami` for your shell.

### Highlights

- **Project sensing is now detector-based**: built-in project context detection uses an internal compile-time registry, making new project types easy to add without centralizing logic in one file.
- **Multiple project signals can coexist**: `me` can now report more than one project context in the same directory instead of collapsing to a single match.
- **Structured project context in JSON**: JSON output now exposes `context.projects` as structured entries while keeping Git context separate and machine-friendly.
- **Broader built-in project support**: project detection now covers Rust, Node, Python, Go, Java, Ruby, C/C++, PHP, Lua, Swift, R, and C#.
- **Richer Node, Python, and Java context**: Node surfaces common package-manager and workspace clues, Python includes virtualenv names, and Java distinguishes Maven and Gradle.
- **Text output stays bounded**: block and compact output now use a shared density limit for project-related context, folding overflow into `(+N)` instead of growing without bound.
- **Default output remains quiet**: identity, session, `pwd`, network, and Git semantics stay stable while project context becomes more capable underneath.

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

- `me-<tag>-macos-arm64.tar.gz`
- `me-<tag>-macos-x64.tar.gz`
- `me-<tag>-linux-x64.tar.gz`
- `me-<tag>-linux-arm64.tar.gz`
- `me-<tag>-windows-x64.zip`
- `me-<tag>-windows-arm64.zip`

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
