# Codex-Resume-Loop

Codex-Resume-Loop 是一个用于管理 Codex 多工作区任务的工具，简称 `CRL`。

- Windows：桌面端 + 命令行
- Linux：命令行

Windows 和 Linux 的发布版都以同一个目标收口：安装完成后，直接输入 `crl` 就能使用。

## 运行依赖

- `codex` 已安装，并且可以在命令行里直接执行
- 当前用户可以访问 `~/.codex`

## 下载地址

- [仓库首页](https://github.com/noooob-coder/Codex-Resume-Loop)
- [Release 页面](https://github.com/noooob-coder/Codex-Resume-Loop/releases/tag/v0.1.0)

## 当前发布物

- Windows 安装包：`crl-setup-windows-x64-0.1.0.exe`
- Linux CLI：`crl-cli-linux-x86_64.tar.gz`

GitHub Release 页面里还会显示 `Source code (zip)` 和 `Source code (tar.gz)`。这两项是 GitHub 针对 tag 自动生成的系统下载入口，不是项目主动发布的资产，也不能关闭。

## Windows 安装

1. 打开 [Release 页面](https://github.com/noooob-coder/Codex-Resume-Loop/releases/tag/v0.1.0)。
2. 下载 `crl-setup-windows-x64-0.1.0.exe`。
3. 运行安装包。
4. 安装时保持“添加 Codex-Resume-Loop CLI 到 PATH”选项开启。
5. 安装完成后，关闭当前终端，再打开一个新的终端窗口。

如果你想用命令行下载并启动安装包：

```powershell
$installer = "$env:TEMP\\crl-setup-windows-x64-0.1.0.exe"
Invoke-WebRequest `
  -Uri "https://github.com/noooob-coder/Codex-Resume-Loop/releases/download/v0.1.0/crl-setup-windows-x64-0.1.0.exe" `
  -OutFile $installer
Start-Process $installer
```

安装完成后，先做最简单的验证：

```powershell
crl
```

如果终端已经能直接识别 `crl`，说明安装和 PATH 已经生效。需要帮助输出时再执行：

```powershell
crl --help
```

最常见的用法就是进入目标项目目录后直接执行：

```powershell
crl
```

这里有一个必须明确的前提：`crl` 必须在目标项目目录里运行。它会根据当前目录去发现这个目录下可恢复的 Codex 会话。如果不在对应项目目录里执行，就不会匹配到你想继续的会话。

桌面端启动入口：

- 开始菜单中的 `Codex-Resume-Loop Desktop`
- 安装目录里的 `crl-desktop.exe`

安装包支持完整卸载，不是只能安装的单向封装。卸载入口包括：

- Windows 设置中的“应用”
- 控制面板中的“程序和功能”
- 安装器创建的卸载入口

卸载时会出现一个选项：

- 是否同时删除本地状态和历史记录

如果勾选，会一并删除本机上的配置状态和日志。

## Linux 安装

Linux 发布版安装完成后也应当直接使用 `crl`。

一条命令安装：

```bash
tmpdir="$(mktemp -d)" && \
curl -L "https://github.com/noooob-coder/Codex-Resume-Loop/releases/download/v0.1.0/crl-cli-linux-x86_64.tar.gz" | tar -xzf - -C "$tmpdir" && \
chmod +x "$tmpdir/install.sh" && \
"$tmpdir/install.sh"
```

安装完成后，先做最简单的验证：

```bash
crl
```

如果 shell 已经能直接找到 `crl`，说明安装成功。需要帮助输出时再执行：

```bash
crl --help
```

Linux 上最常见的用法同样是进入目标项目目录后直接执行：

```bash
crl
```

和 Windows 一样，这里也必须在目标项目目录中执行，才能正确发现当前目录下对应的 Codex 会话。

## 重复机制

重复机制是 Codex-Resume-Loop 的核心能力。

它不是每轮都新建一个全新任务，而是反复恢复同一个 Codex 会话。

每一轮本质都在执行：

```text
codex exec resume --skip-git-repo-check <session_id> <wrapped_prompt>
```

重复机制真正依赖的是三个输入：

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

提示词包装行为是真实存在的。

程序不会把原始提示词直接裸发给 Codex，而是先包装成更严格的恢复提示，再传给 `codex exec resume`。当前包装后的要求包括：

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
