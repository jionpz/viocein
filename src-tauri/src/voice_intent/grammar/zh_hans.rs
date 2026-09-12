use super::CommandMatch;
use crate::voice_intent::normalize::NormalizedUtterance;

pub(super) fn match_draft(view: &NormalizedUtterance<'_>) -> CommandMatch<String> {
    for prefix in ["写一封", "帮我写", "回复说", "写个", "起草"] {
        if !view.starts_with_prefix(prefix, false) {
            continue;
        }
        return view
            .payload_after_prefix(prefix)
            .map(CommandMatch::Matched)
            .unwrap_or(CommandMatch::MissingPayload);
    }
    for prefix in [
        "写一份",
        "我想写一份",
        "我想写一封",
        "帮我写一封邮件",
        "帮我写一份邮件",
    ] {
        if !view.starts_with_prefix(prefix, false) {
            continue;
        }
        let Some(payload) = view.payload_after_prefix(prefix) else {
            return CommandMatch::MissingPayload;
        };
        if looks_like_draft_artifact(&payload) {
            return CommandMatch::Matched(payload);
        }
    }
    CommandMatch::NoMatch
}

fn looks_like_draft_artifact(payload: &str) -> bool {
    ["邮件", "封信", "消息", "通知", "回复", "邀请"]
        .iter()
        .any(|marker| payload.contains(marker))
}

pub(super) fn matches_rewrite(view: &NormalizedUtterance<'_>) -> bool {
    [
        "改写这段",
        "润色这段",
        "把这段写得",
        "精简这段",
        "扩写这段",
        "修正这段",
        "把这段改成",
    ]
    .iter()
    .any(|prefix| view.starts_with_prefix(prefix, false))
}

pub(super) fn matches_translation(view: &NormalizedUtterance<'_>) -> bool {
    ["把这段翻译成", "翻译这段到", "将选中文字翻译成"]
        .iter()
        .any(|prefix| {
            view.starts_with_prefix(prefix, false) && view.payload_after_prefix(prefix).is_some()
        })
}

pub(super) fn matches_informational(view: &NormalizedUtterance<'_>) -> bool {
    [
        "总结这段",
        "解释这段",
        "比较这段",
        "这段是什么意思",
        "为什么",
        "怎么",
        "什么",
        "谁",
        "何时",
        "哪里",
    ]
    .iter()
    .any(|prefix| view.starts_with_prefix(prefix, false))
}
