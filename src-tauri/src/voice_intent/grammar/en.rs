use super::CommandMatch;
use crate::voice_intent::normalize::NormalizedUtterance;

pub(super) fn match_draft(view: &NormalizedUtterance<'_>) -> CommandMatch<String> {
    for prefix in ["reply with", "compose", "draft", "write"] {
        if !view.starts_with_prefix(prefix, true) {
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
        "rewrite this",
        "rephrase this",
        "make this shorter",
        "make this longer",
        "make this warmer",
        "make this friendlier",
        "make this more formal",
        "make this more concise",
        "fix the grammar",
        "fix the spelling",
        "format this as",
        "turn this into",
    ]
    .iter()
    .any(|prefix| view.starts_with_prefix(prefix, true))
}

pub(super) fn matches_translation(view: &NormalizedUtterance<'_>) -> bool {
    [
        "translate this to",
        "translate this into",
        "translate the selection to",
        "translate the selection into",
    ]
    .iter()
    .any(|prefix| {
        view.starts_with_prefix(prefix, true) && view.payload_after_prefix(prefix).is_some()
    })
}

pub(super) fn matches_informational(view: &NormalizedUtterance<'_>) -> bool {
    [
        "summarize this",
        "explain this",
        "compare this",
        "what ",
        "why ",
        "how ",
        "who ",
        "when ",
        "where ",
    ]
    .iter()
    .any(|prefix| view.match_text().starts_with(prefix))
}
