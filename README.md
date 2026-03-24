# CRL

CRL 用来管理 Codex 多工作区任务。

- Windows：桌面 UI + CLI
- Linux：CLI
- iOS：提供 UI + CLI 构建套件，需在 macOS + Xcode 上完成最终构建

## 依赖

运行依赖：

- `codex` 已安装并且在 PATH 中可执行
- 能访问 `~/.codex`
- Windows 桌面端建议安装 WebView2 Runtime

构建或打包依赖：

- Windows 安装包：当前仓库内置 `IExpress` 打包脚本
- Linux CLI 打包：`zig`、`cargo-zigbuild`
- iOS 构建：macOS、Xcode、`xcrun`、Apple SDK

## 下载地址

- 仓库首页：`https://github.com/noooob-coder/Codex-Resume-Loop`
- Releases 页面：`https://github.com/noooob-coder/Codex-Resume-Loop/releases`

## 发布产物

- Windows 安装包：`crl-setup-windows-x64-0.1.0.exe`
- Linux CLI：`crl-cli-linux-x86_64.tar.gz`
- iOS 构建套件：`crl-ios-ui-and-cli-build-kit.tar.gz`

Windows 版本只保留一个安装包，安装后同时提供桌面端和 `crl` 命令。

## Windows 安装与使用

1. 从 Releases 页面下载 `crl-setup-windows-x64-0.1.0.exe`。
2. 运行安装包。
3. 安装完成后，重新打开一个新的终端窗口。
4. 先确认 CLI 已经生效：

```powershell
crl --help
```

5. 需要桌面端时，直接运行安装目录里的 `crl-desktop.exe`，或者通过开始菜单启动。

Windows 常用命令：

```powershell
crl --list-sessions
crl 3 "继续上一次结束的位置，完成未完成的工作。"
crl --dry-run 3 "继续上一次结束的位置，完成未完成的工作。"
```

## Linux 安装与使用

1. 从 Releases 页面下载 `crl-cli-linux-x86_64.tar.gz`。
2. 解压：

```bash
tar -xzf crl-cli-linux-x86_64.tar.gz
cd <解压目录>
```

3. 安装成全局可调用的 `crl`：

```bash
chmod +x install.sh
./install.sh
```

4. 安装后确认：

```bash
crl --help
```

Linux 常用命令：

```bash
crl --list-sessions
crl 3 "继续上一次结束的位置，完成未完成的工作。"
crl --dry-run 3 "继续上一次结束的位置，完成未完成的工作。"
```

## iOS 构建与使用

1. 下载 `crl-ios-ui-and-cli-build-kit.tar.gz`。
2. 在 macOS 上解压。
3. 确认本机已有 Xcode 和 `xcrun`。
4. 执行：

```bash
cd <解压目录>
chmod +x packaging/ios/build-ui-and-cli.sh
./packaging/ios/build-ui-and-cli.sh
```

说明：

- 当前 Windows 环境只能准备 iOS 构建套件，不能直接链接出最终 iOS 二进制
- iOS 最终 UI + CLI 产物必须在 macOS 上完成构建

## 桌面端怎么用

1. 顶部设置 `Codex` 目录。
2. 添加一个或多个项目目录。
3. 左侧选择工作区。
4. 中间栏选择目标会话、轮次和提示词。
5. 点击启动。
6. 右侧查看实时输出。

界面分区：

- 左栏：工作区列表与切换
- 中栏：任务控制、会话选择、提示词、运行反馈
- 右栏：命令输出

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

## 项目组成

- `src/desktop.rs`：桌面控制器
- `src/codex.rs`：Codex 启动与会话发现
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
