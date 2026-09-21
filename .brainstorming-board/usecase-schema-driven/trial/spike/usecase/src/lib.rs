//! ユースケース駆動（concrete schema）── 基盤の上へ注入される側。
//!
//! **方法論の語彙は、全部こちらに在る** ── 集約 ・ ユースケース ・ シナリオ ・ テスト。
//! 基盤はこれらを1つも知らない。基盤にとっては「ID を持つ節」と「外の識別子」である。

use base::{Bindings, Decl, Registry};
use std::collections::BTreeSet;

/// 仕様のファイル ── **事業領域で切ったフォルダの、根から見た位置**を持つ。
///
/// 道具はフォルダを走査しない（論点1）。一覧は、この側が渡す。
pub fn spec_files() -> Vec<base::SpecFile> {
    vec![
        // 根に置く ── **全体のコンセプトだからである**（契約）。
        base::SpecFile {
            path: "business-domain.json".into(),
            decls: base::read(include_str!("../spec/宅配のサービス/business-domain.json"))
                .expect("読めない"),
        },
        base::SpecFile {
            path: "subdomains/all.json".into(),
            decls: base::read(include_str!("../spec/宅配のサービス/subdomains/all.json"))
                .expect("読めない"),
        },
        base::SpecFile {
            path: "contexts/all.json".into(),
            decls: base::read(include_str!("../spec/宅配のサービス/contexts/all.json"))
                .expect("読めない"),
        },
        base::SpecFile {
            path: "usecases/all.json".into(),
            decls: base::read(include_str!("../spec/宅配のサービス/usecases/all.json"))
                .expect("読めない"),
        },
    ]
}

/// 宣言（文書エンティティ）を読む。
pub fn decls() -> Vec<Decl> {
    spec_files().into_iter().flat_map(|f| f.decls).collect()
}

/// 根に置く種類 ── **この方法論では事業領域である**。基盤は知らない。
pub const ROOT_KIND: &str = "business-domain";

/// 検査の実体・関門・射影を、名前で登録する。
///
/// **この関数が、配線を行う唯一の場所である**（合成の根）。
pub fn wire() -> Registry {
    let mut r = Registry::new();

    r.check("check.precondition", |_| Ok(()));
    r.check("check.statusIs", |_| Ok(()));
    r.check("check.sumWithin", |d| {
        // 本物なら実体の値を見る。ここでは宣言の形だけを見る試験である。
        if d.ops.iter().any(|o| o.writes.is_empty()) {
            Err("書き換える先が空の操作がある".into())
        } else {
            Ok(())
        }
    });

    // 呼び出し口の関門 ── 登録された操作だけが呼べる。
    r.op("出荷指示::作成する");
    r.op("出荷指示::明細を足す");
    r.op("出荷指示::確定する");
    r.op("出荷を確定する::実行する");
    r.op("配達予定日を計算する::計算する");

    // 人が読む形は HTML である（決まり）。**射影を持たない型は、承認を通れない。**
    r.render("render.aggregate", |d| {
        let mut s = format!("<h1>{}（集約）</h1>\n<h2>不変条件</h2>\n<ul>\n", d.name);
        for x in &d.rules {
            s.push_str(&format!("<li>{}　<code>{}</code></li>\n", x.name, x.predicate));
        }
        s.push_str("</ul>\n<h2>操作</h2>\n<ul>\n");
        for o in &d.ops {
            s.push_str(&format!(
                "<li><b>{}</b> ── 事前 {:?} ／ 事後 {:?} ／ 書き換える {:?}</li>\n",
                o.name, o.pre, o.post, o.writes
            ));
        }
        s.push_str("</ul>\n<h2>シナリオ</h2>\n<ul>\n");
        for x in &d.nodes {
            s.push_str(&format!("<li><code>{}</code>　{}</li>\n", x.id, x.name));
        }
        s.push_str("</ul>\n");
        s
    });
    // 外の実行系が読む形は MD である（別の契約）。**人が承認する HTML とは、用途が違う。**
    // 雛形は、その実行系の決まりに合わせて**注入する側**が持つ ── 基盤は名前で呼ぶだけである。
    r.render("render.skill.md", |d| {
        let mut s = String::from("---\n");
        s.push_str(&format!("name: {}\n", d.id));
        s.push_str(&format!("description: {}（宣言から導出。手で書かない）\n", d.name));
        s.push_str("---\n\n");
        s.push_str(&format!("# {}\n\n## 守ること\n", d.name));
        for x in &d.rules {
            s.push_str(&format!("- {}\n", x.name));
        }
        s.push_str("\n## できること\n");
        for o in &d.ops {
            s.push_str(&format!("- `{}` ── 事前 {:?}\n", o.name, o.pre));
        }
        s
    });
    // 変換機その2 ── **JSON を出す**（Kiro の形）。形式が違っても、口は同じ `render` である。
    r.render("render.kiro.agent.json", |d| {
        let tools: Vec<String> = d.ops.iter().map(|o| format!("{:?}", o.name)).collect();
        format!(
            "{{\n  \"name\": {:?},\n  \"description\": {:?},\n  \"tools\": [{}]\n}}\n",
            d.id, d.name, tools.join(", ")
        )
    });
    // 変換機その3 ── **JSON を出す**（Claude Code の settings.json の形）。
    r.render("render.claude.settings.json", |d| {
        let ms: Vec<String> = d
            .ops
            .iter()
            .map(|o| format!("{{\"matcher\": {:?}, \"hooks\": []}}", o.name))
            .collect();
        format!("{{\n  \"hooks\": {{\n    \"PreToolUse\": [{}]\n  }}\n}}\n", ms.join(", "))
    });
    // 変換機その4 ── **TOML を出す**（Codex の Agent の形）。
    // 鍵名は原文で照合したものだけを使う ── `name` ・ `description` ・ `sandbox_mode` ・
    // `developer_instructions`（developers.openai.com/codex/agent-configuration/subagents:421-426）。
    r.render("render.codex.agent.toml", |d| {
        let mut s = format!("name = {:?}\ndescription = {:?}\n", d.id, d.name);
        s.push_str("sandbox_mode = \"read-only\"\n");
        s.push_str("developer_instructions = \"\"\"\n");
        for x in &d.rules {
            s.push_str(&format!("{} を保つこと。\n", x.name));
        }
        for o in &d.ops {
            s.push_str(&format!("{} は、事前条件 {:?} を満たすときだけ実行する。\n", o.name, o.pre));
        }
        s.push_str("\"\"\"\n");
        s
    });
    // 事業領域の射影 ── **概念（大枠）が先、当てはめ（リポジトリなど）は後に置く。**
    // 当てはめを先に書くと、抽象の高さが下がって読まれる。
    r.render("render.business-domain", |d| {
        let g = |k: &str| d.body.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
        let mut s = format!(
            "<h1>{}（事業領域）</h1>\n<p>顧客に提供するサービスの大枠は <b>{}</b> である</p>\n<p>{}</p>\n",
            d.name, g("scope"), g("why")
        );
        if let Some(a) = d.body.get("appliedTo") {
            s.push_str(&format!(
                "<h2>この開発での当てはめ</h2>\n<p><b>{}</b> 1つ（{}）── <b>当てはめであって、事業領域の定義ではない</b></p>\n",
                a.get("kind").and_then(|x| x.as_str()).unwrap_or(""),
                a.get("name").and_then(|x| x.as_str()).unwrap_or("")
            ));
        }
        s
    });
    // 業務領域の射影 ── 分類は<b>外から取り込む前提</b>なので、決めた日と理由を並べて出す。
    r.render("render.subdomain", |d| {
        let g = |k: &str| d.body.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
        format!(
            "<h1>{}（業務領域）</h1>\n<p>分類は <b>{}</b> である（{} 時点）</p>\n<p>{}</p>\n",
            d.name, g("category"), g("asOf"), g("why")
        )
    });
    // 業務サービスの射影 ── **状態を持たず、業務ロジックだけを持つ。**
    r.render("render.service", |d| {
        let mut s = format!("<h1>{}（業務サービス）</h1>\n", d.name);
        if let Some(w) = d.body.get("why").and_then(|x| x.as_str()) {
            s.push_str(&format!("<p>{w}</p>\n"));
        }
        s.push_str("<h2>計算</h2>\n<ul>\n");
        for o in &d.ops {
            s.push_str(&format!("<li><b>{}</b> ── 事前 {:?}／事後 {:?}（書き換えない）</li>\n",
                                o.name, o.pre, o.post));
        }
        s.push_str("</ul>\n<h2>シナリオ</h2>\n<ul>\n");
        for x in &d.nodes {
            s.push_str(&format!("<li><code>{}</code>　{}</li>\n", x.id, x.name));
        }
        s.push_str("</ul>\n");
        s
    });
    // 区切られた文脈の射影 ── **語の一覧は、ここにだけ出る**（文脈の内側でしか通用しない）。
    r.render("render.context", |d| {
        let mut s = format!("<h1>{}（区切られた文脈）</h1>\n", d.name);
        if let Some(o) = d.body.get("owner").and_then(|x| x.as_str()) {
            s.push_str(&format!("<p>担当は {o} である（1つの文脈に1チーム）</p>\n"));
        }
        s.push_str("<h2>この文脈で通用する語</h2>\n<ul>\n");
        if let Some(xs) = d.body.get("language").and_then(|x| x.as_array()) {
            for x in xs {
                s.push_str(&format!(
                    "<li><b>{}</b> ── {}</li>\n",
                    x.get("term").and_then(|v| v.as_str()).unwrap_or(""),
                    x.get("means").and_then(|v| v.as_str()).unwrap_or("")
                ));
            }
        }
        s.push_str("</ul>\n");
        s
    });
    r.render("render.usecase", |d| {
        format!("<h1>{}（ユースケース）</h1>\n<p>操作 {} 件</p>\n", d.name, d.ops.len())
    });
    r
}

/// 対応表 ── AI が MCP で申告した内容が、ここへ入る。
///
/// **結ぶのは、シナリオの ID と、テストの一覧が出す識別子である。**
pub fn bindings() -> Bindings {
    let mut b = Bindings::new();
    b.bind("SC-01J7Q4M", "usecase::tests::明細が0件のとき確定できない");
    b.bind("SC-01J7Q4N", "usecase::tests::明細が1件あれば確定できる");
    b.bind("SC-01J8E5M", "usecase::tests::確定していなければ計算できない");
    b
}

/// テストの一覧 ── その言語のテスト実行系が列挙したもの。
///
/// **道具は、これを外から受け取るだけである**（ここでは受け取ったことにする）。
pub fn test_list() -> BTreeSet<String> {
    ["usecase::tests::明細が0件のとき確定できない",
     "usecase::tests::明細が1件あれば確定できる",
     "usecase::tests::確定していなければ計算できない"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// 「確定する」の実装。**書き込みは必ず関門を通す。**
pub fn confirm(tx: &mut base::Tx) {
    tx.write("出荷指示");
    tx.write("出荷明細");
}

/// 宣言していない先へ書く実装 ── 抜け道の再現に使う。
pub fn confirm_smuggling(tx: &mut base::Tx) {
    tx.write("出荷指示");
    tx.write("出荷明細");
    tx.write("在庫"); // 宣言していない
}

/// 出す先1件 ── 変換機の名前と、置き場所の決まり。
///
/// **置き場所は AI ツールごとに違うので、注入する側が持つ**（論点2 の決着）。
pub struct Target {
    pub render: &'static str,
    pub path: fn(&str) -> String,
    /// **どの種類の宣言に当てるか。**種類は注入する側の語彙である。
    pub kinds: &'static [&'static str],
}

/// 出す先の一覧。**宣言 × ここ＝出る成果物の集合**である。
pub fn targets() -> Vec<Target> {
    vec![
        Target { render: "render.skill.md",
                 path: |id| format!(".claude/skills/{id}/SKILL.md"),
                 kinds: &["aggregate", "use-case", "domain-service"] },
        Target { render: "render.kiro.agent.json",
                 path: |id| format!(".kiro/agents/{id}.json"),
                 kinds: &["aggregate", "use-case", "domain-service"] },
        Target { render: "render.claude.settings.json",
                 path: |id| format!(".claude/settings.{id}.json"),
                 kinds: &["aggregate"] },
        Target { render: "render.codex.agent.toml",
                 path: |id| format!(".codex/agents/{id}.toml"),
                 kinds: &["aggregate", "use-case", "domain-service"] },
        // 文脈は、語の一覧を出す先が1つだけである。
        Target { render: "render.context",
                 path: |id| format!(".schema/context/{id}.html"),
                 kinds: &["bounded-context"] },
        // 業務領域は、人が読む1枚だけを出す（実行系は読まない）。
        Target { render: "render.business-domain",
                 path: |id| format!(".schema/business-domain/{id}.html"),
                 kinds: &["business-domain"] },
        Target { render: "render.subdomain",
                 path: |id| format!(".schema/subdomain/{id}.html"),
                 kinds: &["subdomain"] },
    ]
}

/// 導出される成果物の名前 ── **宣言 × 出す先で決まる。数えられる。**
pub fn emitted_files() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for d in decls() {
        for t in targets() {
            if t.kinds.contains(&d.kind.as_str()) {
                out.insert((t.path)(&d.id));
            }
        }
    }
    out
}

/// 実際に置いてあるファイルの一覧 ── **道具は外から受け取るだけである**。
///
/// ここでは「決まりどおりに全部置いてある」状態を渡す。
pub fn placed_files() -> BTreeSet<String> {
    emitted_files()
}

/// 宣言が指している業務領域の ID を集める ── **参照の鍵は名前ではなく ID である**。
pub fn subdomain_refs() -> BTreeSet<String> {
    decls()
        .iter()
        .filter_map(|d| d.body.get("subdomain").and_then(|x| x.as_str()).map(String::from))
        .collect()
}

/// 宣言されている業務領域の ID。
pub fn subdomain_ids() -> BTreeSet<String> {
    decls()
        .iter()
        .filter(|d| d.kind == "subdomain")
        .map(|d| d.id.clone())
        .collect()
}

/// 宣言どうしの参照1件。**どの欄が、どの種類を指すか**は注入する側が決める。
pub struct Ref {
    /// 参照を持つ側の種類
    pub from: &'static str,
    /// 参照を書く欄の名前
    pub field: &'static str,
    /// 指す先の種類
    pub to: &'static str,
}

/// この方法論が持つ、宣言どうしの関係。
pub fn refs() -> Vec<Ref> {
    vec![
        Ref { from: "subdomain", field: "businessDomain", to: "business-domain" },
        Ref { from: "use-case", field: "subdomain", to: "subdomain" },
        Ref { from: "aggregate", field: "subdomain", to: "subdomain" },
        Ref { from: "use-case", field: "context", to: "bounded-context" },
        Ref { from: "aggregate", field: "context", to: "bounded-context" },
        Ref { from: "domain-service", field: "subdomain", to: "subdomain" },
        Ref { from: "domain-service", field: "context", to: "bounded-context" },
    ]
}

/// 関係の引き算 ── 1件の関係について、指した先と、指される側の一覧を突き合わせる。
///
/// **同じ `drift()` を使う。**基盤は関係を知らない。
pub fn relation_drift(r: &Ref) -> base::Drift {
    let ds = decls();
    let pointed: BTreeSet<String> = ds
        .iter()
        .filter(|d| d.kind == r.from)
        .filter_map(|d| d.body.get(r.field).and_then(|x| x.as_str()).map(String::from))
        .collect();
    let declared: BTreeSet<String> = ds
        .iter()
        .filter(|d| d.kind == r.to)
        .map(|d| d.id.clone())
        .collect();
    let mut b = base::Bindings::new();
    for id in &declared {
        b.bind(id, id);
    }
    base::drift(&declared, &b, &pointed)
}

/// 「ユースケースの集合が業務領域である」が成立しているか ── **空の業務領域を挙げる**。
pub fn empty_subdomains() -> BTreeSet<String> {
    let ds = decls();
    let used: BTreeSet<String> = ds
        .iter()
        .filter(|d| d.kind == "use-case")
        .filter_map(|d| d.body.get("subdomain").and_then(|x| x.as_str()).map(String::from))
        .collect();
    ds.iter()
        .filter(|d| d.kind == "subdomain" && !used.contains(&d.id))
        .map(|d| d.id.clone())
        .collect()
}

// ── 多重度の決まり ────────────────────────────────────────────
//
// 原典に照らして4本を1に固定し、1本は導出のままにする。
//   ユースケース → 業務領域 ／ ユースケース → 文脈 ／ 集約 → 文脈 ／ 集約 → 業務領域 ── **1件だけ**
//   業務領域 ↔ 文脈 ── **持たない**（ユースケースの2本から導出する。多対多を1に固定しない）

/// 参照の形の誤り ── **2件以上を書いたら誤りである**（多重度は1）。
pub fn ref_shape_errors_in(ds: &[Decl]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for r in refs() {
        for d in ds.iter().filter(|d| d.kind == r.from) {
            match d.body.get(r.field) {
                None => {}
                Some(v) if v.is_string() => {}
                Some(_) => {
                    out.insert(format!("{}.{} は1件だけである", d.id, r.field));
                }
            }
        }
    }
    out
}

/// 同じ文脈の中で、名前が重なっている宣言 ── **またいだ重複は、誤りではない**。
pub fn name_clashes_in(ds: &[Decl]) -> BTreeSet<String> {
    let mut seen: std::collections::BTreeMap<(String, String), usize> = Default::default();
    for d in ds {
        if let Some(c) = d.body.get("context").and_then(|x| x.as_str()) {
            *seen.entry((c.to_string(), d.name.clone())).or_insert(0) += 1;
        }
    }
    seen.into_iter()
        .filter(|(_, n)| *n > 1)
        .map(|((c, name), _)| format!("{c} の中に「{name}」が2件以上ある"))
        .collect()
}

/// ユースケースが2つ以上の文脈へ散らばっている業務領域。
///
/// **常時ゼロにしない** ── 原典は「一つの業務領域でも課題が複数あれば
/// 課題ごとに別のモデルを作るほうがよいことがある」と認めている。承認の境目で人が判定する。
pub fn subdomains_spanning_contexts_in(ds: &[Decl]) -> BTreeSet<String> {
    let mut m: std::collections::BTreeMap<String, BTreeSet<String>> = Default::default();
    for d in ds.iter().filter(|d| d.kind == "use-case") {
        let sd = d.body.get("subdomain").and_then(|x| x.as_str());
        let cx = d.body.get("context").and_then(|x| x.as_str());
        if let (Some(sd), Some(cx)) = (sd, cx) {
            m.entry(sd.to_string()).or_default().insert(cx.to_string());
        }
    }
    m.into_iter().filter(|(_, cs)| cs.len() > 1).map(|(sd, _)| sd).collect()
}

pub fn ref_shape_errors() -> BTreeSet<String> { ref_shape_errors_in(&decls()) }
pub fn name_clashes() -> BTreeSet<String> { name_clashes_in(&decls()) }
pub fn subdomains_spanning_contexts() -> BTreeSet<String> {
    subdomains_spanning_contexts_in(&decls())
}

/// 操作を持つのに、節を1件も持たない宣言 ── **振る舞いが宣言されていない**。
///
/// 操作が在るなら、確かめるべき振る舞いも在るはずである。
/// **常時ゼロにはしない** ── 宣言を書いてから節を足すまでの間は、正当にこの状態になる。
pub fn ops_without_nodes() -> BTreeSet<String> {
    decls()
        .iter()
        .filter(|d| !d.ops.is_empty() && d.nodes.is_empty())
        .map(|d| d.id.clone())
        .collect()
}
