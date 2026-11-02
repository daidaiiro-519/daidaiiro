//! UC1 作るの new：Skill を新しく作る手順を返す。creator は AI を起動しないので、打ったセッションがこの手順で作る人に
//! 問1〜問3（役割が資料に基づくときは資料の問いも）を1往復に1問ずつ聞き、別の場面2つに「同じ ・ 違う」を聞き、答えから skill.json の値を組んで SAS に書かせ、
//! build → check → confirm を打つ。何も書かない。
//!
//! 値を組む決まりは、Concrete（published-language/v1/skill.schema.json）の項目ごとの説明 ・ 読み方 ・ rule を写す。
//! 作る人の答えから入れるのは purpose と role（と、資料の問いに答えたときの references）だけで、ほかは導く
//! （confirm で推測として見せる）。
//!
//! 同じ形で作るとき（new --like、UC1 の拡張 2a）は、元の Skill の skill.json を読み、同じ形の Skill の間で同じになる項目
//! （domain の表で Basis が Shape のもの）を元の値のまま既定にし、分野と場面 ・ 原典と知見で変わる項目だけを聞く ・ 作り直す。
//! 元の Skill には何も書かない。

use crate::ports::SkillFiles;
use crate::use_cases::confirm::UNKNOWN;
use crate::use_cases::{Refused, read_skill, trimmed};
use serde_json::{Value, json};
use skills_domain::services::derivation::{ANSWERS, Basis, basis, derived_from};

/// Skill の項目の Concrete（この文脈の公開された言葉）。
pub const CONCRETE: &str = include_str!("../../../published-language/v1/skill.schema.json");

/// Concrete の $id。
pub const CONCRETE_ID: &str = "urn:agents-skills-creator:skill:v1";

/// 手順の形の $id。
pub const PROCEDURE: &str = "urn:agents-skills-creator:skills:new-procedure:v1";

/// 項目ごとに、答えから入れるか導くかと、何から入れる ・ 導くか。導く項目は domain の1つの表（derivation）から引く。
fn source(field: &str) -> (&'static str, &'static str) {
    match field {
        "purpose" => ("answer", "問1（who ・ when）と問2（gets）の答え"),
        "role" => (
            "answer",
            "問1（does）と問3（does_not。AI が挙げた隣の仕事の候補から選んだもの）の答え",
        ),
        // 参照する資料は、資料の問いの答え（名前と置き場所）から入れる
        "references" => (
            "answer",
            derived_from("references").unwrap_or("資料の問いの答えから"),
        ),
        other => (
            "derive",
            derived_from(other).unwrap_or("purpose と role から"),
        ),
    }
}

/// Concrete の項目ごとの、値を組む決まり。
fn compose(concrete: &Value) -> Vec<Value> {
    concrete
        .get("properties")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter(|(name, _)| name.as_str() != "$schema")
        .map(|(name, f)| {
            let (src, from) = source(name);
            let mut v = json!({"field": format!("/{name}"), "source": src, "from": from});
            for k in ["title", "description"] {
                if let Some(t) = f.get(k) {
                    v[k] = t.clone();
                }
            }
            for k in ["meaning", "when"] {
                if let Some(t) = f.pointer(&format!("/x-prompt/{k}")) {
                    v[k] = t.clone();
                }
            }
            let rules: Vec<Value> = f
                .pointer("/x-prompt/rules")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|r| json!({"id": r.get("id").cloned().unwrap_or_default(), "rule": r.get("rule").cloned().unwrap_or_default()}))
                .collect();
            v["rules"] = Value::Array(rules);
            v
        })
        .collect()
}

/// 置き場所にある Skill（skill.json を読めるフォルダ）の folder ・ name ・ description。置き場所を読めなければ空。
fn existing(place: &str, files: &dyn SkillFiles) -> Vec<Value> {
    files
        .folders(place)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|folder| {
            let skill = read_skill(files, &folder).ok()?;
            let text = |k: &str| {
                skill
                    .get(k)
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string()
            };
            Some(
                json!({"folder": folder, "name": text("name"), "description": text("description")}),
            )
        })
        .collect()
}

/// Skill を作る手順を返す。place は Skill を置くフォルダの親（無ければ埋める所のまま）。
/// 置き場所にある Skill を existing に並べ、同じ形の Skill に気づいたら示して選ばせる（UC1 の拡張 4a）。
#[must_use]
pub fn new_procedure(place: Option<&str>, files: &dyn SkillFiles) -> Value {
    let concrete: Value = serde_json::from_str(CONCRETE).unwrap_or_default();
    let existing = place
        .map(|p| existing(p.trim_end_matches('/'), files))
        .unwrap_or_default();
    let place = place
        .map(|p| p.trim_end_matches('/').to_string())
        .unwrap_or_else(|| "<Skill の置き場所>".to_string());
    let folder = format!("{place}/<name>");
    let mut procedure = json!({
        "$schema": PROCEDURE,
        "kind": "procedure",
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
            "different": "「違う」の場面は examples に入れない。理由が言われたら、受け持たないことの候補として確認に出す"
        },
        "unknown": format!("どの問いも「{UNKNOWN}」を選べる。会話を止めず、その所に「{UNKNOWN}」と入れて進める（confirm で推測の一覧の先頭に出る）。目的の who と gets が{UNKNOWN}ままなら、confirm は承認を受け付けない"),
        "compose": compose(&concrete),
        "write": {"command": format!("sas create {folder}/skill.json --concrete {CONCRETE_ID} --value <JSON>")},
        "then": [
            format!("agents-skills-creator build {folder}"),
            format!("agents-skills-creator check {folder}"),
            format!("agents-skills-creator confirm {folder}")
        ],
        "constraints": [
            "1往復に1問だけ聞く",
            "会話に既に出ていることは聞き直さない",
            "name は聞かず、AI が付けて確認に出す",
            "purpose と role のほかの項目は、答えから導き、確認で推測として見せる",
            "skill.json は書かず、write の command で SAS に書かせる",
            "creator は AI を起動しない。意味の判定と例を動かすのは、confirm の delegate のとおり別の会話に任せる",
            "承認するか「〇〇は違う、△△」の一言を受け取るまで、配らない",
            "問1〜問3 の答えが、existing のどれかと目的の種類 ・ 役割 ・ 手順で同じなら、その Skill を作る人に示し、同じ形にするかを選んでもらう（UC1 の拡張 4a）。同じ形にするなら agents-skills-creator new --like <その Skill の folder> の手順に移る"
        ]
    });
    procedure["existing"] = Value::Array(existing);
    // same_shape_as は同じ形で作るとき（new --like）だけ書く
    if let Some(c) = procedure["compose"].as_array_mut() {
        c.retain(|f| f["field"] != "/same_shape_as");
    }
    procedure
}

/// 同じ形で作るときの、項目ごとの扱い：元の値のまま（keep）・ 作る人に聞く（ask）・ 新しい答えから作り直す（derive）。
/// 何によって決まるか（Basis）は domain の表から引く。purpose は欄ごとに、ほかは skill.json の根の項目ごとに見る。
fn like_fields(concrete: &Value) -> Vec<(String, &'static str)> {
    let mut fields: Vec<String> = ANSWERS.iter().map(|(f, _)| (*f).to_string()).collect();
    fields.extend(
        concrete
            .get("properties")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .map(|(name, _)| name.clone())
            .filter(|n| !matches!(n.as_str(), "$schema" | "purpose" | "role" | "same_shape_as")),
    );
    fields
        .into_iter()
        .filter_map(|f| {
            let how = match basis(&f)? {
                Basis::Shape => "keep",
                // 作る人の答えから入れる所は聞き、答えから導く所は新しい答えから作り直す
                Basis::Domain if derived_from(&f).is_none() => "ask",
                Basis::Domain => "derive",
                Basis::Material => "ask",
            };
            Some((format!("/{f}"), how))
        })
        .collect()
}

/// 既にある Skill（folder）と同じ形で作る手順を返す（UC1 の拡張 2a）。place が無ければ、元の Skill と同じ置き場所。
///
/// # Errors
/// フォルダが無い（usage）・ 元の skill.json を読めない（not_found）・ JSON でない（invalid_input）とき。
pub fn new_like_procedure(
    folder: Option<&str>,
    place: Option<&str>,
    files: &dyn SkillFiles,
) -> Result<Value, Refused> {
    let Some(folder) = folder else {
        return Err(Refused {
            code: "usage",
            message: "--like には、同じ形にする既にある Skill のフォルダを渡してください"
                .to_string(),
        });
    };
    let original = read_skill(files, folder)?;
    let folder = trimmed(folder);
    let place = place.map_or_else(
        || folder.rsplit_once('/').map_or(".", |(p, _)| p).to_string(),
        |p| p.trim_end_matches('/').to_string(),
    );
    let name = original
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let concrete: Value = serde_json::from_str(CONCRETE).unwrap_or_default();
    let fields = like_fields(&concrete);
    let pick = |how: &str| -> Vec<String> {
        fields
            .iter()
            .filter(|(_, h)| *h == how)
            .map(|(f, _)| f.clone())
            .collect()
    };
    let keep: Vec<Value> = pick("keep")
        .into_iter()
        .filter_map(|f| {
            original
                .pointer(&f)
                .map(|v| json!({"field": f, "value": v}))
        })
        .collect();
    let does: Vec<&str> = original
        .pointer("/role/does")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let steps = original
        .get("steps")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let references: Vec<String> = original
        .get("references")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|r| {
            format!(
                "{}（{}）",
                r.get("name").and_then(Value::as_str).unwrap_or_default(),
                r.get("path").and_then(Value::as_str).unwrap_or_default()
            )
        })
        .collect();
    let mut q4 = json!({
        "id": "q4",
        "fills": ["/references"],
        "how": "答えの資料を Skill のフォルダの中に置き、元の Skill の資料と同じ name ・ path ・ form ・ read_by にして、holds（中身）だけを新しい資料で書き、from を「資料の問いの答え」にする。元に無い資料が要ると言われたら（UC1 の拡張 2a2a）、references に足し、確認で「足した所」として示して承認してもらう"
    });
    if references.is_empty() {
        q4["ask"] = json!("元にする資料は何ですか（名前と、手元の置き場所）。");
        q4["when"] = json!(
            "元の Skill は資料を持たない。問1 の答えの場面で、資料 ・ 原典 ・ 知見に基づく仕事が要るときだけ聞く（元に無いものを足す、UC1 の拡張 2a2a）"
        );
    } else {
        q4["ask"] = json!(format!(
            "元の Skill の資料 {} に当たる、この分野の資料は何ですか（名前と、手元の置き場所）。元に無い資料が要れば、それも教えてください。",
            references.join(" ・ ")
        ));
    }
    let keep_fields = pick("keep");
    let mut compose = compose(&concrete);
    for c in &mut compose {
        let field = c["field"].as_str().unwrap_or_default().to_string();
        if keep_fields.contains(&field) {
            c["source"] = json!("keep");
            c["from"] = json!(format!(
                "元の Skill（{name}）の値（like.keep）をそのまま使う"
            ));
        } else if field == "/purpose" {
            c["from"] = json!(format!(
                "who ・ gets は元の Skill（{name}）の値（like.keep）、when は問1 の答え"
            ));
        } else if field == "/same_shape_as" {
            c["source"] = json!("answer");
            c["from"] = json!(format!("問0 の答え（同じ形なら「{name}」）"));
        }
    }
    let new_folder = format!("{place}/<name>");
    Ok(json!({
        "$schema": PROCEDURE,
        "kind": "procedure",
        "like": {
            "folder": folder,
            "name": name,
            "keep": keep,
            "ask": pick("ask"),
            "derive": pick("derive")
        },
        "questions": [
            {"id": "q0",
             "ask": format!(
                 "「{name}」と、役割（{}）と手順（{steps} ステップ）が同じで、分野と原典 ・ 知見だけが違う Skill を作りますか。",
                 does.join(" ・ ")
             ),
             "fills": ["/same_shape_as"],
             "how": format!(
                 "同じなら same_shape_as に「{name}」を入れて次の問いへ進む。役割や手順が違うなら（UC1 の拡張 2a1a）、違う所を作る人に示し、agents-skills-creator new {place} の3問へ戻る"
             )},
            {"id": "q1",
             "ask": "この分野で、させたいことを最近手でした場面1つで教えてください（いつ ・ 何をしたか）。",
             "fills": ["/purpose/when"],
             "how": "場面から使う場面（when）を取り、examples の最初の例（from「作るときの場面」）にもする。who と受け持つことは元の値のまま。examples の id ・ covers ・ caller は元の例と同じ並びにし、rules ・ edge_cases ・ completion_criteria が指す id を残す"},
            q4
        ],
        "scenes": {
            "count": 2,
            "ask": "問いのあと、この分野で別の場面を2つ出し、「同じ」「違う」だけで答えてもらう",
            "same": "「同じ」の場面は examples に足す（from「別の場面で「同じ」」）",
            "different": "「違う」の場面は examples に入れない。役割の違いが理由なら、同じ形ではない（UC1 の拡張 2a1a）"
        },
        "unknown": format!("どの問いも「{UNKNOWN}」を選べる。会話を止めず、その所に「{UNKNOWN}」と入れて進める（confirm で推測の一覧の先頭に出る）"),
        "compose": compose,
        "write": {"command": format!("sas create {new_folder}/skill.json --concrete {CONCRETE_ID} --value <JSON>")},
        "then": [
            format!("agents-skills-creator build {new_folder}"),
            format!("agents-skills-creator check {new_folder}"),
            format!("agents-skills-creator confirm {new_folder}")
        ],
        "constraints": [
            "1往復に1問だけ聞く",
            "会話に既に出ていることは聞き直さない",
            "like.keep の値は書き換えずに使う（作る人が答えた所でだけ元と違う）",
            "like.derive の所（name ・ description ・ examples）は、新しい答えから作り直す。name は聞かず、AI が付けて確認に出す",
            format!("書く値の same_shape_as に「{name}」を入れる"),
            format!("元の Skill（{folder}）には何も書かない"),
            "skill.json は書かず、write の command で SAS に書かせる",
            "creator は AI を起動しない。意味の判定と例を動かすのは、confirm の delegate のとおり別の会話に任せる",
            "承認するか「〇〇は違う、△△」の一言を受け取るまで、配らない"
        ]
    }))
}
