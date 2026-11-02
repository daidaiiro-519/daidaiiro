//! UC1 作るの create：SAS に Skill の項目を書かせる。中で sas create を呼び、結果は書き換えずに返す。
//!
//! - --semantic：置き場所の下の <name>/skill.json を経路にして、意味で書く手順を受け取る（何も書かない）。Concrete が
//!   Agent Skill の Concrete（skill.schema）のときだけ、作る人に聞く問いの順番（問1〜問3 と資料の問い）と別の場面の出し方を
//!   先頭に添える。ほかの Concrete のときは、SAS の手順そのままにする
//! - --value：Skill のフォルダの skill.json に、値を書かせる

use crate::ports::{SasRun, SkillWriter};
use crate::use_cases::approval::UNKNOWN;
use crate::use_cases::{Refused, trimmed};
use serde_json::{Value, json};

/// Agent Skill の Concrete の $id（作る人に聞く問いを添える Concrete）。
pub const SKILL_ID: &str = "urn:agents-skills-creator:skill:v1";

/// 書き方。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// 意味で書く手順を受け取る（target は置き場所）
    Semantic,
    /// 値を書かせる（target は Skill のフォルダ）
    Value(String),
}

/// 書かせた結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Created {
    /// 作る人に聞く問い（Agent Skill の Concrete で --semantic のときだけ）
    pub questions: Option<Value>,
    /// SAS の結果（書き換えない）
    pub sas: SasRun,
}

/// Agent Skill の Concrete で作るときの、作る人に聞く問いの順番と、別の場面の出し方。
fn questions() -> Value {
    json!({
        "questions": [
            {"id": "q1",
             "ask": "させたいことを、最近それを手でした場面1つで教えてください（いつ ・ 誰が ・ 何をしたか）。",
             "fills": ["/purpose/who", "/purpose/when", "/role/does"],
             "how": "場面から、使う人（who）・ 使う場面（when）・ 受け持つこと（does）を取る。この場面は examples の最初の例（from「作るときの場面」）にもする"},
            {"id": "q2",
             "ask": "終わったとき、手元に何があれば成功ですか。",
             "fills": ["/purpose/gets"],
             "how": "答えを、使い終えたときに手元にある状態として書く。completion_criteria の根拠（from「問2 の答え」）にもする"},
            {"id": "q3",
             "ask": "似ているけれど、させないことを選んでください。",
             "fills": ["/role/does_not"],
             "how": "AI が、取り違えやすい隣の仕事の候補を3つほど挙げ、作る人が選ぶ。選んだものを does_not に入れ、from を「問3 の答え」にする"},
            {"id": "q4",
             "ask": "元にする資料は何ですか（名前と、手元の置き場所）。",
             "fills": ["/references"],
             "when": "問1 の答えの役割（does）に、資料 ・ 原典 ・ 知見に基づく仕事が入るときだけ、問3 のあとに聞く。入らなければ聞かず、references を書かない",
             "how": "答えの資料を Skill のフォルダの中（references/ など）に置き、references の1件に name ・ path（Skill のフォルダからの相対）を入れ、holds ・ form ・ read_by（読むステップの番号）は steps から導き、from を「資料の問いの答え」にする"}
        ],
        "scenes": {
            "count": 2,
            "ask": "問いのあと、同じ目的のまま別の場面を2つ出し、「同じ」「違う」だけで答えてもらう",
            "same": "「同じ」の場面は examples に足す（from「別の場面で「同じ」」）",
            "different": "「違う」の場面は examples に入れない。理由が言われたら、受け持たないことの候補として承認の材料に出す"
        },
        "unknown": format!("どの問いも「{UNKNOWN}」を選べる。会話を止めず、その所に「{UNKNOWN}」と入れて進める（validate の承認の材料で推測の一覧の先頭に出る）")
    })
}

/// SAS に Skill の項目を書かせる。
///
/// # Errors
/// 置き場所かフォルダか Concrete が無い（usage）・ sas を呼べない（sas_unavailable）とき。
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
        usage("置き場所（--semantic）か Skill のフォルダ（--value）を渡してください")
    })?;
    let concrete =
        concrete.ok_or_else(|| usage("--concrete で、従う Concrete の URN を渡してください"))?;
    let mode =
        mode.ok_or_else(|| usage("--semantic か --value <JSON> のどちらかを渡してください"))?;
    let unavailable = |message: String| Refused {
        code: "sas_unavailable",
        message,
    };
    match mode {
        Mode::Semantic => {
            let path = format!("{}/<name>/skill.json", trimmed(target));
            let run = sas.create_semantic(&path, concrete).map_err(unavailable)?;
            Ok(Created {
                questions: (concrete == SKILL_ID).then(questions),
                sas: run,
            })
        }
        Mode::Value(value) => {
            let path = format!("{}/skill.json", trimmed(target));
            let run = sas
                .create_value(&path, concrete, &value)
                .map_err(unavailable)?;
            Ok(Created {
                questions: None,
                sas: run,
            })
        }
    }
}
