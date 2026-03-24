# Codex-Resume-Loop

Codex-Resume-Loop 是一个用于管理 Codex 多工作区任务的工具，简称 `CRL`。

- Windows：桌面 UI + CLI
- Linux：CLI

发布版的目标很明确：安装完成后，在目标项目目录里直接输入 `crl` 就能开始工作。

## 运行依赖

- `codex` 已安装，并且可以在命令行里直接执行
- 当前用户可以访问 `~/.codex`

## 下载地址

- [仓库首页](https://github.com/noooob-coder/Codex-Resume-Loop)
- [Release 页面](https://github.com/noooob-coder/Codex-Resume-Loop/releases/tag/v0.1.0)

## 当前发布物

- Windows 安装包：`crl-setup-windows-x64-0.1.0.exe`
- Linux CLI：`crl-cli-linux-x86_64.tar.gz`

GitHub Release 页面里还会出现 `Source code (zip)` 和 `Source code (tar.gz)`。那两项是 GitHub 针对 tag 自动生成的下载入口，不是这个项目主动发布的资产，不能在 GitHub Release 页面里关闭。当前项目主动发布的资产只有上面两个。

## Windows 安装

1. 打开 [Release 页面](https://github.com/noooob-coder/Codex-Resume-Loop/releases/tag/v0.1.0)。
2. 下载 `crl-setup-windows-x64-0.1.0.exe`。
3. 运行安装包。
4. 安装时保持“添加 Codex-Resume-Loop CLI 到 PATH”选项开启。
5. 安装完成后，关闭当前终端，再打开一个新的终端窗口。

命令行下载并启动安装包：

```powershell
$installer = "$env:TEMP\\crl-setup-windows-x64-0.1.0.exe"
Invoke-WebRequest `
  -Uri "https://github.com/noooob-coder/Codex-Resume-Loop/releases/download/v0.1.0/crl-setup-windows-x64-0.1.0.exe" `
  -OutFile $installer
Start-Process $installer
```

安装生效验证：

```powershell
crl
```

如果终端已经能直接识别 `crl`，说明安装和 PATH 已经生效。

帮助输出查看：

```powershell
crl --help
```

桌面端启动入口：

- 开始菜单中的 `Codex-Resume-Loop Desktop`
- 安装目录里的 `crl-desktop.exe`

卸载方式：

- Windows 设置里的“应用和功能”
- 控制面板里的“程序和功能”
- 安装目录中 Inno Setup 自动生成的卸载程序

这次使用的是 Inno Setup 安装器，所以安装包本身带完整卸载功能，不是只能安装、不能卸载的单向封装。

## Linux 安装

Linux 发布版安装完成后也应当直接使用 `crl`。

一条命令安装：

```bash
tmpdir="$(mktemp -d)" && \
curl -L "https://github.com/noooob-coder/Codex-Resume-Loop/releases/download/v0.1.0/crl-cli-linux-x86_64.tar.gz" | tar -xzf - -C "$tmpdir" && \
chmod +x "$tmpdir/install.sh" && \
"$tmpdir/install.sh"
```

安装生效验证：

```bash
crl
```

如果 shell 已经能直接找到 `crl`，说明安装成功。

帮助输出查看：

```bash
crl --help
```

## 最常见的使用方式

无论 Windows 还是 Linux，最常见的方式都是：

1. 先进入目标项目目录
2. 再直接输入：

```bash
crl
```

这里的前提很重要：`crl` 必须在目标项目目录里运行。它会基于当前目录去发现这个目录下可恢复的 Codex 会话。如果不在相应项目目录里执行，通常不会匹配到你想继续的会话。

## 重复机制

重复机制是 Codex-Resume-Loop 的核心能力。

它不是每轮都新建一个全新任务，而是反复恢复同一个 Codex 会话。

每一轮本质都在执行：

```text
codex exec resume --skip-git-repo-check <session_id> <wrapped_prompt>
```

重复机制真正依赖的是这三个输入：

- `目标对话`
  说明：恢复哪个 Codex 会话
- `执行轮次`
  说明：总共重复多少轮
- `提示词`
  说明：每一轮继续发给同一个会话的任务要求

例子：

- 目标会话：`session-a`
- 执行轮次：`3`
- 提示词：`继续上一次结束的位置，完成未完成的工作。`

这表示对同一个 `session-a` 连续恢复 3 轮。

`crl` 直接启动时会进入交互式流程，让你在当前目录下选择会话、轮次和提示词；如果你已经非常明确，也可以直接写成：

```bash
crl 3 "继续上一次结束的位置，完成未完成的工作。"
```

提示词包装行为是真实存在的，不是文档猜测。

代码里会先调用 `build_resume_prompt()`，把你的原始提示词包装成更严格的恢复提示，再传给 `codex exec resume`。当前包装后的要求包括：

- 从上一次停止的位置继续
- 不要再问“要不要继续”
- 完成后自己检查有没有遗漏
- 只有遇到真实阻塞才允许提前停

默认轮次：

- 当前默认轮次是 `1`

失败轮次处理：

1. 记录失败轮次和退出码
2. 继续尝试后面的轮次
3. 全部轮次结束后统一汇总失败情况

这意味着它更偏向“把计划轮次尽量跑完”。

实时输出的作用：

命令输出现在是实时流式显示。这样你可以立刻知道：

- 当前轮有没有开始
- Codex 有没有开始输出
- 当前轮是不是卡住
- 某轮失败后有没有继续进入下一轮

## 进阶命令

列出当前目录可恢复的会话：

```bash
crl --list-sessions
```

执行指定轮次：

```bash
crl 3 "继续上一次结束的位置，完成未完成的工作。"
```

只看计划：

```bash
crl --dry-run 3 "继续上一次结束的位置，完成未完成的工作。"
```

## 项目组成

- `src/desktop.rs`：桌面控制器
- `src/codex.rs`：Codex 启动、会话发现、恢复提示词构造
- `src/runtime.rs`：后台运行时与实时输出
- `src/model.rs`：状态模型
- `src/persistence.rs`：本地状态持久化
- `src/bin/crl.rs`：CLI 入口
- `ui/main.slint`：桌面 UI
- `packaging/windows`：Windows 安装包脚本
- `packaging/linux`：Linux CLI 打包脚本

## 开发检查

```powershell
cargo test
cargo check
```
