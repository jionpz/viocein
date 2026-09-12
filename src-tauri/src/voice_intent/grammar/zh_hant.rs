use super::CommandMatch;
use crate::voice_intent::normalize::NormalizedUtterance;

pub(super) fn match_draft(view: &NormalizedUtterance<'_>) -> CommandMatch<String> {
    for prefix in ["寫一封", "幫我寫", "回覆說", "寫個", "起草"] {
        if !view.starts_with_prefix(prefix, false) {
            continue;
        }
        return view
            .payload_after_prefix(prefix)
            .map(CommandMatch::Matched)
            .unwrap_or(CommandMatch::MissingPayload);
    }
    CommandMatch::NoMatch
}

pub(super) fn matches_rewrite(view: &NormalizedUtterance<'_>) -> bool {
    [
        "改寫這段",
        "潤色這段",
        "把這段寫得",
        "精簡這段",
        "擴寫這段",
        "修正這段",
        "把這段改成",
    ]
    .iter()
    .any(|prefix| view.starts_with_prefix(prefix, false))
}

pub(super) fn matches_translation(view: &NormalizedUtterance<'_>) -> bool {
    ["把這段翻譯成", "翻譯這段到", "將選取文字翻譯成"]
        .iter()
        .any(|prefix| {
            view.starts_with_prefix(prefix, false) && view.payload_after_prefix(prefix).is_some()
        })
}

pub(super) fn matches_informational(view: &NormalizedUtterance<'_>) -> bool {
    [
        "總結這段",
        "解釋這段",
        "比較這段",
        "這段是什麼意思",
        "為什麼",
        "怎麼",
        "什麼",
        "誰",
        "何時",
        "哪裡",
    ]
    .iter()
    .any(|prefix| view.starts_with_prefix(prefix, false))
}
