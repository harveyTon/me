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

## Publish The Formula

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

3. Copy the formula:

```bash
cp ../me/Formula/me.rb Formula/me.rb
```

4. Generate the release tarball SHA after `v0.1.0` is pushed:

```bash
curl -L https://github.com/harveyTon/me/archive/refs/tags/v0.1.0.tar.gz | shasum -a 256
```

5. Replace the placeholder SHA in `Formula/me.rb`.

6. Audit and install locally:

```bash
brew audit --strict --online Formula/me.rb
brew install --build-from-source Formula/me.rb
```

7. Commit and push:

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
