# CRL

CRL 用来管理 Codex 多工作区任务。

- Windows：桌面 UI + CLI
- Linux：CLI
- iOS：提供 UI + CLI 构建套件，需在 macOS + Xcode 上完成最终构建

## 依赖

- `codex` 已安装并可执行
- 可访问 `~/.codex`
- Windows 桌面端建议安装 WebView2 Runtime
- Linux 交叉打包需要 `zig` 和 `cargo-zigbuild`
- iOS 构建需要 macOS、Xcode、`xcrun`

## 发布版怎么用

仓库地址：

- `https://github.com/noooob-coder/Codex-Resume-Loop`
- Releases 页面：`https://github.com/noooob-coder/Codex-Resume-Loop/releases`

Windows：

1. 下载 `crl-setup-windows-x64-0.1.0.exe`
2. 运行安装包
3. 安装完成后直接在终端输入：

```powershell
crl --help
```

Linux：

1. 下载 `crl-cli-linux-x86_64.tar.gz`
2. 解压
3. 进入解压目录执行：

```bash
chmod +x install.sh
./install.sh
crl --help
```

iOS：

1. 下载 `crl-ios-ui-and-cli-build-kit.tar.gz`
2. 在 macOS 上解压
3. 运行 `packaging/ios/build-ui-and-cli.sh`

## 日常使用

列出当前目录可恢复的会话：

```bash
crl --list-sessions
```

执行指定轮次：

```bash
crl 3 "继续上一次结束的位置，完成未完成的工作。"
```

只看计划，不真正执行：

```bash
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
