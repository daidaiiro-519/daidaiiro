//! UC1 作るの create：Semantic Agent Schema に Skill の項目を書かせる。中で semantic-agent-schema create を呼び、結果は書き換えずに返す。
//!
//! どちらの書き方も Skill のフォルダを受け取り、<フォルダ>/skill.json を経路にする。名前がまだ決まっていないときは
//! skills/<name> のように、決まっていない所を <name> のまま渡してよい（--semantic は何も書かないので、Semantic Agent Schema は経路の
//! 名前を決めずに手順を返し、手順の書くコマンドに同じ経路が入る）。
//!
//! - --semantic：意味で書く手順を受け取る（何も書かない）。Concrete が
//!   Agent Skill の Concrete（skill.schema）のときだけ、作る流れ（最初の返事で見せる目的の仮説 ・ 完成の像 ・ 構造の像 ・
//!   推奨付きの問い、聞く所の見分け方、2往復目から、仕上げ、守ること）を添える。ほかの Concrete のときは、Semantic Agent Schema の手順そのままにする
//! - --value：値を書かせる

use crate::ports::{SasRun, SkillWriter};
use crate::use_cases::approval::UNKNOWN;
use crate::use_cases::{Refused, as_creator_procedure, trimmed};
use serde_json::{Value, json};

/// Agent Skill の Concrete の $id（作る流れを添える Concrete）。
pub const SKILL_ID: &str = "urn:agents-skills-creator:skill:v1";

/// 書き方。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// 意味で書く手順を受け取る
    Semantic,
    /// 値を書かせる
    Value(String),
}

/// 書かせた結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Created {
    /// 作る流れ（Agent Skill の Concrete で --semantic のときだけ）
    pub flow: Option<Value>,
    /// Semantic Agent Schema の結果（書き換えない）
    pub sas: SasRun,
}

/// 推測した所に付ける印。
pub const GUESS: &str = "【推測】";

/// Agent Skill の Concrete で作るときの、作る流れ。打ったセッション（AI）が読んで進める指示で、作る人に見せる順と、
/// 聞く所の見分け方と、守ることを持つ。値の組み方そのものは、semantic-agent-schema の手順（各欄の説明 ・ 読み方 ・ rule）に従う。
fn flow() -> Value {
    json!({
        "guess_mark": GUESS,
        "first_reply": [
            {"id": "purpose", "show": "目的の仮説", "how": format!("作る人が言ったことから、誰が ・ どんな場面で ・ 何を得るかを1文の仮説にする。作る人が「この目的で進めてよい」と答えて決まる。仮説には{GUESS}を付ける")},
            {"id": "completion", "show": "完成の像（頼む文と返る答え）", "how": "使い手がこう頼むと、こう返る、を具体の文で1組以上見せる。返る答えは、出力の形のまま書く"},
            {"id": "structure", "show": "構造の像", "how": "structure_image の4つを見せる"},
            {"id": "questions", "show": "推奨付きの問い 0〜2個", "how": "ask_only_when の両方を満たす所だけを、question の形で聞く。満たす所が無ければ聞かない"}
        ],
        "structure_image": [
            {"id": "scope", "show": "受け持つ範囲", "how": "入力 → 使い手がすること → 出力 と、外に置くもの（受け持たないこと）"},
            {"id": "materials", "show": "資料と出力の関係", "how": "どの資料を、どのステップで読み、出力のどの根拠になるか"},
            {"id": "steps", "show": "手順と分かれ道", "how": "ステップの並びと、どのステップで、どんなときに分かれ、どこへ戻るか ・ 止まるか"},
            {"id": "common", "show": "共通の所と変わる所", "how": "同じ種類の Skill に共通の所と、Skill ごとに変わる所。種類が無ければ「無し」と書く"}
        ],
        "box_names": "構造の像の箱の名前は、skill.json の欄名でなく、使い手がすることの動詞で書く（例：「当たる判断基準を読む」）",
        "ask_only_when": [
            "値を変えると、完成の像が目に見えて変わる",
            "会話 ・ リポジトリ ・ 既にある Skill のどれからも決まらない"
        ],
        "otherwise": format!("ask_only_when のどちらかを満たさない所は、AI が決め、{GUESS}を付けて像の中に出す"),
        "question": [
            {"id": "decide", "show": "決めること（作る人の言葉で1文）"},
            {"id": "recommended", "show": "推奨（理由を1文）"},
            {"id": "alternatives", "show": "ほかの選択肢"},
            {"id": "changes", "show": "完成の像で変わる所（どの行がどう変わるか）"}
        ],
        "later_rounds": "2往復目からは、変わった所だけを「前 → 後」で出し、残りの問いの数を付ける。作る人が「ここが違う」と言った所は、そのまま直して見せる",
        "finish": [
            "値を組み、semantic-agent-schema の手順の書くコマンドの代わりに agents-skills-creator skills create <フォルダ> --value <JSON か @<ファイル>> で Semantic Agent Schema に書かせ（--concrete は省ける）、agents-skills-creator skills write-skill-md と agents-skills-creator skills validate を打つ",
            "別の会話で例を動かし（validate の承認の材料の delegate）、実際の答えを完成の像と並べて見せる",
            "作る前と後の振る舞い（使い手が何をすると、何が変わるか）を出し、作る人に承認（agents-skills-creator approve）を打ってもらうか、「ここが違う」の一言を受け取る"
        ],
        "rules": [
            "目的は AI が仮説として示し、作る人が進めてよいと答えて決まる（「承認」は、approve で approved を書くことだけを指す）",
            "承認の前に自分で気づいた直しも、agents-skills-creator skills update <フォルダ> --patch <JSON Patch か @<ファイル>> --if-match <ハッシュ> --reason 層：<層> で書かせる",
            "像は判断してもらう手段で、目的ではない。作る人が「ここが違う」と言える所だけを見せる",
            "作る人が既に言ったことは聞き直さない",
            "推測した所には必ず印を付ける。作る人が答えた件の from は「作る人の答え」にする",
            "creator は AI を起動せず、skill.json を書かない（Semantic Agent Schema が書く）"
        ],
        "presentation": "チャットの中で見せるので、表 ・ コードブロック ・ 木構造で出す。図を出せるツールなら図でもよい",
        "unknown": format!("作る人は「{UNKNOWN}」と答えてよい。会話を止めず、その所に「{UNKNOWN}」と入れて進める（validate の承認の材料で推測の一覧の先頭に出る）")
    })
}

/// Semantic Agent Schema に Skill の項目を書かせる。
///
/// # Errors
/// 置き場所かフォルダか Concrete が無い（usage）・ semantic-agent-schema を呼べない（semantic_agent_schema_unavailable）とき。
pub fn create(
    target: Option<&str>,
    concrete: Option<&str>,
    mode: Option<Mode>,
    sas: &dyn SkillWriter,
) -> Result<Created, Refused> {
    let usage = |message: &str| Refused {
        code: "usage",
        message: message.to_string(),
    };
    let target = target.ok_or_else(|| {
        usage("Skill のフォルダを渡してください（名前が決まっていなければ skills/<name> のように）")
    })?;
    // --concrete を省いたら、Agent Skill の Concrete
    let concrete = concrete.unwrap_or(SKILL_ID);
    let mode =
        mode.ok_or_else(|| usage("--semantic か --value <JSON> のどちらかを渡してください"))?;
    let unavailable = |message: String| Refused {
        code: "semantic_agent_schema_unavailable",
        message,
    };
    match mode {
        Mode::Semantic => {
            let path = format!("{}/skill.json", trimmed(target));
            let mut run = sas.create_semantic(&path, concrete).map_err(unavailable)?;
            as_creator_procedure(
                &mut run.output,
                if concrete == SKILL_ID {
                    format!(
                        "agents-skills-creator skills create {} --value <JSON か @<ファイル>>",
                        trimmed(target)
                    )
                } else {
                    format!(
                        "agents-skills-creator skills create {} --concrete {concrete} --value <JSON か @<ファイル>>",
                        trimmed(target)
                    )
                },
                "skills",
            );
            Ok(Created {
                flow: (concrete == SKILL_ID).then(flow),
                sas: run,
            })
        }
        Mode::Value(value) => {
            let path = format!("{}/skill.json", trimmed(target));
            let run = sas
                .create_value(&path, concrete, &value)
                .map_err(unavailable)?;
            Ok(Created {
                flow: None,
                sas: run,
            })
        }
    }
}
