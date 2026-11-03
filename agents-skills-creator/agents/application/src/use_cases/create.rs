//! UC2 Agent を作るの create：SAS に Agent の項目を書かせる。中で sas create を呼び、結果は書き換えずに返す。
//! どちらの書き方も Agent のフォルダを受け取り、<フォルダ>/agent.json を経路にする（名前が決まっていなければ
//! agents/<name> のように渡す）。Concrete が Agent の Concrete で --semantic のときだけ、作る流れを添える。

use crate::ports::{AgentWriter, Run};
use crate::use_cases::{AGENT_JSON, Refused, trimmed};
use serde_json::{Value, json};

/// Agent の Concrete の $id。
pub const AGENT_ID: &str = "urn:agents-skills-creator:agent:v1";

/// 推測した所に付ける印。
pub const GUESS: &str = "【推測】";

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
    /// 作る流れ（Agent の Concrete で --semantic のときだけ）
    pub flow: Option<Value>,
    /// SAS の結果（書き換えない）
    pub sas: Run,
}

/// Agent を作る流れ。打ったセッション（AI）が読んで進める指示（Skill を作る流れと同じ考え）。
fn flow() -> Value {
    json!({
        "guess_mark": GUESS,
        "first_reply": [
            {"id": "purpose", "show": "目的の仮説", "how": format!("作る人が言ったことから、誰が ・ どんな場面で ・ 何を得るかを1文の仮説にする。決めるのは作る人の承認で、仮説には{GUESS}を付ける")},
            {"id": "completion", "show": "完成の像（頼む文と返る答え）", "how": "使い手がこう頼むと、こう返る、を具体の文で見せる。返る答えの各行に、どの Skill の出力から来たかを付ける"},
            {"id": "structure", "show": "構造の像", "how": "structure_image の4つを見せる"},
            {"id": "questions", "show": "推奨付きの問い 0〜2個", "how": "ask_only_when の両方を満たす所だけを聞く"}
        ],
        "structure_image": [
            {"id": "scope", "show": "受け持つ範囲", "how": "入力 → Agent がすること → 出力 と、外に置くもの（Skill がすること ・ Agent がしないこと）"},
            {"id": "calls", "show": "呼ぶ Skill と受け渡し", "how": "どのステップで、どの Skill を同時に呼び、どの出力（Skill の outputs の name）を次へ渡すか。配下の Skill は skills validate の結果の name ・ description ・ inputs ・ outputs だけで選ぶ"},
            {"id": "steps", "show": "手順と分かれ道", "how": "Skill を呼ぶステップと人が承認するステップの並びと、分かれ道"},
            {"id": "merge", "show": "まとめ方", "how": "呼んだ Skill の出力をどうまとめるか（rules）。Agent は作業の中身を持たない"}
        ],
        "ask_only_when": [
            "値を変えると、完成の像が目に見えて変わる",
            "会話 ・ リポジトリ ・ 既にある Skill のどれからも決まらない"
        ],
        "otherwise": format!("ask_only_when のどちらかを満たさない所は、AI が決め、{GUESS}を付けて像の中に出す"),
        "finish": [
            "値を組み、agents create <フォルダ> --concrete <URN> --value <JSON> で SAS に書かせる",
            "Agent のフォルダの skills/ に、配下の Skill のフォルダへのシンボリックリンクを同じ名前で置き、agents write-agent-md と agents validate を打つ",
            "別の会話で例を動かし（validate の承認の材料の delegate）、実際の答えを完成の像と並べ、承認を受ける"
        ],
        "rules": [
            "目的は AI が仮説として示し、作る人が承認して決める",
            "Agent は作業の中身を持たず、配下の Skill を呼ぶステップと、人が承認するステップだけを持つ",
            "足りない Skill は skills create で先に作る（Agent の中に書かない）",
            "creator は AI を起動せず、agent.json を書かない（SAS が書く）"
        ],
        "presentation": "チャットの中で見せるので、表 ・ コードブロック ・ 木構造で出す。図を出せるツールなら図でもよい"
    })
}

/// SAS に Agent の項目を書かせる。
///
/// # Errors
/// フォルダ ・ Concrete ・ 書き方が無い（usage）・ sas を呼べない（sas_unavailable）とき。
pub fn create(
    folder: Option<&str>,
    concrete: Option<&str>,
    mode: Option<Mode>,
    sas: &dyn AgentWriter,
) -> Result<Created, Refused> {
    let usage = |m: &str| Refused {
        code: "usage",
        message: m.to_string(),
    };
    let folder = folder.ok_or_else(|| {
        usage("Agent のフォルダを渡してください（名前が決まっていなければ agents/<name> のように）")
    })?;
    let concrete =
        concrete.ok_or_else(|| usage("--concrete で、従う Concrete の URN を渡してください"))?;
    let mode =
        mode.ok_or_else(|| usage("--semantic か --value <JSON> のどちらかを渡してください"))?;
    let unavailable = |message: String| Refused {
        code: "sas_unavailable",
        message,
    };
    let path = format!("{}/{AGENT_JSON}", trimmed(folder));
    match mode {
        Mode::Semantic => Ok(Created {
            flow: (concrete == AGENT_ID).then(flow),
            sas: sas.create_semantic(&path, concrete).map_err(unavailable)?,
        }),
        Mode::Value(v) => Ok(Created {
            flow: None,
            sas: sas.create_value(&path, concrete, &v).map_err(unavailable)?,
        }),
    }
}
