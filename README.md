# viocein（内部本地优先版）

> 面向公司内部使用的 **local-first** 版本：所有处理都在本机完成，唯一允许的外部依赖是公司自建的 OpenAI 兼容大模型网关。
>
> 本项目是 [OpenTypeless](https://github.com/tover0314-w/opentypeless) 的 fork，遵循 MIT 许可（见 [LICENSE](LICENSE) 与 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)）。

## 这是什么

按一下热键说话，文本就会出现在你正在使用的应用里；也可以提一个一次性的语音问题，直接拿到答案，不用切换聊天软件。

与上游版本的区别：**所有云功能与第三方对接都已移除**，只保留本地能力。

## 保留的能力

- 语音输入与转写（本地模型 / Apple Speech）
- AI polish（清理口述文本、按应用场景调整措辞）
- Ask：语音提问、划词提问
- 翻译
- 应用感知上下文（识别当前前台应用类别）
- 本地历史记录与自定义词典（SQLite，仅存本机）

## 已移除的能力

- 云账号、登录、订阅、升级与支付
- 自动更新器与 deep-link
- 云备份 / 云同步
- 第三方云语音识别（Deepgram、AssemblyAI、火山引擎、阿里云等）
- 第三方大模型 provider（OpenAI、OpenRouter 等）
- 语音搜索与一切外链跳转（Google / YouTube / Amazon / GitHub 等）
- 上游的 GitHub 发布流水线与社区机器人

## 网络出口策略

出口控制由 Rust 侧单一策略模块 `src-tauri/src/egress.rs` 强制执行，**fail-closed**：

| 允许 | 说明 |
|---|---|
| 公司大模型网关 | 必须与设置中配置的 Base URL **同源**，且路径落在其前缀之下 |
| loopback | `localhost` / `127.0.0.0/8` / `[::1]`，仅用于本地语音识别服务 |

其余一切请求在发出前即被拒绝。Base URL 本身也会被校验：必须 `http(s)`、必须有 host、不得包含凭据或 fragment。

补充约束：前端 CSP 的 `connect-src` 只放行 `'self'` 与 loopback，因此 webview 无法直连网关——所有对外请求都必须经过 Rust 策略层。

代码中**不内置任何第三方端点**。

## 配置

### 公司大模型网关

设置 → AI Polish → Provider 选择 **Company LLM Gateway**，然后填写：

- **Base URL**：公司网关地址，例如 `https://llm.your-company.internal/v1`
- **Model**：模型名（可点右侧刷新按钮从 `/models` 拉取列表）
- **API Key**：可选。公司网关若无鉴权可留空；填写后存入系统凭据库，不写入配置文件

填好后点 **Test** 验证连通性。

### 本地语音识别

设置 → 语音识别，二选一：

- **Custom Whisper**：指向本机运行的 OpenAI 兼容转写服务，默认 `http://localhost:8000/v1`（Speaches / faster-whisper 等）。**地址被强制要求为 loopback**，远程地址会被拒绝。
- **Apple Speech**：macOS 内置本地识别。

### 本地大模型（可选）

Provider 选择 **Ollama**，地址必须为 loopback，默认 `http://localhost:11434/v1`。

## 开发

```bash
npm install
npm run tauri dev
```

## 校验

```bash
npm run build        # tsc + vite build
npm run lint         # eslint
npm test             # vitest
npx tsc --noEmit

cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --all-targets --manifest-path src-tauri/Cargo.toml
cargo test --lib --manifest-path src-tauri/Cargo.toml
```

CI 配置见 [.github/workflows/ci.yml](.github/workflows/ci.yml)。

## 目录结构

```
src/            React 前端（设置、历史、Ask 面板、Capsule 悬浮条）
src-tauri/      Rust 后端（Tauri 命令、pipeline、出口策略、本地存储）
docs/           工程设计与实现说明（索引见 docs/README.md）
```

`docs/2026-*.md` 是上游时期的历史 spec，其中涉及云端/订阅/发布的部分已不适用，详见 [docs/README.md](docs/README.md)。出口相关的改动一律先读 `src-tauri/src/egress.rs`。

## 许可

MIT，见 [LICENSE](LICENSE)。上游第三方组件声明见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。
