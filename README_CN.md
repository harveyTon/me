[English](README.md) | **中文**

# me

一个更好的 `whoami`，但带上上下文。

`me` 仍然保持 Unix 小工具的形状：轻量、快速、本地优先、适合脚本。它只增加刚好够用的结构，来回答“我现在在这里是谁？”而不是把自己做成系统盘点工具。

## 快速示例

```txt
identity:
  user:   user
  host:   dev-machine
  shell:  zsh

system:
  uid:     501
  gid:     20
  groups:  staff, admin, _developer (+2)
  pid:     24811
  ppid:    24803
  tty:     ttys001

session:
  privilege:  user
  sudo:       no
  ssh:        no

network:
  ipv4:  192.168.0.10 (+2)
  ipv6:  fd12::10 (+1)

location:
  pwd:      /Users/user/dev/me
  context:  rust 1.94.1 · git(main)
```

## 安装

在 macOS 上最快的方式是 Homebrew；其他平台直接使用 release 二进制即可。

### macOS（推荐 Homebrew）

```bash
brew tap harveyTon/me
brew install me
```

### macOS / Linux（一行安装）

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/harveyTon/me/main/scripts/install.sh)
```

### 下载 release 二进制

发布页面：[github.com/harveyTon/me/releases](https://github.com/harveyTon/me/releases)

预编译归档支持 macOS、Linux 和 Windows。

### 从源码构建

```bash
git clone https://github.com/harveyTon/me.git
cd me
cargo build --locked --release
```

二进制位置：

```bash
target/release/me
```

### 验证安装

```bash
me
me --compact
```

## 为什么是 `me`

`whoami` 很好用，但它只回答最基本的那个问题：

```console
$ whoami
user
```

`me` 保留了这种小工具的使用方式，但继续回答后面的几个问题：

- 我是谁？
- 我现在在哪个目录（`pwd`）？
- 我是在 SSH、`sudo` 还是容器环境里？
- 我当前在做什么项目（`rust`、`node`、`python` 等）？

范围仍然很克制：身份优先，上下文其次，顺带给出位置，不会漂移成通用系统信息面板。

## 示例

### SSH 会话

```txt
identity:
  user:   user
  host:   server-01
  shell:  bash

system:
  uid:     1000
  gid:     1000
  groups:  user, deploy
  pid:     24811
  ppid:    24803
  tty:     pts/0

session:
  privilege:  user
  sudo:       no
  ssh:        yes

network:
  ipv4:  10.0.0.5

location:
  pwd:      /srv/app
  context:  rust 1.94.1 · git(main)
```

### sudo

```txt
identity:
  user:   root
  host:   server-01
  shell:  bash

system:
  uid:     0
  gid:     0
  groups:  root
  pid:     24902
  ppid:    24890
  tty:     pts/0

session:
  privilege:  root
  sudo:       yes
  ssh:        yes

network:
  ipv4:  10.0.0.5

location:
  pwd:      /srv/app
  context:  rust 1.94.1 · git(main)
```

### Compact 模式

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

## 用法

```bash
me
me --compact
me --json
me update --check
me --help
```

字段筛选：

```bash
me -u
me -u -h
me -n
```

其他模式：

```bash
me --plain
me --format config
me --watch
me --compact --fast
me --full
```

## 输出模式

### 默认模式

适合人读的分组 block 输出：

```bash
me
```

### Compact

适合 prompt 和快速查看的一行输出：

```bash
me --compact
```

```txt
user@dev-machine · local · rust 1.94.1 · git:main · me
```

### JSON

适合脚本处理的结构化输出：

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

## 上下文感知

`me` 会自动检测一小组本地上下文信号：

- SSH 会话
- Docker / container 环境
- Rust、Node、Python、Go、Java、Ruby、C/C++、PHP、Lua、Swift、R、C#、Docker Compose 项目上下文
- 当前目录位于 Git 工作树内时的分支信息

多个项目信号可以同时存在，但默认输出会保持克制而安静。
即使匹配到多个 detector，上下文在默认文本输出里仍然只是次级信号。

```txt
location:
  context:  node 24.14.1 (pnpm, turbo) · python 3.12 (.venv) · git(main) (+1)
```

默认命令保持本地优先：不会查询云身份、不会探测远端服务、也不会做公网 IP 查询。

## Shell 集成

把 `me` 接入你的 shell：

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

## 配置

配置文件：

```txt
~/.config/me/config.yaml
```

如果配置文件不存在，`me` 会在首次运行时创建默认配置。
如果配置文件无效，`me` 会打印一条简短警告并回退到内置默认值。
如果配置文件过大，`me` 也会打印一条简短警告并回退到内置默认值。

最小示例：

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

优先级顺序为：CLI 参数覆盖环境变量，环境变量覆盖配置文件，配置文件覆盖默认值。

## 设计原则

`me` 是 identity-first 的。

默认输出应该尽快回答最常见的问题：我是谁、我在哪台机器上、当前会话是什么状态。上下文当然有用，但不应该压过主体信息。它会作为一个低调的次级信号，出现在主要身份与状态字段之后。

没有重型 UI，没有守护进程，也没有插件系统。默认输出就是纯文本，并且应该在任何终端里都保持可读，无论是否开启颜色。

## Release

当前版本以 `Cargo.toml` 为准。发布遵循语义化版本。

GitHub release workflow 会构建以下平台的产物：

- macOS arm64
- macOS x64
- Linux x64
- Linux arm64
- Windows x64
- Windows arm64

本地可以用下面的命令构建 release 产物：

```bash
scripts/build.sh
```

详细发布流程见 [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md)。

## Roadmap

- 继续保持 block、compact、JSON 和 prompt 场景输出的稳定性。
- 在保持本地优先和低成本的前提下，逐步扩展 detector 覆盖面。
- 继续让 `--fast` 保持适合 prompt 的可预测启动开销。
- 保持 release 产物、Homebrew 分发和安装路径在各平台上的一致性。

## License

[MIT](LICENSE)
