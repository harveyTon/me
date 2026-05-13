# Homebrew Tap

Tap repository:

```txt
https://github.com/harveyTon/homebrew-me
```

Homebrew maps that repository to the tap name:

```txt
harveyTon/me
```

User install command:

```bash
brew tap harveyTon/me
brew install me
```

Standardized Homebrew distribution path:

- Tap repository: `harveyTon/homebrew-me`
- Tap name: `harveyTon/me`
- Formula path: `Formula/me.rb`

## Automatic Updates

The `me` release workflow updates this tap automatically after a GitHub Release is published.
The generated formula prefers prebuilt release archives on supported macOS and Linux targets,
and falls back to `cargo install` only when no matching binary path is available.

Required secret in `harveyTon/me`:

```txt
HOMEBREW_TAP_TOKEN
```

Use a fine-grained token with contents write access to `harveyTon/homebrew-me`.

## Manual Fallback

1. Create the tap repository:

```bash
gh repo create harveyTon/homebrew-me --public
```

2. Clone the tap:

```bash
git clone https://github.com/harveyTon/homebrew-me.git
cd homebrew-me
mkdir -p Formula
```

3. Generate the source tarball SHA after the release tag is pushed:

```bash
VERSION="vX.Y.Z"
curl -L "https://github.com/harveyTon/me/archive/refs/tags/${VERSION}.tar.gz" | shasum -a 256
```

4. Update `Formula/me.rb` with:

- the source tarball `url` and `sha256`
- the matching release archive URLs and SHA256 values for macOS/Linux targets

5. Tap, audit, and install locally:

```bash
brew tap harveyTon/me
brew audit --strict --online harveyTon/me/me
brew install me
```

6. Commit and push:

```bash
git add Formula/me.rb
git commit -m "Add me formula"
git push origin main
```

After that, users can install with:

```bash
brew tap harveyTon/me
brew install me
```
