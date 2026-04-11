[English](README.md) | **中文**

# me

一个现代化的、上下文感知的 `whoami` 替代工具。

`me` 保持了 Unix 工具的风格：小巧、快速、本地优先、可脚本化。它添加了足够的结构信息来回答"我在这里是谁？"，而不会变成一个系统清单工具。

## 快速示例

```txt
user@dev-machine  zsh

groups:     staff, admin, _developer (+3)
privilege:  user
ssh:        no
network:    192.168.0.10 (+2)

context:    rust (rustc 1.94.0)
```

## 安装

### macOS (Homebrew)

```bash
brew tap harveyTon/me
brew install me
```

### 下载二进制文件

发布页面: [github.com/harveyTon/me/releases](https://github.com/harveyTon/me/releases)

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/harveyTon/me/main/scripts/install.sh)
```

### 从源码构建

```bash
git clone https://github.com/harveyTon/me.git
cd me
cargo build --locked --release
```

二进制文件:

```bash
target/release/me
```

### 验证安装

```bash
me
me --compact
```

## 为什么需要 `me`

`whoami` 在你只需要用户名时很好用:

```console
$ whoami
user
```

`me` 保持了同样的简洁，但补充了我通常紧接着需要查看的信息:

```console
$ me
user@dev-machine  zsh
...
ssh:        no
network:    192.168.0.10 (+2)
context:    rust (rustc 1.94.0)
```

目标仍然是小巧和本地化：身份信息优先，上下文信息其次，不会演变成通用系统检测工具。

## 示例

### SSH 会话

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

### 精简模式

```txt
user@dev-machine · zsh · user · local · rust
```

## 使用方法

```bash
me
me --compact
me --json
me --help
```

字段过滤:

```bash
me -u
me -u -h
me -n
```

其他模式:

```bash
me --plain
me --format config
me --watch
me --full
```

## 输出模式

### 默认模式

人性化块状输出:

```bash
me
```

### 精简模式

单行输出，适用于提示符和快速查看:

```bash
me --compact
```

```txt
user@dev-machine · zsh · user · local · rust
```

### JSON 模式

机器可读输出，适用于脚本:

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

## 上下文感知

`me` 自动检测一组本地上下文信号:

- SSH 会话
- Docker/容器环境
- Rust 项目（通过 `Cargo.toml`）
- Node 项目（通过 `package.json`）

上下文作为辅助信号展示:

```txt
context:    rust (rustc 1.94.0)
```

默认命令保持本地优先。它不会查询云端身份、检查远程服务或执行公网 IP 查询。

## Shell 集成

### zsh

```zsh
PROMPT='$(me --compact) %~ %# '
```

### bash

```bash
PS1='$(me --compact) \w \$ '
```

提示命令会频繁执行。如果你的提示符感觉变慢了，请将 `me --compact` 移出热路径，或在 shell 配置中缓存其输出。

## 配置

配置文件:

```txt
~/.config/me/config.yaml
```

`me` 在首次运行时会创建默认配置（如果不存在）。如果文件无效，`me` 会打印警告并回退到内置默认值。

最小配置示例:

```yaml
view: block
icons: auto

context:
  enabled: true
  project: true
  container: true
  ssh: true
```

优先级: CLI 参数 > 环境变量 > 配置文件 > 默认值。

## 设计理念

`me` 以身份信息优先。

默认输出应该快速回答常见问题: 我是谁、在哪台机器上、在什么会话中？

上下文信息有用，但不应主导输出。它作为安静的辅助信号出现在主要身份和状态字段之后。

没有繁重的 UI、没有守护进程、没有插件系统。默认输出是纯文本，在任何终端中都应保持可读性，无论是否支持颜色。

## 发布

当前版本为 `v0.1.2`。版本号遵循语义化版本规范。

构建本地发布产物:

```bash
scripts/build.sh
```

GitHub 发布工作流会构建所有发布产物。本地脚本默认构建主机支持的目标；传入特定的 Rust target 可构建单个产物。

Linux ARM64 交叉编译在主机不是 Linux ARM64 时使用 `cross`。Windows MSVC 产物在 Windows 运行器上构建。

创建发布标签:

```bash
git tag -a v0.1.2 -m "Release v0.1.2"
git push origin v0.1.2
```

GitHub Actions 发布工作流会构建并上传:

- `me-v0.1.2-macos-arm64.tar.gz`
- `me-v0.1.2-macos-x64.tar.gz`
- `me-v0.1.2-linux-x64.tar.gz`
- `me-v0.1.2-linux-arm64.tar.gz`
- `me-v0.1.2-windows-x64.zip`
- `me-v0.1.2-windows-arm64.zip`

完整的发布清单请参见 `RELEASE_CHECKLIST.md`。

## 路线图

- 改进默认块状输出，消除小重复，保持身份摘要的主体地位。
- 优化 `--compact` 用于提示符场景，保持输出在常见 shell 环境中的稳定性。
- 改善本地、SSH、容器和非交互会话之间的输出一致性。
- 优化提示符场景的启动时间，为频繁调用添加更轻量的快速路径。
- 改进现有项目检测，在安静且有用的地方添加轻量级 Git 分支上下文。
- 改进轻量级容器检测，不将 `me` 变成通用系统检测工具。
- 保持发布二进制文件、Homebrew 分发和安装脚本在各平台上的一致性。
- 扩展快照覆盖率以保证输出稳定性，保持 CLI 帮助和使用示例简洁。

## 许可证

[MIT](LICENSE)
