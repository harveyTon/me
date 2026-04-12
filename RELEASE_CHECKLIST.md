# Release Checklist

Version strategy: releases follow semantic versioning. The release version is sourced from `Cargo.toml`.

Set the release tag once per shell:

```bash
VERSION="v$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
```

## Before Tagging

- Confirm `Cargo.toml` has the intended version.
- Confirm `README.md` examples match the current output.
- Confirm `man/man1/me.1` documents the current CLI flags and project context summary.
- Run `scripts/check-release.sh` before every release. This is required and includes:
  - `cargo fmt --check`
  - `cargo test --locked`
  - `cargo clippy --locked --all-targets --all-features -- -D warnings`
  - version consistency validation
  - Windows target compilation checks for `x86_64-pc-windows-msvc` and `aarch64-pc-windows-msvc`
- Command:

```bash
scripts/check-release.sh
```

- If you changed release scripts, version checks, or cross-platform tests, do not tag until the Windows target checks above pass locally.

## Create The Tag

```bash
git status --short
git add Cargo.toml Cargo.lock README.md README_CN.md RELEASE_NOTES.md RELEASE_CHECKLIST.md HOMEBREW.md .github/workflows/release.yml scripts man src tests
git commit -m "Prepare ${VERSION} release"
git tag -a "${VERSION}" -m "Release ${VERSION}"
git push origin main
git push origin "${VERSION}"
```

## Build Binaries

The release workflow builds and uploads:

- `me-${VERSION}-macos-arm64.tar.gz`
- `me-${VERSION}-macos-x64.tar.gz`
- `me-${VERSION}-linux-x64.tar.gz`
- `me-${VERSION}-linux-arm64.tar.gz`
- `me-${VERSION}-windows-x64.zip`
- `me-${VERSION}-windows-arm64.zip`
- `SHA256SUMS.txt`

For a local build on a supported host:

```bash
scripts/build.sh
```

Linux ARM64 cross-compilation uses `cross` when the host is not Linux ARM64:

```bash
cargo install cross --locked
scripts/build.sh aarch64-unknown-linux-gnu
```

Windows MSVC artifacts are built on Windows runners. macOS artifacts are built on macOS runners.

## GitHub Release

- Confirm the `release.yml` workflow completed.
- Confirm the Windows release jobs completed successfully before considering the release done.
- Confirm all expected artifacts are attached to the GitHub release.
- Confirm `SHA256SUMS.txt` is attached to the GitHub release.
- Copy the relevant text from `RELEASE_NOTES.md` into the release description if needed.
- Smoke-test one downloaded artifact:

```bash
tmpdir="$(mktemp -d)"
curl -L -o "${tmpdir}/me.tar.gz" "https://github.com/harveyTon/me/releases/download/${VERSION}/me-${VERSION}-linux-x64.tar.gz"
tar -C "${tmpdir}" -xzf "${tmpdir}/me.tar.gz"
"${tmpdir}/me" --help
```

## Artifact Verification

macOS:

```bash
tmpdir="$(mktemp -d)"
curl -L -o "${tmpdir}/me.tar.gz" "https://github.com/harveyTon/me/releases/download/${VERSION}/me-${VERSION}-macos-arm64.tar.gz"
tar -C "${tmpdir}" -xzf "${tmpdir}/me.tar.gz"
"${tmpdir}/me" --help
```

Linux:

```bash
tmpdir="$(mktemp -d)"
curl -L -o "${tmpdir}/me.tar.gz" "https://github.com/harveyTon/me/releases/download/${VERSION}/me-${VERSION}-linux-x64.tar.gz"
tar -C "${tmpdir}" -xzf "${tmpdir}/me.tar.gz"
"${tmpdir}/me" --help
```

Windows PowerShell:

```powershell
$version = "vX.Y.Z"
$tmpdir = New-Item -ItemType Directory -Force "$env:TEMP\me-$version"
Invoke-WebRequest -Uri "https://github.com/harveyTon/me/releases/download/$version/me-$version-windows-x64.zip" -OutFile "$tmpdir\me.zip"
Expand-Archive "$tmpdir\me.zip" -DestinationPath $tmpdir -Force
& "$tmpdir\me.exe" --help
```

## Homebrew

The release workflow updates `harveyTon/homebrew-me` automatically after the GitHub Release is published.

Required repository secret:

```txt
HOMEBREW_TAP_TOKEN
```

Use a fine-grained token with contents write access to `harveyTon/homebrew-me`.

If the tap update job fails, update the formula manually:

- Generate the release tarball SHA:

```bash
curl -L "https://github.com/harveyTon/me/archive/refs/tags/${VERSION}.tar.gz" | shasum -a 256
```

- Update `Formula/me.rb` in the Homebrew tap with the new URL and real SHA.
- Publish it to `harveyTon/homebrew-me`.
- Follow `HOMEBREW.md` for the exact tap setup and install verification steps.

## Final User Install Commands

Git:

```bash
cargo install --git https://github.com/harveyTon/me --tag "${VERSION}"
```

GitHub Releases installer on macOS/Linux:

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/harveyTon/me/main/scripts/install.sh)
```

Pinned version:

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/harveyTon/me/main/scripts/install.sh) -- "${VERSION}"
```

Homebrew, after `harveyTon/homebrew-me` is published:

```bash
brew tap harveyTon/me
brew install me
```

Crates.io, after publication:

```bash
cargo install me
```
