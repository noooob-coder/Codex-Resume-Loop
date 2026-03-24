# CRL

CRL 用来管理 Codex 多工作区任务。

- Windows：桌面 UI + CLI
- Linux：CLI

核心目标只有一个：
安装完成后，直接输入 `crl` 就能用。

## 依赖

运行依赖：

- `codex` 已安装并且在 PATH 中可执行
- 能访问 `~/.codex`
- Windows 桌面端建议安装 WebView2 Runtime

构建或打包依赖：

- Windows 安装包：Inno Setup 6
- Linux CLI 打包：`zig`、`cargo-zigbuild`

## 下载地址

- 仓库首页：`https://github.com/noooob-coder/Codex-Resume-Loop`
- Release 页面：`https://github.com/noooob-coder/Codex-Resume-Loop/releases/tag/v0.1.0`

## 当前发布物

- Windows 安装包：`crl-setup-windows-x64-0.1.0.exe`
- Linux CLI：`crl-cli-linux-x86_64.tar.gz`

## Windows 安装与使用

### 安装步骤

1. 打开 Release 页面。
2. 下载 `crl-setup-windows-x64-0.1.0.exe`。
3. 运行安装包。
4. 安装时保持“添加 CRL CLI 到 PATH”选项开启。
5. 安装完成后，关闭当前终端，再打开一个新的终端窗口。

### 命令行下载并启动安装包

```powershell
$installer = "$env:TEMP\\crl-setup-windows-x64-0.1.0.exe"
Invoke-WebRequest `
  -Uri "https://github.com/noooob-coder/Codex-Resume-Loop/releases/download/v0.1.0/crl-setup-windows-x64-0.1.0.exe" `
  -OutFile $installer
Start-Process $installer
```

### 安装后怎么验证

先只做最简单的验证：

```powershell
crl
```

如果终端已经能直接识别 `crl`，说明安装和 PATH 已经生效。

需要看帮助时再执行：

```powershell
crl --help
```

### Windows 上最常见的用法

直接进入当前项目目录后输入：

```powershell
crl
```

这会进入交互式选择和执行流程，是最推荐的入门方式。

## Linux 安装与使用

### 一条命令安装

```bash
tmpdir="$(mktemp -d)" && \
curl -L "https://github.com/noooob-coder/Codex-Resume-Loop/releases/download/v0.1.0/crl-cli-linux-x86_64.tar.gz" | tar -xzf - -C "$tmpdir" && \
chmod +x "$tmpdir/install.sh" && \
"$tmpdir/install.sh"
```

### 安装后怎么验证

```bash
crl
```

如果 shell 已经能直接找到 `crl`，说明安装成功。

需要看帮助时再执行：

```bash
crl --help
```

### Linux 上最常见的用法

进入你的项目目录后直接执行：

```bash
crl
```

## 桌面端怎么用

Windows 桌面端分三栏：

- 左栏：工作区列表与切换
- 中栏：任务控制、会话选择、提示词、运行反馈
- 右栏：命令输出

基本流程：

1. 顶部设置 `Codex` 目录
2. 添加一个或多个项目目录
3. 左侧选择目标工作区
4. 中间栏选择目标会话
5. 设置执行轮次
6. 填写提示词
7. 点击启动
8. 右侧实时观察输出

## CLI 怎么用

最简单的方式就是：

```bash
crl
```

推荐先记住这一条。

只有在你已经熟悉流程之后，再考虑这些进阶命令：

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

## 重复机制

这是 CRL 最核心的行为。

### 1. 重复的本质

CRL 不是每轮都新建一个全新任务，而是反复恢复同一个 Codex 会话。

每一轮本质都在执行：

```text
codex exec resume --skip-git-repo-check <session_id> <wrapped_prompt>
```

也就是说，重复机制的重点不是“重复执行命令”本身，而是：

- 同一个目标会话
- 同一个提示词目标
- 多轮持续恢复

### 2. 哪三个字段决定重复行为

- `目标对话`
  说明：恢复哪个 Codex 会话
- `执行轮次`
  说明：总共重复多少轮
- `提示词`
  说明：每一轮继续发给同一个会话的任务要求

例如：

- 目标会话：`session-a`
- 执行轮次：`3`
- 提示词：`继续上一次结束的位置，完成未完成的工作。`

那就表示对同一个 `session-a` 连续恢复 3 轮。

### 3. CLI 里的重复

CLI 里最简单的入口是：

```bash
crl
```

如果你需要明确轮次，才写成：

```bash
crl 3 "继续上一次结束的位置，完成未完成的工作。"
```

这里的 `3` 就是重复轮次。

### 4. 提示词不会被原样裸发

CRL 会把你的原始提示词包装成一个更严格的恢复提示。

包装后的要求包括：

- 从上一次停止的位置继续
- 不要再问“要不要继续”
- 完成后自己检查有没有遗漏
- 只有遇到真实阻塞才允许提前停

所以重复机制不是简单 for-loop，而是“带执行约束的会话恢复循环”。

### 5. 默认轮次

当前默认轮次是 `1`。

如果你不主动把轮次调大，CRL 默认只恢复一轮。

### 6. 某一轮失败时会发生什么

当前行为不是“第一轮失败就全部停止”，而是：

1. 记录失败轮次和退出码
2. 继续尝试后面的轮次
3. 全部轮次结束后统一汇总失败情况

所以它更偏向“把计划轮次尽量跑完”。

### 7. 为什么实时输出重要

命令输出现在是实时流式显示。

这对重复机制很重要，因为你能立刻知道：

- 当前轮有没有开始
- Codex 有没有开始输出
- 当前轮是不是卡住
- 某轮失败后有没有继续进入下一轮

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
