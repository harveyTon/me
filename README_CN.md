[English](README.md) | **中文**

# me

一个更好的 shell `whoami` 替代工具：告诉你你是谁、你在哪、你正在做什么。

## 安装

### macOS (Homebrew)

```bash
brew tap harveyTon/me
brew install me
```

### 其他平台

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/harveyTon/me/main/scripts/install.sh)
```

发布页面：[github.com/harveyTon/me/releases](https://github.com/harveyTon/me/releases)

### 从源码构建

```bash
git clone https://github.com/harveyTon/me.git
cd me
cargo build --locked --release
```

```bash
target/release/me
```

## 示例

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

`whoami` 只告诉你 *who you are*。

`me` 会告诉你：
- 你是谁
- 你在哪（`pwd`）
- 你处在什么环境里（`ssh` / `sudo`）
- 你正在做什么项目（`rust` / `node` / `python` / ...）

## 项目上下文

`me` 会自动检测常见项目类型：

- Rust
- Node（`pnpm` / `yarn` / `npm` / `turbo` / `nx`）
- Python（包含 virtualenv）
- Go
- Java（`Maven` / `Gradle`）
- Ruby
- C / C++
- PHP
- Lua
- Swift
- R
- C#

多个项目信号可以同时存在。

默认文本输出会保持克制，但完整上下文可以通过 JSON 获取。

## 其他模式

Compact：

```bash
me --compact
```

JSON：

```bash
me --json
```

一个很小的工具，但在日常 shell 工作里会意外地好用。
