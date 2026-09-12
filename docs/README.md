# docs/ 说明

这里放**当前有效**的工程文档;上游时期的实现 spec 已移到 [`archive/`](archive/)。

**重要前提**:本仓库是内部本地优先版,云端账号、订阅(Pro)、托管语音识别、
GitHub 发布流水线等能力已全部移除。当前实现以**代码**为准。

## 现行文档

| 文档 | 内容 |
| --- | --- |
| [2026-06-29-ask-anything-reliability-spec.md](2026-06-29-ask-anything-reliability-spec.md) | Ask 链路可靠性、失败矩阵、弹窗投递(已按内部版改写) |
| [2026-07-17-windows-modifier-safe-output-spec.md](2026-07-17-windows-modifier-safe-output-spec.md) | Windows 修饰键安全输出修复 |
| [app-icon-sources.md](app-icon-sources.md) | 应用图标来源与授权 |

## 归档(历史 spec,只作参考)

`archive/` 下的文档写于裁剪前后,含已移除能力的章节(云端 / 订阅 / 托管 provider / 发布与签名)。
每篇顶部都有归档说明;需要追溯设计动机时再读,不要当作当前规范。

| 文档 | 内容 |
| --- | --- |
| [2026-05-21-optimization-design.md](archive/2026-05-21-optimization-design.md) | 整体优化设计 |
| [2026-05-22-i18n-design.md](archive/2026-05-22-i18n-design.md) | i18n 设计与 key 规划 |
| [2026-05-22-i18n-plan.md](archive/2026-05-22-i18n-plan.md) | i18n 实施计划 |
| [2026-05-22-optimization-plan.md](archive/2026-05-22-optimization-plan.md) | 三阶段优化实施计划(含已删除的第三方 STT 供应商) |
| [2026-06-30-local-first-custom-scenes-spec.md](archive/2026-06-30-local-first-custom-scenes-spec.md) | 自定义场景(local-first) |
| [2026-07-07-ask-anything-floating-note-ux-spec.md](archive/2026-07-07-ask-anything-floating-note-ux-spec.md) | Ask 便签式浮窗形态 |
| [2026-07-08-gap-closure-implementation-spec.md](archive/2026-07-08-gap-closure-implementation-spec.md) | 功能差异补齐 |
| [2026-07-23-desktop-copy-recording-limit-ux-spec.md](archive/2026-07-23-desktop-copy-recording-limit-ux-spec.md) | 复制行为与录音时长上限 UI |
| [2026-08-08-reliability-punctuation-wayland-release-spec.md](archive/2026-08-08-reliability-punctuation-wayland-release-spec.md) | 可靠性、标点、Wayland |

## 与出口策略相关的改动

任何涉及网络请求、provider、URL 的改动,先读 `src-tauri/src/egress.rs` 与根目录 [README.md](../README.md) 的「网络出口策略」一节。前端不是安全边界。
