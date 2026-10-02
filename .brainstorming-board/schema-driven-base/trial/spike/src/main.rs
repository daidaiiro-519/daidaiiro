//! 論点1の7つの契約とドリフト検知を、1つのユースケースと1つの集約で1回通す（引数なし）。
//! 論点2の案（注釈 ・ 集合の検証 ・ 報告の Schema）を確かめる（引数 q2）。
mod base;
mod usecase;
mod usecase_q2;
use serde_json::{json, Value};
use std::{fs, path::Path};

fn step(n: u32, title: &str) {
    println!("\n━━ {n}. {title}");
}
fn show(v: &Value) {
    println!("{}", serde_json::to_string_pretty(v).unwrap());
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("q2") => q2(),
        _ => q1(),
    }
}

fn q1() {
    let root = Path::new("work");
    let _ = fs::remove_dir_all(root);
    let uc_s = base::load(Path::new("schemas/use-case.schema.json"));
    let agg_s = base::load(Path::new("schemas/aggregate.schema.json"));

    step(1, "作成 ── スキーマから実体を作る");
    let uc = base::create(root, &uc_s, &[("id", json!("UC-1"))]);
    let agg = base::create(root, &agg_s, &[("id", json!("AGG-1"))]);
    println!("作成した: {} と {}", uc.display(), agg.display());
    show(&base::load(&uc));
    println!("検証: {:?}", base::validate(&uc_s, &base::load(&uc)));

    step(2, "案内 ── 未記入の項目に、何を書くかを示す（x-prompt.write）");
    for (at, p) in base::guide(&uc_s, &base::load(&uc), "write") {
        println!("{at}\t{p}");
    }

    step(3, "更新 ── JSON Patch で書き込む（検証を通過したので書き込む）");
    let fill = json!([
        {"op": "replace", "path": "/name", "value": "注文を確定する"},
        {"op": "replace", "path": "/goal", "value": "注文が確定済になり、在庫が引き当てられている"},
        {"op": "add", "path": "/writes/-", "value": "AGG-1"},
        {"op": "add", "path": "/scenarios/-", "value": {"id": "S-1", "name": "在庫があり、注文が確定する"}},
        {"op": "add", "path": "/scenarios/-", "value": {"id": "S-2", "name": "在庫が足りず、確定を断る"}}]);
    println!("適用する Patch:");
    show(&fill);
    println!("結果: {:?}", base::update(&uc, &uc_s, fill));
    let r = base::update(&agg, &agg_s, json!([
        {"op": "replace", "path": "/name", "value": "注文"},
        {"op": "add", "path": "/conditions/-", "value": {"id": "INV-1", "name": "明細の合計は、引当済の数量を超えない"}},
        {"op": "add", "path": "/conditions/-", "value": {"id": "PRE-1", "name": "確定する前に、明細が1件以上ある"}}]));
    println!("集約の結果: {r:?}");

    step(4, "更新 ── 検証に通らない Patch は書き込まない");
    let bad = json!([{"op": "replace", "path": "/scenarios", "value": []}]);
    show(&bad);
    println!("結果: {:?}", base::update(&uc, &uc_s, bad));
    println!("ファイルのシナリオ数（変わっていない）: {}", base::get(&base::load(&uc), "length(scenarios)"));

    step(5, "取得 ── JMESPath の式で取り出す");
    let d = base::load(&uc);
    for expr in ["scenarios[].id", "{name: name, scenarios: length(scenarios), writes: writes}", "scenarios[?id == 'S-2'].name | [0]"] {
        println!("式  {expr}\n値  {}", base::get(&d, expr));
    }

    step(6, "描画 ── title と x-view に従って Markdown へ");
    print!("{}", base::render(&uc_s, &d));

    step(7, "案内 ── 読む側への案内（x-prompt.read）");
    for (at, p) in base::guide(&uc_s, &d, "read") {
        println!("{at}\t{p}");
    }

    step(8, "ドリフト検知（concrete だけの契約）── テスト仕様とテスト実装を突き合わせる");
    let report = fs::read_to_string("test-report.txt").unwrap();
    println!("テスト実装の報告（test-report.txt）:\n{report}");
    let decls = vec![base::load(&uc), base::load(&agg)];
    let spec = usecase::spec_items(&decls);
    println!("テスト仕様: {spec:?}");
    let dr = usecase::drift(&spec, &report);
    println!("仕様に在ってテストに無い: {:?}\nテストに在って仕様に無い: {:?}\nテストが通っていない:     {:?}", dr.missing, dr.extra, dr.failing);

    step(9, "削除（項目）── シナリオ S-2 を JSON Patch の remove で消す");
    println!("結果: {:?}", base::update(&uc, &uc_s, json!([{"op": "remove", "path": "/scenarios/1"}])));
    let spec = usecase::spec_items(&[base::load(&uc), base::load(&agg)]);
    let dr = usecase::drift(&spec, &report);
    println!("仕様に在ってテストに無い: {:?}\nテストに在って仕様に無い: {:?}\nテストが通っていない:     {:?}", dr.missing, dr.extra, dr.failing);

    step(10, "削除（実体）── 集約 AGG-1 を消し、残った参照を探す");
    base::delete(&agg);
    println!("AGG-1 を指したまま残っている宣言: {:?}", usecase::dangling(&[base::load(&uc)], "AGG-1"));
}

fn q2() {
    let root = Path::new("work-q2");
    let _ = fs::remove_dir_all(root);
    let dir = root.join("decls");
    let uc_s = base::load(Path::new("schemas/use-case.q2.schema.json"));
    let agg_s = base::load(Path::new("schemas/aggregate.q2.schema.json"));

    step(1, "作成と記入 ── $schema で種類を指し、注釈の付いた Schema から作る");
    let agg = base::create(root, &agg_s, &[("$schema", json!("../../schemas/aggregate.q2.schema.json")), ("id", json!("AGG-1"))]);
    base::update(&agg, &agg_s, json!([
        {"op": "replace", "path": "/name", "value": "注文"},
        {"op": "add", "path": "/conditions/-", "value": {"id": "INV-1", "name": "明細の合計は、引当済の数量を超えない"}},
        {"op": "add", "path": "/conditions/-", "value": {"id": "PRE-1", "name": "確定する前に、明細が1件以上ある"}}])).unwrap();
    let uc = base::create(root, &uc_s, &[("$schema", json!("../../schemas/use-case.q2.schema.json")), ("id", json!("UC-1"))]);
    println!("結果: {:?}", usecase_q2::update(&dir, &uc, json!([
        {"op": "replace", "path": "/name", "value": "注文を確定する"},
        {"op": "replace", "path": "/goal", "value": "注文が確定済になり、在庫が引き当てられている"},
        {"op": "add", "path": "/writes/-", "value": "AGG-1"},
        {"op": "add", "path": "/scenarios/-", "value": {"id": "S-1", "name": "在庫があり、注文が確定する"}},
        {"op": "add", "path": "/scenarios/-", "value": {"id": "S-2", "name": "在庫が足りず、確定を断る"}}])));
    show(&base::load(&uc));

    step(2, "注釈から読む ── x-test-spec と x-ref");
    let all = usecase_q2::load_all(&dir);
    println!("テスト仕様（x-test-spec）: {:?}", usecase_q2::spec_items(&all));
    for d in &all {
        println!("{} の参照（x-ref）: {:?}", d.kind, usecase_q2::refs(d));
    }
    println!("集合の検証: {:?}", usecase_q2::validate_set(&all));

    step(3, "更新 ── 項目の ID が重複する Patch は書き込まない");
    println!("結果: {:?}", usecase_q2::update(&dir, &uc, json!([{"op": "add", "path": "/scenarios/-", "value": {"id": "S-1", "name": "同じ ID の別のシナリオ"}}])));

    step(4, "更新 ── 無い宣言を指す Patch は書き込まない");
    println!("結果: {:?}", usecase_q2::update(&dir, &uc, json!([{"op": "add", "path": "/writes/-", "value": "AGG-9"}])));

    step(5, "ドリフト検知 ── 報告の形が誤っていれば突き合わせない");
    let rs = base::load(Path::new("schemas/test-report.schema.json"));
    let bad = base::load(Path::new("test-report.bad.json"));
    println!("報告: {bad}");
    println!("結果: {:?}", usecase_q2::drift(&all, &bad, &rs).err());

    step(6, "ドリフト検知 ── 報告（test-report.json）とテスト仕様を突き合わせる");
    let rep = base::load(Path::new("test-report.json"));
    show(&rep);
    let dr = usecase_q2::drift(&usecase_q2::load_all(&dir), &rep, &rs).unwrap();
    println!("仕様に在ってテストに無い: {:?}\nテストに在って仕様に無い: {:?}\n通っていない:             {:?}", dr.missing, dr.extra, dr.not_passing);

    step(7, "削除（実体）── 参照されている宣言は削除しない");
    println!("結果: {:?}", usecase_q2::delete(&dir, &agg));
    println!("AGG-1 のファイルは残っている: {}", agg.exists());
}
