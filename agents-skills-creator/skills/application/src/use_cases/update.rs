//! UC3 直す：skills update。中で sas update を呼び、結果は書き換えずに返す。
//!
//! - --semantic --task <外れた具体の文>：意味で直す手順を受け取る（何も書かない）。直す流れ（外れた具体を examples に足す形 ・
//!   直す層の案 ・ 選んだ層より下の導き直し ・ 層の記録 ・ 承認してから書く ・ 例を全部動かす）を添える。足す例は返事で見せるだけで、
//!   書くのは承認のあと1回
//! - --patch <JSON Patch> --if-match <ハッシュ> --reason <理由>：作る人が承認した直しを書かせる。理由は「層：<名前>」で
//!   始め、選んだ層を SAS の記録に残す（新しい欄は足さない）。--if-match で、手順を受け取ったときの中身から変わって
//!   いないことを SAS に確かめさせる
//!
//! 作る人が承認するまで Skill は書き換えない（最低保証）。書くのは承認のあとの --patch だけ。

use crate::ports::{SasRun, SkillWriter};
use crate::use_cases::approval::UNKNOWN;
use crate::use_cases::create::GUESS;
use crate::use_cases::{Refused, trimmed};
use serde_json::{Value, json};

/// 直す層（上ほど狭く、下ほど広く直る）と、選んだときに導き直す項目。
pub const LAYERS: [(&str, &str, &[&str]); 5] = [
    ("examples", "例だけを足す。ほかの項目は変えない", &[]),
    ("edge_cases", "分かれ道を足す ・ 直す", &["/examples"]),
    (
        "rules",
        "決まりを足す ・ 直す",
        &["/edge_cases", "/examples"],
    ),
    (
        "role",
        "受け持つこと ・ 受け持たないことを直す",
        &[
            "/description",
            "/preconditions",
            "/inputs",
            "/steps",
            "/rules",
            "/edge_cases",
            "/references",
            "/examples",
        ],
    ),
    (
        "purpose",
        "目的を直す",
        &[
            "/role",
            "/description",
            "/preconditions",
            "/inputs",
            "/outputs",
            "/steps",
            "/rules",
            "/edge_cases",
            "/minimal_guarantees",
            "/completion_criteria",
            "/references",
            "/examples",
        ],
    ),
];

/// 理由の頭（選んだ層を記録する形）。
pub const LAYER_PREFIX: &str = "層：";

/// 直し方。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// 外れた具体の文を渡して、直す手順を受け取る
    Semantic(String),
    /// 承認した直しを書かせる
    Patch {
        /// JSON Patch
        patch: String,
        /// 手順を受け取ったときの中身のハッシュ
        if_match: Option<String>,
        /// 理由（層：<名前> で始める）
        reason: Option<String>,
    },
}

/// 直させた結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Updated {
    /// 直す流れ（--semantic のときだけ）
    pub flow: Option<Value>,
    /// SAS の結果（書き換えない）
    pub sas: SasRun,
}

/// 直す流れ。打ったセッション（AI）が読んで進める指示。値の直し方そのものは sas の手順に従う。
fn flow(folder: &str) -> Value {
    let layers: Vec<Value> = LAYERS
        .iter()
        .map(|(id, fixes, rederive)| json!({"id": id, "fixes": fixes, "rederive": rederive}))
        .collect();
    json!({
        "add_example": {
            "from": "使われて外れた具体",
            "fields": ["id", "from", "covers", "caller", "given", "input", "expected", "behavior"],
            "how": "外れた具体（頼んだこと ・ 出たもの ・ 期待したもの）から、examples に足す例の形を作り、直す前と後の像と一緒に返事で見せる。書くのは承認のあと1回（--patch に、足す例と選んだ層の直しを一緒に入れる）。input は頼んだこと、expected は期待したもの、given は頼んだときに手元にあったもの、behavior は使い手が何をすると何が変わるか。covers は、この例が確かめる完了条件か分かれ道の id"
        },
        "layers": layers,
        "choose": format!("外れ方から、直す層を推奨付きで1つ示す（examples が最も狭く、purpose が最も広く直る）。層ごとに、直す前と後の像（完成の像の、頼む文と返る答え）を並べ、作る人が1つ選ぶ。推測した所には{GUESS}を付ける"),
        "rederive": "選んだ層を直したら、その層の rederive に並ぶ項目を、直した値から導き直す。rederive に無い項目は変えない",
        "record": format!("書くときの --reason を「{LAYER_PREFIX}<選んだ層の id>」で始める（例：{LAYER_PREFIX}examples）。SAS の記録に残り、validate の承認の材料の changes に出る"),
        "approve_then_write": format!("直す前と後（sas の手順の current と、直した値）を所ごとに並べて見せ、作る人が承認してから、skills update {folder} --patch <JSON Patch> --if-match <sas の手順の target.hash> --reason {LAYER_PREFIX}<層> で書かせる。承認の前には書かない"),
        "finish": [
            format!("skills write-skill-md {folder} と skills validate {folder} を打つ"),
            "validate の承認の材料の delegate のとおり、別の会話で例を全部（足した例を含む）動かして判定させる",
            "承認の材料の changes（前と後）と、例の結果を並べて見せ、作る人が承認するか「ここが違う」の一言を受け取る"
        ],
        "presentation": "チャットの中で見せるので、表 ・ コードブロック ・ 木構造で出す。図を出せるツールなら図でもよい",
        "unknown": format!("作る人は「{UNKNOWN}」と答えてよい。その層は推奨のまま進め、承認の材料で示す")
    })
}

/// 層の名前が理由に入っているか（「層：<LAYERS の id>」で始まる）。
fn layer_of(reason: &str) -> Option<&'static str> {
    let rest = reason.strip_prefix(LAYER_PREFIX)?;
    LAYERS
        .iter()
        .map(|(id, _, _)| *id)
        .find(|id| rest.starts_with(id))
}

/// 直し方が、sas を打つ前に断る形でないかを確かめる（--patch に --if-match と「層：<名前>」の理由があるか）。
///
/// # Errors
/// --patch に --if-match か「層：<名前>」の理由が無いとき（usage）。
pub fn check_mode(mode: &Mode) -> Result<(), Refused> {
    let Mode::Patch {
        if_match, reason, ..
    } = mode
    else {
        return Ok(());
    };
    if if_match.is_none() {
        return Err(Refused {
            code: "usage",
            message: "--if-match に、直す手順を受け取ったときの中身のハッシュ（sas の手順の target.hash）を渡してください".to_string(),
        });
    }
    if reason.as_deref().and_then(layer_of).is_none() {
        let ids: Vec<&str> = LAYERS.iter().map(|(id, _, _)| *id).collect();
        return Err(Refused {
            code: "usage",
            message: format!(
                "--reason を、選んだ層の「{LAYER_PREFIX}<{}>」で始めてください",
                ids.join(" ・ ")
            ),
        });
    }
    Ok(())
}

/// SAS に Skill の項目を直させる。
///
/// # Errors
/// フォルダか直し方が無い ・ --patch に --if-match か「層：<名前>」の理由が無い（usage）・ sas を呼べない（sas_unavailable）とき。
pub fn update(
    folder: Option<&str>,
    mode: Option<Mode>,
    sas: &dyn SkillWriter,
) -> Result<Updated, Refused> {
    let usage = |message: String| Refused {
        code: "usage",
        message,
    };
    let folder =
        folder.ok_or_else(|| usage("直す Skill のフォルダを渡してください".to_string()))?;
    let mode = mode.ok_or_else(|| {
        usage(
            "--semantic --task <外れた具体> か --patch <JSON Patch> のどちらかを渡してください"
                .to_string(),
        )
    })?;
    let unavailable = |message: String| Refused {
        code: "sas_unavailable",
        message,
    };
    let folder = trimmed(folder);
    let path = format!("{folder}/skill.json");
    match mode {
        Mode::Semantic(task) => {
            let run = sas.update_semantic(&path, &task).map_err(unavailable)?;
            Ok(Updated {
                flow: Some(flow(folder)),
                sas: run,
            })
        }
        Mode::Patch {
            patch,
            if_match,
            reason,
        } => {
            check_mode(&Mode::Patch {
                patch: String::new(),
                if_match: if_match.clone(),
                reason: reason.clone(),
            })?;
            let (if_match, reason) = (if_match.unwrap_or_default(), reason.unwrap_or_default());
            let run = sas
                .update_patch(&path, &patch, &if_match, &reason)
                .map_err(unavailable)?;
            Ok(Updated {
                flow: None,
                sas: run,
            })
        }
    }
}
