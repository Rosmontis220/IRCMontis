# IRCMontis

一个面向 [Libera.Chat](https://libera.chat) 的 QQ 风格 IRC 桌面客户端。Rust 原生 IRC 引擎（TLS 加密 + SASL PLAIN 认证），Tauri 2 + Svelte 5 前端，简约机能风界面。

## 特性

- **原生 IRC 引擎**：Rust 实现，TLS 加密连接，SASL PLAIN 认证，断线按退避自动重连。
- **QQ 风格会话**：会话列表、消息气泡、identicon 头像、mIRC 颜色、`/me` 动作、可点击链接。
- **主题**：浅色 / 深色 / 跟随系统，支持自定义背景图与透明度。
- **托盘后台**：最小化到托盘，未读闪烁、提及徽章、按会话免打扰（滑动开关，仅当前会话生效）。
- **消息历史**：SQLite 存储，会话内搜索与跳转到结果。
- **输入辅助**：昵称 Tab 补全，Enter 发送，Shift+Enter 换行，表情面板。
- **双语界面**：中文 / 英文。
- **无边框窗口**：自定义标题栏，扁平哑光质感、细线边框、低饱和橄榄绿点缀。

## 下载

前往 [Releases](https://github.com/Rosmontis220/IRCMontis/releases/latest) 获取最新版本。

| 平台 | 资产 | 说明 |
| --- | --- | --- |
| Windows | `IRCMontis_<版本>_x64-setup.exe` | NSIS 安装版 |
| Windows | `IRCMontis_<版本>_x64_en-US.msi` | MSI 安装版 |
| Windows | `IRCMontis_<版本>_x64-portable.exe` | **便携版**，免安装，双击即可运行 |
| Linux | `IRCMontis_<版本>_amd64.AppImage` | AppImage |
| Linux | `IRCMontis_<版本>_amd64.deb` | Debian / Ubuntu |
| Linux | `IRCMontis-<版本>-1.x86_64.rpm` | Fedora / RHEL |

每个版本附带 `SHA256SUMS.txt`（SHA-256 校验和）与 `latest.json`（桌面自动更新元数据）。

**便携版说明**：免安装绿色版，依赖系统 WebView2 运行时（Windows 10/11 通常已自带）。便携版与安装版共享数据，均存于 `%APPDATA%\com.ircmontis.app\`，不会在 exe 旁生成配置文件。

> Android 工程配置已就绪，但 v1 暂不产出 APK；iOS 尚未支持。

## 从源码构建

环境要求：

- [Node.js](https://nodejs.org) 24 与 [pnpm](https://pnpm.io) 11
- [Rust](https://rustup.rs) stable（edition 2024）
- [Tauri 2 系统依赖](https://tauri.app/start/prerequisites/)（Windows 需 WebView2；Linux 需 webkit2gtk 4.1 等）

```bash
pnpm install

pnpm tauri dev                 # 开发模式（热重载）
pnpm tauri build               # 打包安装器（MSI / NSIS / AppImage / deb / rpm）
pnpm tauri build --no-bundle   # 仅生成便携 exe（target/release/ircmontis.exe）
```

质量检查：

```bash
pnpm check              # svelte-check 类型检查
pnpm test               # vitest 前端单测
cargo test --workspace  # Rust 单测
```

## 数据目录

| 内容 | Windows 路径 |
| --- | --- |
| 消息数据库 | `%APPDATA%\com.ircmontis.app\ircmontis.db` |
| 自定义背景图 | `%APPDATA%\com.ircmontis.app\backgrounds\` |

## 许可

[Apache-2.0](LICENSE)。