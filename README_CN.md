[English](README.md) | **中文**

# me

一个更好的 `whoami`，但带上上下文。

## 安装

在 macOS 上最快的方式是 Homebrew；其他平台直接使用 release 二进制即可。

### macOS (Homebrew)

```bash
brew tap harveyTon/me
brew install me
```

### macOS / Linux（一行安装）

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/harveyTon/me/main/scripts/install.sh)
```

发布页面：[github.com/harveyTon/me/releases](https://github.com/harveyTon/me/releases)

其他平台请直接使用发布页中的 release 二进制。

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
- Docker Compose

多个项目信号可以同时存在，但默认输出会保持克制而安静。

完整上下文可以通过 JSON 获取。

## 其他模式

Compact：

```bash
me --compact
```

JSON：

```bash
me --json
```

检查更新：

```bash
me update --check
```

## Shell 集成

把 `me` 接入当前 shell：

```bash
me install
```

之后也可以移除：

```bash
me uninstall
```

更新 `me` 本身：

```bash
me update
```

一个很小的工具，但在日常 shell 工作里会意外地好用。
