# Release Checklist

Version strategy: current release is `v0.1.1` and follows semantic versioning.

## Before Tagging

- Confirm `Cargo.toml` version is `0.1.1`.
- Confirm `README.md` examples match the current output.
- Confirm `man/man1/me.1` documents the current CLI flags.
- Run:

```bash
scripts/check-release.sh
```

## Create The Tag

```bash
git status --short
git add Cargo.toml Cargo.lock README.md RELEASE_NOTES.md RELEASE_CHECKLIST.md HOMEBREW.md Formula/me.rb .github/workflows/release.yml scripts man
git commit -m "Prepare v0.1.1 release"
git tag -a v0.1.1 -m "Release v0.1.1"
git push origin main
git push origin v0.1.1
```

## Build Binaries

The release workflow builds and uploads:

- `me-v0.1.1-macos-arm64.tar.gz`
- `me-v0.1.1-macos-x64.tar.gz`
- `me-v0.1.1-linux-x64.tar.gz`
- `me-v0.1.1-linux-arm64.tar.gz`
- `me-v0.1.1-windows-x64.zip`
- `me-v0.1.1-windows-arm64.zip`
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
- Confirm all expected artifacts are attached to the GitHub release.
- Confirm `SHA256SUMS.txt` is attached to the GitHub release.
- Copy the relevant text from `RELEASE_NOTES.md` into the release description if needed.
- Smoke-test one downloaded artifact:

```bash
tmpdir="$(mktemp -d)"
curl -L -o "${tmpdir}/me.tar.gz" https://github.com/harveyTon/me/releases/download/v0.1.1/me-v0.1.1-linux-x64.tar.gz
tar -C "${tmpdir}" -xzf "${tmpdir}/me.tar.gz"
"${tmpdir}/me" --help
```

## Artifact Verification

macOS:

```bash
tmpdir="$(mktemp -d)"
curl -L -o "${tmpdir}/me.tar.gz" https://github.com/harveyTon/me/releases/download/v0.1.1/me-v0.1.1-macos-arm64.tar.gz
tar -C "${tmpdir}" -xzf "${tmpdir}/me.tar.gz"
"${tmpdir}/me" --help
```

Linux:

```bash
tmpdir="$(mktemp -d)"
curl -L -o "${tmpdir}/me.tar.gz" https://github.com/harveyTon/me/releases/download/v0.1.1/me-v0.1.1-linux-x64.tar.gz
tar -C "${tmpdir}" -xzf "${tmpdir}/me.tar.gz"
"${tmpdir}/me" --help
```

Windows PowerShell:

```powershell
$tmpdir = New-Item -ItemType Directory -Force "$env:TEMP\me-v0.1.1"
Invoke-WebRequest -Uri https://github.com/harveyTon/me/releases/download/v0.1.1/me-v0.1.1-windows-x64.zip -OutFile "$tmpdir\me.zip"
Expand-Archive "$tmpdir\me.zip" -DestinationPath $tmpdir -Force
& "$tmpdir\me.exe" --help
```

## Homebrew

- Generate the release tarball SHA:

```bash
curl -L https://github.com/harveyTon/me/archive/refs/tags/v0.1.1.tar.gz | shasum -a 256
```

- Update `Formula/me.rb` with the real SHA.
- Publish it to `harveyTon/homebrew-me`.
- Follow `HOMEBREW.md` for the exact tap setup and install verification steps.

## Final User Install Commands

Git:

```bash
cargo install --git https://github.com/harveyTon/me --tag v0.1.1
```

GitHub Releases installer on macOS/Linux:

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/harveyTon/me/main/scripts/install.sh)
```

Pinned version:

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/harveyTon/me/main/scripts/install.sh) -- v0.1.1
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
