//! 宣言 ・ 実体 ・ テストが、常に一致していることを確かめる。
//!
//! **許されない状態は3つ** ── 仕様にあって実装にない／実装にあるのに仕様にない／
//! 仕様どおりの実装でない。どれも、ここで落ちる。

#[test]
fn 宣言に無い操作は関門に登録されていない_そして逆も無い() {
    let decls = usecase::decls();
    let reg = usecase::wire();
    let holes = base::operation_holes(&decls, &reg);
    assert!(
        holes.is_empty(),
        "仕様にあって実装にない: {:?} ／ 実装にあるのに仕様にない: {:?}",
        holes.declared_only,
        holes.registered_only
    );
}

#[test]
fn 宣言された検査の名前には実体が在る() {
    let decls = usecase::decls();
    let reg = usecase::wire();
    let missing = base::missing_checks(&decls, &reg);
    assert!(missing.is_empty(), "実体の無い検査: {:?}", missing);
}

#[test]
fn 射影を持たない型は無い() {
    let decls = usecase::decls();
    let reg = usecase::wire();
    let no = base::without_render(&decls, &reg);
    assert!(no.is_empty(), "射影を持たない型: {:?} ── 承認を通れない", no);
}

#[test]
fn 書き込みは宣言した先に収まる() {
    let decls = usecase::decls();
    let agg = decls.iter().find(|d| d.name == "出荷指示").unwrap();
    let op = agg.ops.iter().find(|o| o.name == "確定する").unwrap();

    let mut tx = base::Tx::new();
    usecase::confirm(&mut tx);
    let (extra, unused) = tx.diff(&op.writes);
    assert!(extra.is_empty(), "宣言に無い書き込み: {:?}", extra);
    assert!(unused.is_empty(), "宣言したのに書いていない: {:?}", unused);
}

#[test]
fn 密輸は書き込み口の関門で捕まる() {
    let decls = usecase::decls();
    let agg = decls.iter().find(|d| d.name == "出荷指示").unwrap();
    let op = agg.ops.iter().find(|o| o.name == "確定する").unwrap();

    let mut tx = base::Tx::new();
    usecase::confirm_smuggling(&mut tx); // 宣言していない「在庫」へ書く
    let (extra, _) = tx.diff(&op.writes);
    assert_eq!(
        extra.into_iter().collect::<Vec<_>>(),
        vec!["在庫".to_string()],
        "呼び出し口の関門では止まらないものが、書き込み口で出る"
    );
}

#[test]
fn 未知の種類は静かに無視されず誤りになる() {
    let bad = r#"[{"kind":"aggregate","id":"X","name":"甲","render":"r","zzz":1}]"#;
    assert!(base::read(bad).is_err(), "未知の欄が、静かに落ちてはならない");
}

#[test]
fn 対応表で三つの引き算が全部ゼロになる() {
    let decls = usecase::decls();
    let d = base::drift(&base::node_ids(&decls), &usecase::bindings(), &usecase::test_list());
    assert!(
        d.is_empty(),
        "対応の無い節: {:?} ／ 結ばれていない識別子: {:?} ／ 切れた対応: {:?}",
        d.unbound_nodes, d.unbound_externals, d.dangling
    );
}

#[test]
fn 仕様にあって実装にないシナリオが出る() {
    let decls = usecase::decls();
    let mut b = base::Bindings::new();
    b.bind("SC-01J7Q4M", "usecase::tests::明細が0件のとき確定できない");
    let d = base::drift(&base::node_ids(&decls), &b, &usecase::test_list());
    // 業務サービスの節も結んでいないので、2件とも出る
    assert_eq!(d.unbound_nodes.into_iter().collect::<Vec<_>>(),
               vec!["SC-01J7Q4N".to_string(), "SC-01J8E5M".to_string()]);
}

#[test]
fn 仕様に無いテストが出る() {
    let decls = usecase::decls();
    let mut list = usecase::test_list();
    list.insert("usecase::tests::誰も宣言していない".into());
    let d = base::drift(&base::node_ids(&decls), &usecase::bindings(), &list);
    assert_eq!(
        d.unbound_externals.into_iter().collect::<Vec<_>>(),
        vec!["usecase::tests::誰も宣言していない".to_string()]
    );
}

#[test]
fn テストの名前を変えると切れた対応が出る() {
    let decls = usecase::decls();
    let mut list = usecase::test_list();
    list.remove("usecase::tests::明細が1件あれば確定できる");
    list.insert("usecase::tests::明細が1件あれば確定できること".into());
    let d = base::drift(&base::node_ids(&decls), &usecase::bindings(), &list);
    assert_eq!(
        d.dangling.into_iter().collect::<Vec<_>>(),
        vec!["usecase::tests::明細が1件あれば確定できる".to_string()],
        "静かには切れない ── 引き算で出る"
    );
}

#[test]
fn 人が読む形はHTMLである() {
    let decls = usecase::decls();
    let reg = usecase::wire();
    for d in &decls {
        let s = base::render(d, &reg).expect("射影が無い");
        assert!(s.starts_with("<h1>"), "HTML で出ていない: {}", d.id);
    }
}

#[test]
fn 外の実行系が読む形はMDとして導出される() {
    let decls = usecase::decls();
    let reg = usecase::wire();
    let agg = decls.iter().find(|d| d.name == "出荷指示").unwrap();
    let md = base::render_as(agg, &reg, "render.skill.md").expect("MD の射影が無い");
    assert!(md.starts_with("---\n"), "frontmatter が無い");
    assert!(md.contains("name: AGG-01J7Q4K"), "宣言の ID が入っていない");
    assert!(md.contains("- 明細の合計は、引当済の数量を超えない"), "規則が導出されていない");
}

#[test]
fn 同じ宣言から二本の射影が出る() {
    let decls = usecase::decls();
    let reg = usecase::wire();
    let agg = decls.iter().find(|d| d.name == "出荷指示").unwrap();
    let html = base::render(agg, &reg).expect("HTML の射影が無い");
    let md = base::render_as(agg, &reg, "render.skill.md").expect("MD の射影が無い");
    assert!(html.starts_with("<h1>") && md.starts_with("---\n"));
    assert_ne!(html, md, "用途が違えば、形も違う");
}

#[test]
fn 変換機はJSONも出せる() {
    let decls = usecase::decls();
    let reg = usecase::wire();
    let d = decls.iter().find(|x| x.name == "出荷指示").unwrap();
    let kiro = base::render_as(d, &reg, "render.kiro.agent.json").expect("射影が無い");
    let cc = base::render_as(d, &reg, "render.claude.settings.json").expect("射影が無い");
    assert!(kiro.starts_with("{\n  \"name\""), "Kiro の形で出ていない: {kiro}");
    assert!(cc.contains("\"PreToolUse\""), "settings.json の形で出ていない: {cc}");
    assert!(serde_json::from_str::<serde_json::Value>(&kiro).is_ok(), "JSON として読めない");
    assert!(serde_json::from_str::<serde_json::Value>(&cc).is_ok(), "JSON として読めない");
}

#[test]
fn 同じ宣言から形式の違う射影が5本出る() {
    let decls = usecase::decls();
    let reg = usecase::wire();
    let d = decls.iter().find(|x| x.name == "出荷指示").unwrap();
    let mut out = std::collections::BTreeSet::new();
    for name in ["render.aggregate", "render.skill.md", "render.kiro.agent.json",
                 "render.claude.settings.json", "render.codex.agent.toml"] {
        out.insert(base::render_as(d, &reg, name).expect("射影が無い"));
    }
    assert_eq!(out.len(), 5, "同じ文字列が出ている ── 射影が分かれていない");
}

#[test]
fn 置いてある成果物と導出した成果物を引き算できる() {
    let d = base::drift(
        &usecase::emitted_files(),
        &{
            let mut b = base::Bindings::new();
            for f in usecase::emitted_files() {
                b.bind(&f, &f);
            }
            b
        },
        &usecase::placed_files(),
    );
    assert!(d.is_empty(), "{:?}", d);
}

#[test]
fn 宣言していない成果物が置かれたら出る() {
    let mut placed = usecase::placed_files();
    placed.insert(".claude/skills/誰も宣言していない/SKILL.md".into());
    let mut b = base::Bindings::new();
    for f in usecase::emitted_files() {
        b.bind(&f, &f);
    }
    let d = base::drift(&usecase::emitted_files(), &b, &placed);
    assert_eq!(
        d.unbound_externals.into_iter().collect::<Vec<_>>(),
        vec![".claude/skills/誰も宣言していない/SKILL.md".to_string()],
        "宣言に無い成果物が、引き算で出る"
    );
}

#[test]
fn 変換機はTOMLも出せる() {
    let decls = usecase::decls();
    let reg = usecase::wire();
    let d = decls.iter().find(|x| x.name == "出荷指示").unwrap();
    let toml = base::render_as(d, &reg, "render.codex.agent.toml").expect("射影が無い");
    // 鍵名は原文で照合したものだけを使う。
    for key in ["name = ", "description = ", "sandbox_mode = ", "developer_instructions = "] {
        assert!(toml.contains(key), "{key} が出ていない: {toml}");
    }
    assert!(toml.contains("\"\"\""), "複数行の文字列になっていない");
}

#[test]
fn 出る成果物の数は宣言の種類ごとの出す先で決まる() {
    let decls = usecase::decls();
    let want: usize = decls
        .iter()
        .map(|d| usecase::targets().iter().filter(|t| t.kinds.contains(&d.kind.as_str())).count())
        .sum();
    assert_eq!(usecase::emitted_files().len(), want, "種類ごとの出す先と合わない");
}

#[test]
fn 出す先の全部が実体を持つ() {
    let decls = usecase::decls();
    let reg = usecase::wire();
    for d in &decls {
        for t in usecase::targets().into_iter().filter(|t| t.kinds.contains(&d.kind.as_str())) {
            assert!(
                base::render_as(d, &reg, t.render).is_some(),
                "{} の実体が無い ── 置き場所だけ決まっている",
                t.render
            );
        }
    }
}

#[test]
fn 文脈の宣言は節を持たないので振る舞いの引き算に出ない() {
    let decls = usecase::decls();
    let bc = decls.iter().find(|d| d.kind == "bounded-context").expect("文脈の宣言が無い");
    assert!(bc.nodes.is_empty(), "語を節にしてはならない ── 節はテストと結ぶ単位である");
    // 節が無いので、対応表を増やさなくても引き算はゼロのままである。
    let d = base::drift(&base::node_ids(&decls), &usecase::bindings(), &usecase::test_list());
    assert!(d.is_empty(), "{:?}", d);
}

#[test]
fn 語の一覧は文脈の射影にだけ出る() {
    let decls = usecase::decls();
    let reg = usecase::wire();
    let bc = decls.iter().find(|d| d.kind == "bounded-context").unwrap();
    let html = base::render(bc, &reg).expect("射影が無い");
    assert!(html.contains("<b>引当</b> ── 在庫を、その注文のために確保すること"), "{html}");
    assert!(html.contains("担当は 受注チーム である"), "1つの文脈に1チームが出ていない");
    for other in decls.iter().filter(|d| d.kind != "bounded-context") {
        let s = base::render(other, &reg).expect("射影が無い");
        assert!(!s.contains("引当 ──"), "語の一覧が、文脈の外へ漏れている: {}", other.id);
    }
}

#[test]
fn 業務領域の参照は_ID_なので引き算できる() {
    let mut b = base::Bindings::new();
    for id in usecase::subdomain_ids() {
        b.bind(&id, &id);
    }
    let d = base::drift(&usecase::subdomain_ids(), &b, &usecase::subdomain_refs());
    assert!(
        d.unbound_externals.is_empty(),
        "宣言していない業務領域を指している: {:?}",
        d.unbound_externals
    );
}

#[test]
fn 存在しない業務領域を指すと出る() {
    let mut refs = usecase::subdomain_refs();
    refs.insert("SD-存在しない".into());
    let mut b = base::Bindings::new();
    for id in usecase::subdomain_ids() {
        b.bind(&id, &id);
    }
    let d = base::drift(&usecase::subdomain_ids(), &b, &refs);
    assert_eq!(
        d.unbound_externals.into_iter().collect::<Vec<_>>(),
        vec!["SD-存在しない".to_string()]
    );
}

#[test]
fn どこからも指されていない業務領域が出る() {
    let mut b = base::Bindings::new();
    for id in usecase::subdomain_ids() {
        b.bind(&id, &id);
    }
    let d = base::drift(&usecase::subdomain_ids(), &b, &usecase::subdomain_refs());
    assert_eq!(
        d.dangling.into_iter().collect::<Vec<_>>(),
        vec!["SD-01J8C2B".to_string()],
        "配送先住所の検証は、まだどの宣言からも指されていない"
    );
}

#[test]
fn 分類は射影に出るが節にはならない() {
    let decls = usecase::decls();
    let reg = usecase::wire();
    let sd = decls.iter().find(|d| d.id == "SD-01J8C1A").unwrap();
    assert!(sd.nodes.is_empty(), "分類を節にしてはならない");
    let html = base::render(sd, &reg).expect("射影が無い");
    assert!(html.contains("分類は <b>中核</b> である（2026-09-21 時点）"), "{html}");
}

#[test]
fn 宣言どうしの関係が全部つながっている() {
    for r in usecase::refs() {
        let d = usecase::relation_drift(&r);
        assert!(
            d.unbound_externals.is_empty(),
            "{}.{} が、宣言していない {} を指している: {:?}",
            r.from, r.field, r.to, d.unbound_externals
        );
    }
}

#[test]
fn 存在しない先を指すと関係の引き算で出る() {
    // 「ユースケース → 文脈」の関係で、指し先を1つ消したのと同じ状態を作る。
    let r = usecase::Ref { from: "use-case", field: "context", to: "subdomain" };
    let d = usecase::relation_drift(&r);
    assert!(
        !d.unbound_externals.is_empty(),
        "種類の取り違えが出ない ── 文脈の ID を業務領域として探しても出ないなら、検査になっていない"
    );
}

#[test]
fn ユースケースの集合が業務領域であることを確かめられる() {
    // 配送先住所の検証（補完）には、まだユースケースが1件も無い。
    assert_eq!(
        usecase::empty_subdomains().into_iter().collect::<Vec<_>>(),
        vec!["SD-01J8C2B".to_string()],
        "空の業務領域が挙がらない"
    );
}

#[test]
fn 事業領域は当てはめを持つが定義ではない() {
    let decls = usecase::decls();
    let reg = usecase::wire();
    let bd = decls.iter().find(|d| d.kind == "business-domain").unwrap();
    let html = base::render(bd, &reg).expect("射影が無い");
    let i = html.find("顧客に提供するサービスの大枠").expect("大枠が先に出ていない");
    let j = html.find("この開発での当てはめ").expect("当てはめが出ていない");
    assert!(i < j, "当てはめが先に出ている ── 抽象の高さが下がる");
    assert!(html.contains("当てはめであって、事業領域の定義ではない"));
}

// ── 多重度（原典に照らして固定した4本）────────────────────────

const 二文脈: &str = r#"[
 {"kind":"bounded-context","id":"BC-1","name":"受注の文脈","render":"r","body":{}},
 {"kind":"bounded-context","id":"BC-2","name":"倉庫の文脈","render":"r","body":{}},
 {"kind":"subdomain","id":"SD-1","name":"配送手配","render":"r","body":{}},
 {"kind":"use-case","id":"UC-1","name":"注文を確定する","render":"r",
  "body":{"subdomain":"SD-1","context":"BC-1"}},
 {"kind":"use-case","id":"UC-2","name":"注文を確定する","render":"r",
  "body":{"subdomain":"SD-1","context":"BC-2"}}
]"#;

#[test]
fn 参照を2件書いたら形の誤りとして出る() {
    let bad = r#"[{"kind":"use-case","id":"UC-9","name":"甲","render":"r",
                   "body":{"subdomain":["SD-1","SD-2"]}}]"#;
    let ds = base::read(bad).expect("読めない");
    assert_eq!(
        usecase::ref_shape_errors_in(&ds).into_iter().collect::<Vec<_>>(),
        vec!["UC-9.subdomain は1件だけである".to_string()]
    );
    assert!(usecase::ref_shape_errors().is_empty(), "いまの仕様に形の誤りは無い");
}

#[test]
fn 文脈をまたいだ同名は誤りではない() {
    let ds = base::read(二文脈).expect("読めない");
    assert!(
        usecase::name_clashes_in(&ds).is_empty(),
        "別の文脈に同じ名前が在るのは正常である ── どちらの文脈にいるかだけが両者を区別する"
    );
}

#[test]
fn 同じ文脈の中の同名は出る() {
    let same = 二文脈.replace(r#""context":"BC-2""#, r#""context":"BC-1""#);
    let ds = base::read(&same).expect("読めない");
    assert_eq!(
        usecase::name_clashes_in(&ds).into_iter().collect::<Vec<_>>(),
        vec!["BC-1 の中に「注文を確定する」が2件以上ある".to_string()]
    );
}

#[test]
fn 業務領域が文脈をまたいでいたら挙がる() {
    let ds = base::read(二文脈).expect("読めない");
    assert_eq!(
        usecase::subdomains_spanning_contexts_in(&ds).into_iter().collect::<Vec<_>>(),
        vec!["SD-1".to_string()],
        "同じ業務領域のユースケースが2つの文脈へ分散している"
    );
    assert!(
        usecase::subdomains_spanning_contexts().is_empty(),
        "いまの仕様では、またいでいない"
    );
}

// ── 置き場所の決まり ──────────────────────────────────────────

#[test]
fn 事業領域の宣言は根にちょうど1件ある() {
    let found = base::root_findings(&usecase::spec_files(), usecase::ROOT_KIND);
    assert!(found.is_empty(), "{:?}", found.iter().map(|f| f.line()).collect::<Vec<_>>());
}

#[test]
fn 根の外に置くと出る() {
    let files = vec![base::SpecFile {
        path: "subdomains/all.json".into(),
        decls: base::read(r#"[{"kind":"business-domain","id":"BD-1","name":"甲","render":"r"}]"#)
            .unwrap(),
    }];
    let lines: Vec<String> = base::root_findings(&files, usecase::ROOT_KIND)
        .iter()
        .map(|f| f.line())
        .collect();
    assert!(lines.iter().any(|l| l.contains("根の外に在る")), "{lines:?}");
}

#[test]
fn 根に2件あると出る() {
    let files = vec![base::SpecFile {
        path: "business-domain.json".into(),
        decls: base::read(
            r#"[{"kind":"business-domain","id":"BD-1","name":"甲","render":"r"},
                {"kind":"business-domain","id":"BD-2","name":"乙","render":"r"}]"#,
        )
        .unwrap(),
    }];
    let lines: Vec<String> = base::root_findings(&files, usecase::ROOT_KIND)
        .iter()
        .map(|f| f.line())
        .collect();
    assert!(lines.iter().any(|l| l.contains("根に2件以上ある")), "{lines:?}");
}

#[test]
fn 根に無いと出る() {
    let files = vec![base::SpecFile { path: "usecases/all.json".into(), decls: vec![] }];
    let lines: Vec<String> = base::root_findings(&files, usecase::ROOT_KIND)
        .iter()
        .map(|f| f.line())
        .collect();
    assert!(lines.iter().any(|l| l.contains("全体の枠が決まっていない")), "{lines:?}");
}

#[test]
fn 操作を持つのに節が無い宣言が挙がる() {
    // いまの仕様では、ユースケースが操作1件に対して節を持っていない。
    assert_eq!(
        usecase::ops_without_nodes().into_iter().collect::<Vec<_>>(),
        vec!["UC-01J7Q8Q".to_string()],
        "振る舞いを宣言していない宣言が挙がらない"
    );
}

#[test]
fn 振る舞いを持つ宣言は三種類だけである() {
    let decls = usecase::decls();
    // **業務サービスも振る舞いを持つ** ── 状態は持たないが、業務ロジックを持つ。
    let 振る舞いを持てる: std::collections::BTreeSet<&str> =
        ["aggregate", "use-case", "domain-service"].into_iter().collect();
    for d in &decls {
        if !d.ops.is_empty() || !d.nodes.is_empty() {
            assert!(
                振る舞いを持てる.contains(d.kind.as_str()),
                        "{} が振る舞いを持っている ── 事業領域 ・ 業務領域 ・ 文脈は持たない",
                d.kind
            );
        }
    }
}

#[test]
fn 業務サービスは状態を書き換えない() {
    let decls = usecase::decls();
    for d in decls.iter().filter(|d| d.kind == "domain-service") {
        for o in &d.ops {
            assert!(o.writes.is_empty(),
                    "{} が書き換えている ── 業務サービスは自分自身の状態を持たない", d.id);
        }
    }
}
