//! プロンプト：基盤の呼び出し（call.schema.json の形）を、コーディングエージェントに渡す文に直す

use serde_json::Value;

/// judging の任せる先に渡すプロンプト：立場 ・ 判定する rule ・ 読む値 ・ 返す形
pub fn judging(role: &str, call: &Value, output_schema: &Value) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "あなたは次の立場で、rule の合否だけを判定する：{role}\n"
    ));
    s.push_str("道具は使えず、ファイルも読めない。根拠は「読む値」にある値だけにする。\n\n");
    s.push_str(&format!(
        "## 判定する rule（対象 {}）\n",
        call["target"].as_str().unwrap_or_default()
    ));
    for r in call["rules"].as_array().into_iter().flatten() {
        let t = |k: &str| r[k].as_str().unwrap_or_default().to_string();
        s.push_str(&format!(
            "- {}：{}\n  なぜ：{}\n  確かめ方：{}\n",
            t("id"),
            t("rule"),
            t("why"),
            t("test")
        ));
    }
    s.push_str("\n## 読む値\n");
    for i in call["inputs"].as_array().into_iter().flatten() {
        s.push_str(&format!("- {}", i["at"].as_str().unwrap_or_default()));
        if let Some(d) = i["description"].as_str() {
            s.push_str(&format!("：{d}"));
        }
        s.push('\n');
        if let Some(r) = i.get("read").filter(|r| !r.is_null()) {
            s.push_str(&format!("  読み方：{r}\n"));
        }
        s.push_str(&format!("  値：{}\n", i["value"]));
    }
    s.push_str("\n## 返す形\n");
    s.push_str("次の JSON Schema に沿った JSON だけを返す。rule ごとに results へ1件入れ、result は verified ・ violated ・ unverified のどれか、grounds には根拠にした読む値の場所（上の - の後の文字列）を入れる。決めきれなければ unverified にする。\n");
    s.push_str(&output_schema.to_string());
    s.push('\n');
    s
}
