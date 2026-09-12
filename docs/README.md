# docs/ 说明

这里是设计文档与实施 spec。

**重要前提**:`docs/2026-*.md` 里的 spec 写于 **上游 OpenTypeless 时期**,当时产品还有云端账号、订阅(Pro)、托管语音识别与 GitHub 发布流水线。本仓库是**内部本地优先版**,这些内容已全部移除。因此:

- spec 中对**本地行为**的描述(Ask 可靠性、Windows 输出修复、i18n、自定义场景、录音时长 UI 等)通常仍然有效;
- spec 中涉及**云端服务、订阅/Pro 分层、托管供应商、计费、发布与签名**的章节**已不再适用**,请以代码为准。

新写文档时请直接描述当前实现,不要再引用已移除的云端路径。

## 索引

| 文档 | 内容 | 现状 |
| --- | --- | --- |
| [2026-05-21-optimization-design.md](2026-05-21-optimization-design.md) | 整体优化设计 | 本地部分有效 |
| [2026-05-22-i18n-design.md](2026-05-22-i18n-design.md) | i18n 设计与 key 规划 | 本地部分有效 |
| [2026-05-22-i18n-plan.md](2026-05-22-i18n-plan.md) | i18n 实施计划 | 本地部分有效 |
| [2026-05-22-optimization-plan.md](2026-05-22-optimization-plan.md) | 三阶段优化实施计划(含已删除的第三方 STT 供应商) | 部分过时 |
| [2026-06-29-ask-anything-reliability-spec.md](2026-06-29-ask-anything-reliability-spec.md) | Ask 链路可靠性、失败矩阵、弹窗投递 | 已按内部版改写 |
| [2026-06-30-local-first-custom-scenes-spec.md](2026-06-30-local-first-custom-scenes-spec.md) | 自定义场景(local-first) | 部分过时(Pro/云端章节) |
| [2026-07-07-ask-anything-floating-note-ux-spec.md](2026-07-07-ask-anything-floating-note-ux-spec.md) | Ask 便签式浮窗形态 | 部分过时(cloud quota/BYOK 章节) |
| [2026-07-08-gap-closure-implementation-spec.md](2026-07-08-gap-closure-implementation-spec.md) | 功能差异补齐 | 部分过时 |
| [2026-07-17-windows-modifier-safe-output-spec.md](2026-07-17-windows-modifier-safe-output-spec.md) | Windows 修饰键安全输出修复 | 仍适用 |
| [2026-07-23-desktop-copy-recording-limit-ux-spec.md](2026-07-23-desktop-copy-recording-limit-ux-spec.md) | 复制行为与录音时长上限 UI | 部分过时(托管云端上限章节) |
| [2026-08-08-reliability-punctuation-wayland-release-spec.md](2026-08-08-reliability-punctuation-wayland-release-spec.md) | 可靠性、标点、Wayland | 部分过时(发布章节) |
| [app-icon-sources.md](app-icon-sources.md) | 应用图标来源与授权 | 仍适用 |

## 与出口策略相关的改动

任何涉及网络请求、provider、URL 的改动,先读 `src-tauri/src/egress.rs` 与根目录 [README.md](../README.md) 的「网络出口策略」一节。前端不是安全边界。
