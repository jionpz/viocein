# Ask Anything Reliability Design Spec

Date: 2026-06-29
Status: implemented；2026-09-13 按内部本地优先版范围改写（移除云端代理与计费相关内容）
Scope: viocein 桌面端 Ask Anything（本地 STT + 公司 / 本地 LLM）

## Executive Summary

Ask Anything 不能只修一个超时或弹窗问题。它是一条跨桌面端、麦克风、STT、LLM、结果窗口的链路。每个失败点都必须有明确的用户反馈，并且失败时不得继续调用后续阶段。

当前根因判断：

1. 结果弹窗丢失是真问题。快捷键路径在 Rust 里打开 Ask 窗口后立即发 `ask:result`，如果前端监听还没挂上，事件会丢，用户会看到胶囊完成但没有结果弹窗内容。
2. 无声音长时间 thinking 是真问题。当前 Ask STT finalize 最长等 `120s`，没有语音或音频流无法结束时，用户会长时间卡在 thinking。
3. STT、LLM、网络错误必须原样返回给用户。不能把 STT auth/service error 覆盖成 `No speech detected`。
4. 空语音必须在桌面端止住：没有有效 transcript 时不得调用 LLM。

## User Contract

用户触发 Ask Anything 后，只会看到这些状态：

1. Recording: 胶囊显示正在录音。
2. Thinking: 用户停止录音后，开始处理 STT 和 LLM。
3. Result popup: 成功时弹窗只显示最终答案，不显示输入框、不显示发送按钮、不显示上下文。
4. Error popup: 失败时弹窗只显示一条可理解错误，不显示输入框、不显示发送按钮。

失败不能静默。只要链路中任一阶段失败，就必须让用户看到错误，并且恢复 idle。

## End-to-End Chain

### Desktop Hotkey Flow

1. User presses Ask hotkey.
2. `src-tauri/src/hotkey.rs` calls `start_ask_dictation`.
3. `start_ask_dictation` loads config, validates STT auth/config, connects STT provider, starts audio capture, emits `PipelineState::Recording`.
4. User presses Ask hotkey again.
5. `stop_ask_dictation` stops audio capture, emits `PipelineState::Polishing`, waits for STT finalize, validates transcript, then calls LLM.
6. On success, Rust returns `AskDictationResult { question, answer }`.
7. Hotkey handler opens/focuses the `ask` window and delivers the result.
8. `src/components/AskPanel/AskPanel.tsx` renders answer-only popup content.

### LLM Flow

1. Desktop calls the configured LLM only after a non-empty validated transcript exists.
2. The provider is either the company OpenAI-compatible gateway (`company`) or a loopback Ollama instance (`ollama`）。
3. Before any request is built, the base URL is re-validated against `src-tauri/src/egress.rs`；不合规的地址直接拒绝，不会发出请求。

## Failure Matrix

| Stage       | Failure                                        | Required user feedback                                   | Continue to next stage? |
| ----------- | ---------------------------------------------- | -------------------------------------------------------- | ----------------------- |
| Start       | Ask already processing                         | Ignore duplicate hotkey                                  | No                      |
| Config      | Config load fails                              | `Could not load settings. Please retry.`                 | No                      |
| STT config  | Provider requires key but key missing          | `Configure speech recognition before using Ask.`         | No                      |
| STT config  | Base URL is not loopback                       | `Local STT must point at localhost / 127.0.0.1 / [::1]`  | No                      |
| STT connect | Provider connect fails                         | Provider error text, sanitized                           | No                      |
| Audio       | No input device                                | `Microphone unavailable. Check your input device.`       | No                      |
| Audio       | Permission denied                              | `Microphone permission is required.`                     | No                      |
| Recording   | Send audio fails                               | STT error text, sanitized                                | No                      |
| STT         | Provider returns auth/service error            | Exact mapped error: auth or service                      | No                      |
| STT         | Empty transcript                               | `No speech detected. Please try again.`                  | No                      |
| STT         | Finalize timeout with no transcript            | `No speech detected. Please try again.`                  | No                      |
| STT         | Finalize timeout after final transcript exists | Continue with collected final transcript                 | Yes                     |
| Transcript  | Over 500 chars                                 | `Question is too long.`                                  | No                      |
| LLM config  | No usable LLM configuration                    | `Configure an LLM provider.`                             | No                      |
| LLM config  | Gateway URL fails egress validation            | Config error text from `egress.rs`                       | No                      |
| LLM         | Provider/service error                         | `Ask service error. Please try again.`                   | No                      |
| Popup       | Native result event is missed                  | Frontend fetches pending result once on mount            | N/A                     |
| Popup       | Native error event is missed                   | Frontend fetches pending error once on mount             | N/A                     |

## Error Handling Requirements

Desktop must use one canonical Ask message shape:

```ts
type AskPopupMessage =
  | { kind: 'result'; payload: { question: string; answer: string } }
  | { kind: 'error'; payload: string }
```

Rules:

1. Hotkey path stores the latest result/error in Rust before showing the Ask window.
2. Ask window listens for `ask:result` and `ask:error`.
3. Ask window also calls `take_pending_ask_message` once after listeners are ready.
4. `take_pending_ask_message` consumes the message exactly once.
5. Event delivery and pending-message delivery must both render identical answer-only/error-only popup UI.
6. On any error, desktop emits idle state and clears busy/recording state.
7. STT errors must not be overwritten by later empty-transcript validation.
8. Empty transcript must never call `answer_question`.
9. Error strings shown to users must be sanitized and bounded.

## Hard Rules

1. STT start/connect/audio failures must not call LLM.
2. STT provider auth/service failures must not call LLM.
3. Empty transcript must not call LLM.
4. `question` validation must run before the LLM request.
5. Ask output stays capped at `ASK_OUTPUT_TOKEN_LIMIT = 80`.
6. Repeated popup retries must not re-call STT or LLM.

## Current Code Evidence

1. `src-tauri/src/commands/ask.rs` validates empty questions before calling LLM.
2. `src-tauri/src/commands/ask.rs` bounds the STT finalize wait.
3. `src-tauri/src/hotkey.rs` stores the result/error before showing the Ask window, and the frontend also polls `take_pending_ask_message` as a fallback.
4. `src/components/AskPanel/AskPanel.tsx` renders non-embedded popup as result/error-only and recovers pending messages when a native event is missed.

## Required Implementation Tasks

### Task 1: Result/Error Popup Delivery

Files:

1. `src-tauri/src/commands/ask.rs`
2. `src-tauri/src/hotkey.rs`
3. `src-tauri/src/lib.rs`
4. `src/lib/tauri.ts`
5. `src/components/AskPanel/AskPanel.tsx`
6. `src/components/AskPanel/__tests__/AskPanel.test.tsx`
7. `src/lib/__tests__/tauri-ask.test.ts`

Acceptance:

1. Add pending one-shot Ask result/error state in Rust.
2. Register `take_pending_ask_message`.
3. Frontend fetches pending message after listeners attach.
4. Tests cover missed native result event.
5. Tests cover Tauri wrapper invocation.

Commands:

```bash
npm test -- --run src/components/AskPanel/__tests__/AskPanel.test.tsx src/lib/__tests__/tauri-ask.test.ts
cargo test ask::tests --manifest-path src-tauri/Cargo.toml
```

### Task 2: No-Speech and Timeout Handling

Files:

1. `src-tauri/src/commands/ask.rs`
2. `src-tauri/src/audio/capture.rs`
3. `src-tauri/src/stt/*.rs` as needed after provider-specific review

Acceptance:

1. Stop path uses a short bounded STT finalize wait.
2. No transcript returns `No speech detected. Please try again.`
3. Empty transcript never calls LLM.
4. Existing STT provider error wins over no-speech.
5. If a final transcript already exists when finalize times out, continue with that transcript.
6. Manual Windows test covers silence and real speech.
7. Manual macOS test covers silence and real speech.

Commands:

```bash
cargo test ask::tests --manifest-path src-tauri/Cargo.toml
npm test -- --run src/components/AskPanel/__tests__/AskPanel.test.tsx
```

### Task 3: Error Mapping Audit

Files:

1. `src-tauri/src/commands/ask.rs`
2. `src-tauri/src/stt/config.rs`
3. `src/i18n/locales/*.json` if Ask-specific localized errors are added

Acceptance:

1. Microphone permission/device errors become user-readable messages.
2. STT auth/network/service errors remain visible and are not replaced by no-speech.
3. LLM auth/network/service errors remain visible.
4. Egress policy rejections surface as a configuration error, not a generic network failure.
5. Errors are short enough for the Ask popup.

## Manual Verification Gate

Before an internal build is handed out, verify all rows below:

| Platform | Case                                    | Expected result                  |
| -------- | --------------------------------------- | -------------------------------- |
| Windows  | Press Ask, say nothing, stop            | Error popup appears; no LLM call |
| Windows  | Press Ask, speak one question, stop     | Answer-only popup appears        |
| Windows  | Disable/deny mic                        | Error popup appears              |
| macOS    | Press Ask, say nothing, stop            | Error popup appears; no LLM call |
| macOS    | Press Ask, speak one question, stop     | Answer-only popup appears        |
| macOS    | Ask window was not loaded before result | Pending result still appears     |
| Linux    | Press Ask, speak one question, stop     | Answer-only popup appears        |

## Release Decision

Ask Anything is ready for an internal build only when:

1. Unit tests for pending popup and no-speech pass.
2. Desktop build passes.
3. No request is issued to a destination outside the egress allowlist.
