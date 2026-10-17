//! 文の決まり。1文の長さを数える。引用（「」の中）は数えない。

/// 文の長さの上限（文字）。
pub const SENTENCE_MAX: usize = 120;

/// 文に分ける。句点 ・ 感嘆符 ・ 疑問符 ・ 改行で切り、「」の中の句点では切らない。空の文は返さない。
pub fn sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    for ch in text.chars() {
        match ch {
            '「' | '『' => depth += 1,
            '」' | '』' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if ch == '\n' {
            push(&mut out, &mut current);
            continue;
        }
        current.push(ch);
        if depth == 0 && matches!(ch, '。' | '！' | '？') {
            push(&mut out, &mut current);
        }
    }
    push(&mut out, &mut current);
    out
}

fn push(out: &mut Vec<String>, current: &mut String) {
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        out.push(trimmed.to_owned());
    }
    current.clear();
}

/// 長すぎる文。引用を除いた文字数が上限を超えるものを、文字数と一緒に返す。
pub fn too_long(text: &str) -> Vec<(usize, String)> {
    sentences(text)
        .into_iter()
        .filter_map(|sentence| {
            let length = without_quotes(&sentence).chars().count();
            (length > SENTENCE_MAX).then_some((length, sentence))
        })
        .collect()
}

fn without_quotes(sentence: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for ch in sentence.chars() {
        match ch {
            '「' | '『' => depth += 1,
            '」' | '』' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(ch),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentences_split_on_marks_and_lines_but_not_inside_quotes() {
        assert_eq!(
            sentences("一つ目。二つ目！「引用。の中」も一つ\n三つ目"),
            vec!["一つ目。", "二つ目！", "「引用。の中」も一つ", "三つ目"]
        );
    }

    #[test]
    fn too_long_counts_characters_outside_quotes() {
        let long = "あ".repeat(121);
        let quoted = format!("「{}」と言った。", "い".repeat(200));
        assert_eq!(too_long(&format!("{long}。")).len(), 1);
        assert_eq!(too_long(&quoted), vec![]);
        assert_eq!(too_long(&"う".repeat(120)), vec![]);
    }
}
