# CRL

CRL 是一个用于管理 Codex 多工作区任务的工具。

- Windows：支持桌面 UI 和 CLI
- Linux：支持 CLI
- iOS：提供 UI + CLI 构建脚本，需在 macOS + Xcode 上完成最终构建

## 依赖

- `codex` 已安装并可执行
- 可访问 `~/.codex`
- Windows 桌面端建议安装 WebView2 Runtime
- 交叉发布 Linux CLI 需要 `zig` 和 `cargo-zigbuild`
- 构建 iOS 版本需要 macOS、Xcode、`xcrun`

## 怎么用

桌面端：

```powershell
cargo run --bin crl-desktop
```

CLI：

```powershell
cargo run --bin crl -- --help
```

列出当前目录可恢复的会话：

```powershell
crl --list-sessions
```

执行指定轮次：

```powershell
crl 3 "继续上一次结束的位置，完成未完成的工作。"
```

只看计划：

```powershell
crl --dry-run 3 "继续上一次结束的位置，完成未完成的工作。"
```

## 桌面端界面

- 左栏：工作区列表与切换
- 中栏：任务控制、会话选择、提示词、运行反馈
- 右栏：命令输出

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

## 发布产物

- Windows 安装包：`dist/crl-setup-windows-x64-0.1.0.exe`
- Linux CLI：`dist/crl-cli-linux-x86_64.tar.gz`
- iOS 构建套件：`dist/crl-ios-ui-and-cli-build-kit.tar.gz`

## 开发检查

```powershell
cargo test
cargo check
```
