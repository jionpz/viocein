# Changelog

本文件只记录内部本地优先版(viocein)自身的变更;本仓库是
[OpenTypeless](https://github.com/tover0314-w/opentypeless) 的 fork,
fork 之前的历史见 git 记录。云端账号、订阅、托管语音识别、第三方 provider、
自动更新与 deep-link 等能力已整体移除。

格式参考 [Keep a Changelog](https://keepachangelog.com/)。

## [Unreleased]

### Fixed

- 发布流水线:Windows 安装包在 artifact 里带了 `nsis/` / `msi/` 子目录层级,而
  publish job 只扫顶层文件,导致 v0.2.2 的 Release 一度只挂上 macOS 的 dmg。改为
  上传前先把安装包摊平到 `dist/`,并在发布前断言 exe / msi / dmg 三种产物齐全。

## [0.2.2] - 2026-09-13

### Changed

- 出口策略不再跟随 HTTP 重定向:`egress` 与各 provider HTTP 客户端统一使用
  `redirect::Policy::none()`,重定向响应直接作为错误暴露,避免请求绕过初始 URL 校验。
- 清理内部版死代码:移除托盘“账户”入口、未使用的 clipboard-manager 与
  global-shortcut 前端插件、`http` crate,以及已不再渲染的系统状态检查(diagnostics)链路。
- 界面语言收敛为 English / 简体中文:删除其余 8 个语言包、`UI_LANGUAGES` 与托盘文案;
  旧的 `ui_language` 值会回退到英文。
- capabilities 只保留前端实际调用的权限(删 6 条未使用的 window 权限),并去掉恒为
  `false` 的 `setFocusable` 调用 —— 胶囊窗口本身已配置 `focusable: false`,改由 Rust 测试断言。
- 9 篇上游时期实现 spec 移入 `docs/archive/`,现行文档只留在 `docs/` 顶层。

### Removed

- 移除指向上游仓库的 `scripts/create-labels.sh`。

## [0.2.1] - 2026-09-13

### Changed

- 打 `v*` tag 时同时构建 Windows 与 macOS 安装包并发布到 GitHub Release;
  手动触发只产出 workflow artifact。

## [0.2.0] - 2026-09-13

### Added

- GitHub Release 发布流水线(Windows NSIS/MSI 安装包)。

## [0.1.42] - 2026-09-13

### Changed

- 项目更名为 viocein。

### Removed

- 移除云账号 / 登录、订阅与支付、云备份、托管语音识别、第三方 LLM provider、
  自动更新器与 deep-link。
- 出口策略收敛为 Rust 侧单一 fail-closed 模块(`src-tauri/src/egress.rs`):
  只允许公司 OpenAI 兼容网关与 loopback 本地语音识别。
