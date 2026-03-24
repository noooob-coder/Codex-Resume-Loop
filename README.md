# CRL

CRL 用来管理 Codex 多工作区任务，并把同一个任务按轮次持续推进。

- Windows：桌面 UI + CLI
- Linux：CLI
- iOS：提供 UI + CLI 构建套件，最终构建需在 macOS + Xcode 上完成

## 依赖

运行依赖：

- `codex` 已安装并且在 PATH 中可执行
- 能访问 `~/.codex`
- Windows 桌面端建议安装 WebView2 Runtime

打包或构建依赖：

- Windows 安装包：Inno Setup 6
- Linux CLI 打包：`zig`、`cargo-zigbuild`
- iOS 构建：macOS、Xcode、`xcrun`、Apple SDK

## 下载地址

- 仓库首页：`https://github.com/noooob-coder/Codex-Resume-Loop`
- Release 页面：`https://github.com/noooob-coder/Codex-Resume-Loop/releases/tag/v0.1.0`

## 当前发布物

- Windows 安装包：`crl-setup-windows-x64-0.1.0.exe`
- Linux CLI：`crl-cli-linux-x86_64.tar.gz`
- iOS 构建套件：`crl-ios-ui-and-cli-build-kit.tar.gz`

Windows 版本当前只保留一个安装包。

## Windows 安装与使用

### 图形安装

1. 打开 Release 页面。
2. 下载 `crl-setup-windows-x64-0.1.0.exe`。
3. 运行安装包。
4. 安装时保持“添加 CRL CLI 到 PATH”选项开启。
5. 安装完成后，关闭当前终端，重新打开一个新的终端窗口。

### 命令行拉起安装包

```powershell
$installer = "$env:TEMP\\crl-setup-windows-x64-0.1.0.exe"
Invoke-WebRequest `
  -Uri "https://github.com/noooob-coder/Codex-Resume-Loop/releases/download/v0.1.0/crl-setup-windows-x64-0.1.0.exe" `
  -OutFile $installer
Start-Process $installer
```

### 安装后验证

安装完成后直接验证：

```powershell
crl --help
```

如果这条命令能直接输出帮助信息，说明你已经可以像本机一样直接使用 `crl`，不需要再写 `cargo run`。

### Windows 常用命令

```powershell
crl --list-sessions
crl 3 "继续上一次结束的位置，完成未完成的工作。"
crl --dry-run 3 "继续上一次结束的位置，完成未完成的工作。"
```

### 桌面端怎么打开

- 开始菜单里的 `CRL Desktop`
- 安装目录里的 `crl-desktop.exe`

## Linux 直接命令行安装

Linux 版本是 CLI-only，但安装完成后同样直接使用 `crl`。

### 一条命令安装

```bash
tmpdir="$(mktemp -d)" && \
curl -L "https://github.com/noooob-coder/Codex-Resume-Loop/releases/download/v0.1.0/crl-cli-linux-x86_64.tar.gz" | tar -xzf - -C "$tmpdir" && \
chmod +x "$tmpdir/install.sh" && \
"$tmpdir/install.sh"
```

### 安装后验证

```bash
crl --help
```

### Linux 常用命令

```bash
crl --list-sessions
crl 3 "继续上一次结束的位置，完成未完成的工作。"
crl --dry-run 3 "继续上一次结束的位置，完成未完成的工作。"
```

## iOS 直接命令行构建

iOS 不是在这台 Windows 机器上直接产出最终二进制，而是提供一个可直接命令行拉起的构建套件。

必须前提：

- 在 macOS 上执行
- 已安装 Xcode
- `xcrun` 可用

### 一条命令下载并构建

```bash
tmpdir="$(mktemp -d)" && \
curl -L "https://github.com/noooob-coder/Codex-Resume-Loop/releases/download/v0.1.0/crl-ios-ui-and-cli-build-kit.tar.gz" | tar -xzf - -C "$tmpdir" && \
chmod +x "$tmpdir/packaging/ios/build-ui-and-cli.sh" && \
"$tmpdir/packaging/ios/build-ui-and-cli.sh"
```

说明：

- 这会直接从 release 拉下 iOS 构建套件并执行构建脚本
- 最终 UI + CLI 构建仍然必须依赖 macOS + Xcode 的签名与链接环境

## 桌面端怎么用

桌面端主要分三栏：

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

CRL 不是每轮都新建一个全新上下文，而是反复恢复同一个 Codex 会话。

每一轮本质都在执行：

```text
codex exec resume --skip-git-repo-check <session_id> <wrapped_prompt>
```

所以“重复”指的是：

- 目标会话不变
- 提示词不变
- 轮次递增
- 会话持续被恢复

### 2. 哪三个字段决定重复行为

无论桌面端还是 CLI，本质上都由这三项决定：

- `目标对话`
  说明：恢复哪个 Codex 会话
- `执行轮次`
  说明：总共重复多少轮
- `提示词`
  说明：每一轮继续发送给同一个会话的任务要求

例如：

- 目标会话：`session-a`
- 执行轮次：`3`
- 提示词：`继续上一次结束的位置，完成未完成的工作。`

那么 CRL 会对同一个 `session-a` 连续恢复 3 轮。

### 3. CLI 里的重复

CLI 里最直接：

```bash
crl 3 "继续上一次结束的位置，完成未完成的工作。"
```

这里的 `3` 就是重复轮次。

### 4. 提示词会被二次包装

CRL 不会把你的提示词直接裸发给 Codex，而是先包装成一个更严格的恢复提示。

包装后的约束包括：

- 从上一次停止的位置继续
- 不要再问“要不要继续”
- 完成后自己检查有没有遗漏
- 只有遇到真实阻塞才允许提前停

所以重复机制不是简单循环，而是“带执行契约的会话恢复循环”。

### 5. 默认轮次

当前默认轮次是 `1`。

也就是说，如果你不主动把轮次调大，CRL 默认只恢复一轮。

### 6. 某一轮失败时会发生什么

当前行为不是“第一轮失败就全部停止”，而是：

1. 记录失败轮次和退出码
2. 继续尝试后面的轮次
3. 全部轮次结束后统一汇总失败情况

这意味着重复机制更偏向“把计划轮次尽量全部跑完”。

### 7. 为什么实时输出重要

命令输出已经改成按数据块实时显示，不再等一整行或者一个阶段结束才刷新。

这对重复机制很重要，因为你可以立刻知道：

- 当前轮是否已经开始
- Codex 是否正在输出
- 当前轮是否卡住
- 某轮失败后是否已经进入下一轮

### 8. 什么场景适合把轮次调大

适合：

- 任务规模较大，一轮不一定做完
- 你明确希望它持续推进同一个目标会话
- 你希望即使某轮失败，也继续尝试后续轮次

不适合：

- 提示词本身不稳定
- 当前目标会话上下文已经跑偏
- 你还没确认目标会话选对

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
- `packaging/ios`：iOS 构建脚本和说明

## 开发检查

```powershell
cargo test
cargo check
```
