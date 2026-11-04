//! semantic-agent-schema コマンドの入口。引数を読み、adapter を組み立ててユースケースを呼び、結果を JSON の1行で返す。
//! 意味の部分（--semantic）は、結果の代わりに手順（procedure.schema.json）を返す。Semantic Agent Schema はエージェントを起動しない。
//! semantic-agent-schema log（UC7）は記録を読むだけで、何も足さない（記録が無ければ作らない）。semantic-agent-schema search（UC6）は記録しない。
//!
//! 終了コード：0 = violated と error が無い（作成 ・ 更新は書けた）／ 1 = どちらかがある、または始められなかった ・
//! 書かなかった ／ 2 = 使い方の誤り。
//!
//! 入口はコンポジションルートで、置き場所を決めてアダプターを組み立てる：経路の根は Git の根、操作の記録は
//! `<git-common-dir>/semantic-agent-schema/records.sqlite`（ワークツリーで1つ。決めるのは db_path の1か所）。記録はコミットしない。
//! 無ければ空で作り、表の形の版が違えば開かずに、消すと作り直せると返す（作り直せるデータ）。Git のリポジトリの外では断る。

mod caller;
mod output;

use caller::FromArguments;
use outbound_filesystem::{OutDir, WorkingTree};
use outbound_git::GitCommits;
use outbound_javascript::Boa;
use outbound_jsonschema::Validator;
use outbound_sqlite::SqliteLog;
use outbound_templates::Mustache;
use output::{Refused, log_refused};
use recording_application::ports::OperationFilter;
use recording_application::use_cases::history::{
    Unidentified, identify, read_operation, read_operations, register, register_session,
};
use rendering_application::use_cases::design::{DESIGN_SYSTEM, check_design_systems};
use rendering_application::use_cases::render::{RenderRequest, render};
use search_application::use_cases::search::{
    Range, search_procedure, search_where, search_with_answer,
};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use validation_application::use_cases::validate::{
    Scope, Selection, semantic_procedures, validate, validate_with_answer,
};
use writing_application::use_cases::write::{
    AdoptAs, AdoptRequest, CreateRequest, DeleteRequest, SemanticRequest, UpdateRequest, adopt,
    create, delete, semantic_procedure, update,
};
use writing_domain::values::{Caller, SchemaPlaces};

const USAGE: &str = "使い方：semantic-agent-schema init ／ \
semantic-agent-schema validate <経路>... --schemas <置き場所>... (--session <ID> | --human) ／ \
semantic-agent-schema validate <経路>... --schemas <置き場所>... [--all-rules] --semantic (--session <ID> | --human) ／ \
semantic-agent-schema validate <経路> --schemas <置き場所>... [--all-rules] --answer <判定の JSON のファイル | -> (--session <ID> | --human) ／ \
semantic-agent-schema validate <デザインシステム>... --schemas <置き場所>... ／ \
semantic-agent-schema create <経路> --schemas <置き場所>... --concrete <$id> --value <JSON> [--reason <理由>] (--session <ID> | --human) ／ \
semantic-agent-schema create <経路> --schemas <置き場所>... --concrete <$id> --semantic [--reason <理由>] (--session <ID> | --human) ／ \
semantic-agent-schema create <経路> --schemas <置き場所>... --adopt [--concrete <$id>] [--reason <理由>] (--session <ID> | --human) ／ \
semantic-agent-schema update <経路> --schemas <置き場所>... --patch <JSON Patch> --if-match <ハッシュ> [--range <経路>...] [--reason <理由>] (--session <ID> | --human) ／ \
semantic-agent-schema update <経路> --schemas <置き場所>... --semantic [--range <経路>...] [--reason <理由>] (--session <ID> | --human) ／ \
semantic-agent-schema update <経路> --schemas <置き場所>... --adopt --reason <理由> [--base <コミット>] [--range <経路>...] (--session <ID> | --human) ／ \
semantic-agent-schema delete <経路> --reason <理由> --if-match <ハッシュ> [--range <経路>...] (--session <ID> | --human) ／ \
semantic-agent-schema render <経路>... --schemas <置き場所>... --design-system <経路> --out <書き出す先> ／ \
semantic-agent-schema search <経路>... --schemas <置き場所>... --where <JSON Pointer>=<JSON> [--where …]... ／ \
semantic-agent-schema search <経路>... --schemas <置き場所>... --semantic --question <問い> ／ \
semantic-agent-schema search <経路>... --schemas <置き場所>... --answer <答えの JSON のファイル | -> ／ \
semantic-agent-schema log [<経路>...] [--by <セッション ID>] [--folder <フォルダ>] [--since <時刻>] [--limit <件数>] ／ \
semantic-agent-schema log --operation <ID> ／ \
経路はどこで打っても Git の根からの相対（根の外を指す経路は読まない）。--schemas はファイルか、直下の JSON のファイルを読むフォルダ（Concrete と $ref の先を $id で引く）。\
--if-match は読んだ時の中身のハッシュで、いまの中身と違えば書かない ・ 消さない。\
Concrete を直す ・ 消すときは --range <経路>... が要る（値の無い --range は、範囲が空と明示する）。\
AI のセッションは --session <ID>（Claude Code などツールの環境変数があれば省ける。Gemini CLI ・ Cursor は毎回付ける。init は新しい ID を返す）、人は --human（ツールの環境変数があるところでは受け取らない）。search と render とデザインシステムの validate は、誰が打ったかを取らず、記録しない。\
答えと --where の値は JSON だけ。--base は取り込む前の中身があるコミット（無ければ前の値の無い差分で取り込む）。\
操作の記録は <git-common-dir>/semantic-agent-schema/records.sqlite。表の形の版が違う記録は開かないので、その記録のファイルを消して打ち直すと空で作り直す。\
終了コード：0 = violated と error が無い ／ 1 = どちらかがある ・ 断った ／ 2 = 使い方の誤り";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--help" | "help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some("init") => run_init(&args[1..]),
        Some("validate") => run_validate(&args[1..]),
        Some("create") => run_change(&args[1..], Command::Create),
        Some("update") => run_change(&args[1..], Command::Update),
        Some("delete") => run_delete(&args[1..]),
        Some("render") => run_render(&args[1..]),
        Some("log") => run_log(&args[1..]),
        Some("search") => run_search(&args[1..]),
        Some(other) => {
            eprintln!("semantic-agent-schema: 知らないコマンドです: {other}。{USAGE}");
            ExitCode::from(2)
        }
        None => {
            eprintln!("semantic-agent-schema: コマンドを指定してください。{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// 値を並べて取る指定（--schemas ・ --range）を抜き出す。次の -- で始まる引数までが値で、何度渡してもよい。
/// 指定が無ければ None、残りの引数も返す。--range は値が無くてもよい（範囲が空と明示する）。
fn take_list(args: &[String], name: &str) -> Result<(Option<Vec<String>>, Vec<String>), String> {
    let mut values: Option<Vec<String>> = None;
    let mut rest = Vec::new();
    let mut it = args.iter().peekable();
    while let Some(a) = it.next() {
        if a != name {
            rest.push(a.clone());
            continue;
        }
        let list = values.get_or_insert_with(Vec::new);
        let before = list.len();
        while let Some(v) = it.next_if(|v| !v.starts_with("--")) {
            if !v.is_empty() {
                list.push(v.clone());
            }
        }
        if list.len() == before && name != "--range" {
            return Err(format!("{name} に経路がありません"));
        }
    }
    Ok((values, rest))
}

/// --schemas の置き場所を、読むファイルの経路の並びに開く。無ければ断る。
fn schema_files(places: Option<Vec<String>>) -> Result<SchemaPlaces, ExitCode> {
    let Some(places) = places else {
        return Err(usage(
            "--schemas <置き場所>... でスキーマの置き場所を渡してください（従う Concrete と $ref の先を、その中から $id で引く）",
        ));
    };
    WorkingTree::new(root())
        .schema_files(&places)
        .map(|files| SchemaPlaces::new(places.clone(), files))
        .map_err(|message| {
            refuse(Refused {
                code: "not_found",
                message,
                ..Refused::default()
            })
        })
}

fn usage(message: &str) -> ExitCode {
    output::print_refused(&Refused {
        code: "usage",
        message: format!("{message}。{USAGE}"),
        ..Refused::default()
    });
    ExitCode::from(2)
}

/// 操作の記録の置き場所。決めるのはここだけ：`<git-common-dir>/semantic-agent-schema/records.sqlite`（ワークツリーで1つ）。
/// Git のリポジトリの外なら、記録を作らずに断る。
fn db_path(root: &Path) -> Result<PathBuf, Refused> {
    let Some(common) = GitCommits::new(root.to_path_buf()).common_dir() else {
        return Err(Refused {
            code: "not_in_repository",
            message: "Git のリポジトリの外なので、記録を作らない（中身の版は Git が持つ前提）"
                .to_string(),
            hint: Some(Box::new(
                "Git のリポジトリの中で使う（リポジトリでなければ git init してから打つ）"
                    .to_string(),
            )),
            ..Refused::default()
        });
    };
    Ok(common.join("semantic-agent-schema").join("records.sqlite"))
}

/// 文書の経路の根：作業ツリーの根（Git の根）。文書 ・ --schemas ・ --range ・ --design-system ・ --out ・
/// --answer のファイルの経路は、どこで打っても、ここからの相対で読む。Git が読めなければ作業フォルダ。
fn root() -> PathBuf {
    GitCommits::new(PathBuf::from("."))
        .toplevel()
        .unwrap_or_else(|| PathBuf::from("."))
}

/// semantic-agent-schema を打ったフォルダ：作業ツリーの根（ワークツリーごとに違う。絶対の経路）。
/// 記録は Git の中だけで作るので、作業フォルダになるのは Git が読めないときだけ。
fn folder() -> String {
    GitCommits::new(PathBuf::from("."))
        .toplevel()
        .or_else(|| std::env::current_dir().ok())
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| ".".to_string())
}

/// 誰が打ったかを決められなかった理由を、出力の形にする。
fn unidentified(u: Unidentified) -> Refused {
    match u {
        Unidentified::Ambiguous(tools) => Refused {
            code: "usage",
            message: format!(
                "{} の中で打たれていて、どれが打ったかを決められません。--session <打った会話の ID> で名乗ってください",
                tools.join(" と ")
            ),
            ..Refused::default()
        },
        Unidentified::HumanInTool(tools) => Refused {
            code: "usage",
            message: format!(
                "{} の中（ツールの環境変数がある）で打たれたので、--human を受け取りません。AI のセッションの中では人を名乗れません。人の操作は、ツールの外の端末で打ってください",
                tools.join(" と ")
            ),
            ..Refused::default()
        },
        Unidentified::Missing => Refused {
            code: "usage",
            message: "誰が打ったかが決まりません。AI のセッションは --session <打った会話の ID> で名乗り、人は --human を渡してください（人とみなさない）".to_string(),
            ..Refused::default()
        },
        Unidentified::Log(message) => log_refused(message),
    }
}

/// 誰が打ったかの引数（--session <ID> か --human）。
#[derive(Debug, Clone, Default)]
struct Who {
    session: Option<String>,
    human: bool,
}

/// --session <ID> と --human を抜き出す。両方を渡したら断る。残りの引数も返す。
fn take_who(args: &[String]) -> Result<(Who, Vec<String>), String> {
    let mut who = Who::default();
    let mut rest = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--session" => match it.next() {
                Some(s) if !s.is_empty() && !s.starts_with("--") => who.session = Some(s.clone()),
                _ => return Err("--session に ID がありません".to_string()),
            },
            "--human" => who.human = true,
            _ => rest.push(a.clone()),
        }
    }
    if who.human && who.session.is_some() {
        return Err(
            "--human と --session は一緒に渡せません（人の操作は --human、AI のセッションは --session）"
                .to_string(),
        );
    }
    Ok((who, rest))
}

/// 呼び出し元：--human ・ --session ・ ツールごとの環境変数（記録のモジュールの手続きで決める）。どれも無ければ断る。
/// sessions にはまだ足さない（足すのは register）。
fn caller(who: Who) -> Result<Caller, Refused> {
    let folder = folder();
    let from = FromArguments::new(
        who.session,
        who.human,
        |name| std::env::var(name).ok(),
        &folder,
    );
    identify(&from).map_err(unidentified)
}

/// 置き場所 ・ バリデーター ・ 記録を組み立てる。
fn adapters() -> Result<(WorkingTree, Validator, SqliteLog), Refused> {
    let root = root();
    let validator = Validator::new().map_err(|message| Refused {
        code: "validator",
        message,
        ..Refused::default()
    })?;
    let log = SqliteLog::open(&db_path(&root)?)
        .map_err(|message| Refused {
            code: "log",
            message,
            ..Refused::default()
        })?
        .in_worktree(folder());
    Ok((WorkingTree::new(root), validator, log))
}

/// 記録を開かずに、Git の根の置き場所とバリデーターを組み立てる（検索 ・ デザインシステムの検証 ・ 描画）。
/// Git のリポジトリの外では、経路の根が決まらないので断る。
fn read_only() -> Result<(WorkingTree, Validator), Refused> {
    let Some(root) = GitCommits::new(PathBuf::from(".")).toplevel() else {
        return Err(Refused {
            code: "not_in_repository",
            message: "Git のリポジトリの外なので、経路の根（Git の根）が決まらない".to_string(),
            hint: Some(Box::new(
                "Git のリポジトリの中で使う（リポジトリでなければ git init してから打つ）"
                    .to_string(),
            )),
            ..Refused::default()
        });
    };
    let validator = Validator::new().map_err(|message| Refused {
        code: "validator",
        message,
        ..Refused::default()
    })?;
    Ok((WorkingTree::new(root), validator))
}

/// semantic-agent-schema init：新しいセッションの ID を決めて sessions に登録し、その行を返す。
/// あとで打つコマンドの既定にはしない（返った ID を --session で渡す）。
fn run_init(args: &[String]) -> ExitCode {
    if let Some(a) = args.first() {
        return usage(&format!(
            "init は引数を取りません（新しい ID を作って返す）: {a}"
        ));
    }
    let (_, _, mut log) = match adapters() {
        Ok(a) => a,
        Err(r) => return refuse(r),
    };
    match register_session(&folder(), &mut log) {
        Ok(row) => {
            output::print_session(&row);
            ExitCode::SUCCESS
        }
        Err(u) => refuse(unidentified(u)),
    }
}

fn run_validate(args: &[String]) -> ExitCode {
    let (schemas, args) = match take_list(args, "--schemas") {
        Ok(v) => v,
        Err(m) => return usage(&m),
    };
    let (who, args) = match take_who(&args) {
        Ok(v) => v,
        Err(m) => return usage(&m),
    };
    let mut all_rules = false;
    let mut semantic = false;
    let mut answer = None;
    let mut paths = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--all-rules" => all_rules = true,
            "--semantic" => semantic = true,
            "--answer" => match it.next() {
                Some(s) if !s.is_empty() => answer = Some(s.clone()),
                _ => return usage("--answer に判定の JSON のファイルか - がありません"),
            },
            s if s.starts_with("--") => return usage(&format!("知らない指定です: {s}")),
            p => paths.push(p.to_string()),
        }
    }
    if semantic && answer.is_some() {
        return usage(
            "--semantic と --answer は一緒に渡せません（--semantic で手順を受け取り、判定を --answer で渡す）",
        );
    }
    if all_rules && !semantic && answer.is_none() {
        return usage("--all-rules は --semantic か --answer と一緒に渡してください");
    }
    // 意味の判定は、既定では前の判定のあと変わった所だけ。--all-rules ならすべて
    let selection = Selection { all_rules };
    if paths.is_empty() {
        return usage("検証する対象の経路を渡してください（範囲は渡された経路だけ）");
    }
    let schemas = match schema_files(schemas) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let scope = Scope { paths, schemas };
    // デザインシステムは、受け取りの型とテンプレートの書き方の契約に照らす（描画のモジュール）
    {
        let paths = &scope.paths;
        let design: Vec<bool> = paths.iter().map(|p| is_design_system(p)).collect();
        if design.iter().any(|d| *d) {
            if !design.iter().all(|d| *d) || all_rules || semantic || answer.is_some() {
                return usage(
                    "デザインシステムは、ほかの対象や --semantic ・ --answer と一緒に渡さずに、それだけで検証してください",
                );
            }
            // 注入された依存を契約に照らして返すだけで、記録しない（誰が打ったかも取らない）
            let (store, validator) = match read_only() {
                Ok(a) => a,
                Err(r) => return refuse(r),
            };
            return match check_design_systems(
                paths,
                scope.schemas.files(),
                &store,
                &validator,
                &Mustache,
                &Boa,
            ) {
                Ok(v) => {
                    output::print_design_validation(&v);
                    if v.counts.fails() {
                        ExitCode::from(1)
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Err(r) => refuse(r),
            };
        }
    }
    let answer_text = match &answer {
        Some(from) => {
            if scope.paths.len() != 1 {
                return usage("--answer は経路を1つだけ渡してください");
            }
            match read_answer(from) {
                Ok(t) => Some(t),
                Err(r) => return refuse(r),
            }
        }
        None => None,
    };
    let (store, validator, mut log) = match adapters() {
        Ok(a) => a,
        Err(r) => return refuse(r),
    };
    let caller = match caller(who.clone()) {
        Ok(c) => c,
        Err(r) => return refuse(r),
    };
    if let Err(message) = register(&caller, &mut log) {
        return refuse(log_refused(message));
    }
    if semantic {
        // 判定の手順を対象ごとに出すだけで、記録しない。判定する所が0件の対象も、steps が空の手順を出す
        return match semantic_procedures(&scope, selection, &store, &validator, &caller, &log) {
            Ok(procedures) => {
                for p in &procedures {
                    output::print_procedure(p);
                }
                ExitCode::SUCCESS
            }
            Err(r) => refuse(r),
        };
    }
    let result = match answer_text {
        Some(text) => validate_with_answer(
            &scope, selection, &text, &store, &validator, &caller, &mut log,
        ),
        _ => validate(&scope, &store, &validator, &caller, &mut log),
    };
    match result {
        Ok(v) => {
            output::print_validation(&v);
            if v.counts.fails() {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(r) => refuse(r),
    }
}

/// 判定の JSON を、ファイルか標準入力（-）から読む。
fn read_answer(from: &str) -> Result<String, Refused> {
    let read = if from == "-" {
        let mut text = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut text)
            .map(|_| text)
            .map_err(|e| format!("標準入力を読めない：{e}"))
    } else {
        std::fs::read_to_string(root().join(from)).map_err(|e| format!("{from} を読めない：{e}"))
    };
    read.map_err(|message| Refused {
        code: "invalid_input",
        message,
        ..Refused::default()
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Command {
    Create,
    Update,
}

fn run_change(args: &[String], command: Command) -> ExitCode {
    let (schemas, args) = match take_list(args, "--schemas") {
        Ok(v) => v,
        Err(m) => return usage(&m),
    };
    let (range, args) = match take_list(&args, "--range") {
        Ok(v) => v,
        Err(m) => return usage(&m),
    };
    let (who, args) = match take_who(&args) {
        Ok(v) => v,
        Err(m) => return usage(&m),
    };
    if command == Command::Create && range.is_some() {
        return usage("--range は Concrete を直す ・ 消すときだけ渡してください");
    }
    let args = args.as_slice();
    // --adopt ・ --semantic は値を取らない旗なので、先に抜き出す
    let adopting = args.iter().any(|a| a == "--adopt");
    let semantic = args.iter().any(|a| a == "--semantic");
    let args: Vec<String> = args
        .iter()
        .filter(|a| *a != "--adopt" && *a != "--semantic")
        .cloned()
        .collect();
    let mut flags: Vec<(&str, Option<String>)> = match command {
        Command::Create => vec![("--concrete", None), ("--value", None)],
        Command::Update => vec![("--patch", None), ("--if-match", None), ("--base", None)],
    };
    flags.push(("--reason", None));
    let mut paths = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if let Some(slot) = flags.iter_mut().find(|(name, _)| *name == a.as_str()) {
            match it.next() {
                Some(v) if !v.is_empty() => slot.1 = Some(v.clone()),
                _ => return usage(&format!("{a} に値がありません")),
            }
        } else if a.starts_with("--") {
            return usage(&format!("知らない指定です: {a}"));
        } else {
            paths.push(a.clone());
        }
    }
    let [path] = paths.as_slice() else {
        return usage("経路を1つ渡してください");
    };
    let take = |name: &str| {
        flags
            .iter()
            .find(|(n, _)| *n == name)
            .and_then(|(_, v)| v.clone())
    };
    let reason = take("--reason");
    if let Some(base) = take("--base") {
        if !adopting {
            return usage("--base は --adopt で更新を取り込むときに渡してください");
        }
        if base.starts_with('-') {
            return usage(&format!("--base はコミットの名前で渡してください: {base}"));
        }
    }

    let schemas = match schema_files(schemas) {
        Ok(s) => s,
        Err(code) => return code,
    };
    if semantic {
        // 意味で書く：手順を返すだけ。値と JSON Patch は、手順を受け取ったセッションが決めて、手順の command で書く
        if adopting {
            return usage("--semantic と --adopt は一緒に渡せません");
        }
        if let Some(name) = ["--value", "--patch", "--if-match"]
            .into_iter()
            .find(|n| take(n).is_some())
        {
            return usage(&format!("--semantic と {name} は一緒に渡せません"));
        }
        let concrete = match command {
            Command::Create => match take("--concrete") {
                Some(c) => Some(c),
                None => return usage("--concrete を渡してください"),
            },
            Command::Update => None,
        };
        let request = SemanticRequest {
            path: path.clone(),
            concrete,
            schemas,
            range,
            reason,
        };
        let result = run_with(
            |store, validator, caller, _| semantic_procedure(&request, store, validator, caller),
            who.clone(),
        );
        return match result {
            Ok(p) => {
                output::print_procedure(&p);
                ExitCode::SUCCESS
            }
            Err(r) => refuse(r),
        };
    }
    if adopting {
        // 取り込みはファイルを書かないので、書く中身（--value ・ --patch）と前提のハッシュ（--if-match）を受けない
        if let Some(name) = ["--value", "--patch", "--if-match"]
            .into_iter()
            .find(|n| take(n).is_some())
        {
            return usage(&format!("--adopt と {name} は一緒に渡せません"));
        }
        if command == Command::Update && reason.is_none() {
            return usage(
                "--adopt で更新を取り込むときは、外で直した理由を --reason で渡してください",
            );
        }
        let request = AdoptRequest {
            path: path.clone(),
            as_kind: match command {
                Command::Create => AdoptAs::Create,
                Command::Update => AdoptAs::Update,
            },
            concrete: take("--concrete"),
            schemas,
            range,
            reason,
            base: take("--base"),
        };
        let committed = GitCommits::new(root());
        let result = run_with(
            |store, validator, caller, log| {
                adopt(&request, store, validator, &committed, caller, log)
            },
            who.clone(),
        );
        return match result {
            Ok(c) => {
                output::print_change(&c);
                ExitCode::SUCCESS
            }
            Err(r) => refuse(r),
        };
    }
    let json = |name: &str| -> Result<Value, ExitCode> {
        let text = take(name).ok_or_else(|| usage(&format!("{name} を渡してください")))?;
        serde_json::from_str(&text).map_err(|e| {
            refuse(Refused {
                code: "invalid_input",
                path: Some(path.as_str().into()),
                message: format!("{name} が JSON として読めない：{e}"),
                ..Refused::default()
            })
        })
    };
    let result = match command {
        Command::Create => {
            let Some(concrete) = take("--concrete") else {
                return usage("--concrete を渡してください");
            };
            let value = match json("--value") {
                Ok(v) => v,
                Err(code) => return code,
            };
            let request = CreateRequest {
                path: path.clone(),
                concrete,
                value,
                schemas,
                reason,
            };
            run_with(
                |store, validator, caller, log| create(&request, store, validator, caller, log),
                who.clone(),
            )
        }
        Command::Update => {
            let Some(if_match) = take("--if-match") else {
                return usage(
                    "--if-match に、読んだ時の中身のハッシュを渡してください（いまの中身と同じときだけ直す）",
                );
            };
            let patch = match json("--patch") {
                Ok(v) => v,
                Err(code) => return code,
            };
            let request = UpdateRequest {
                path: path.clone(),
                patch,
                if_match,
                schemas,
                range,
                reason,
            };
            run_with(
                |store, validator, caller, log| update(&request, store, validator, caller, log),
                who.clone(),
            )
        }
    };
    match result {
        Ok(c) => {
            output::print_change(&c);
            ExitCode::SUCCESS
        }
        Err(r) => refuse(r),
    }
}

fn run_delete(args: &[String]) -> ExitCode {
    let (range, args) = match take_list(args, "--range") {
        Ok(v) => v,
        Err(m) => return usage(&m),
    };
    let (who, args) = match take_who(&args) {
        Ok(v) => v,
        Err(m) => return usage(&m),
    };
    let mut flags: Vec<(&str, Option<String>)> = vec![("--reason", None), ("--if-match", None)];
    let mut paths = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if let Some(slot) = flags.iter_mut().find(|(name, _)| *name == a.as_str()) {
            match it.next() {
                Some(v) if !v.is_empty() => slot.1 = Some(v.clone()),
                _ => return usage(&format!("{a} に値がありません")),
            }
        } else if a.starts_with("--") {
            return usage(&format!("知らない指定です: {a}"));
        } else {
            paths.push(a.clone());
        }
    }
    let [path] = paths.as_slice() else {
        return usage("経路を1つ渡してください");
    };
    let take = |name: &str| {
        flags
            .iter()
            .find(|(n, _)| *n == name)
            .and_then(|(_, v)| v.clone())
    };
    // 消すと中身は記録の事前の状態にしか残らないので、なぜ消すかを必ず残す
    let Some(reason) = take("--reason") else {
        return usage("消すときは、なぜ消すかを --reason で渡してください");
    };
    let Some(if_match) = take("--if-match") else {
        return usage(
            "--if-match に、読んだ時の中身のハッシュを渡してください（いまの中身と同じときだけ消す）",
        );
    };
    let request = DeleteRequest {
        path: path.clone(),
        if_match,
        reason: Some(reason),
        range,
    };
    match run_with(
        |store, _, caller, log| delete(&request, store, caller, log),
        who,
    ) {
        Ok(d) => {
            output::print_deletion(&d);
            ExitCode::SUCCESS
        }
        Err(r) => refuse(r),
    }
}

fn run_render(args: &[String]) -> ExitCode {
    let (schemas, args) = match take_list(args, "--schemas") {
        Ok(v) => v,
        Err(m) => return usage(&m),
    };
    let mut design_system = None;
    let mut out = None;
    let mut targets = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let slot = match a.as_str() {
            "--design-system" => &mut design_system,
            "--out" => &mut out,
            s if s.starts_with("--") => return usage(&format!("知らない指定です: {s}")),
            p => {
                targets.push(p.to_string());
                continue;
            }
        };
        match it.next() {
            Some(v) if !v.is_empty() => *slot = Some(v.clone()),
            _ => return usage(&format!("{a} に値がありません")),
        }
    }
    if targets.is_empty() {
        return usage("描く対象の経路を1つ以上渡してください");
    }
    let Some(design_system) = design_system else {
        return usage(
            "--design-system でデザインシステムのファイルを渡してください（Semantic Agent Schema は既定を持たない）",
        );
    };
    let Some(out) = out else {
        return usage("--out で書き出す先を渡してください");
    };
    // 描くのは Git の根からの経路だけで、Git の外では断る（記録は開かない）
    let (store, validator) = match read_only() {
        Ok(a) => a,
        Err(r) => return refuse(r),
    };
    let schemas = match schema_files(schemas) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let request = RenderRequest {
        targets,
        design_system,
        schemas,
        out,
    };
    match render(
        &request,
        &store,
        &validator,
        &Mustache,
        &Boa,
        &OutDir::new(root()),
    ) {
        Ok(r) => {
            output::print_render(&r);
            ExitCode::SUCCESS
        }
        Err(r) => refuse(r),
    }
}

/// semantic-agent-schema log：操作の記録を読む（UC7）。読むだけで、セッションの行も足さない。記録が無ければ作らずに、無いものとして返す。
fn run_log(args: &[String]) -> ExitCode {
    let mut filter = OperationFilter::default();
    let mut operation = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let name = a.as_str();
        if !name.starts_with("--") {
            filter.paths.push(a.clone());
            continue;
        }
        let Some(v) = it.next().filter(|v| !v.is_empty()) else {
            return if ["--by", "--folder", "--since", "--limit", "--operation"].contains(&name) {
                usage(&format!("{a} に値がありません"))
            } else {
                usage(&format!("知らない指定です: {a}"))
            };
        };
        match name {
            "--by" => filter.by = Some(v.clone()),
            "--folder" => filter.folder = Some(normalized_folder(v)),
            "--since" => {
                if !is_time(v) {
                    return usage(&format!(
                        "--since は UTC の RFC 3339 の時刻（2026-11-01 ・ 2026-11-01T09:30:00Z など）で渡してください: {v}"
                    ));
                }
                filter.since = Some(v.clone());
            }
            "--limit" => match v.parse::<usize>() {
                Ok(n) if n > 0 => filter.limit = Some(n),
                _ => return usage(&format!("--limit は1以上の整数で渡してください: {v}")),
            },
            "--operation" => operation = Some(v.clone()),
            _ => return usage(&format!("知らない指定です: {a}")),
        }
    }
    if operation.is_some() && filter != OperationFilter::default() {
        return usage("--operation は1件を読むので、経路やほかの絞り方と一緒に渡せません");
    }
    let path = match db_path(&PathBuf::from(".")) {
        Ok(p) => p,
        Err(r) => return refuse(r),
    };
    // 記録が無ければ、作らずに「無い」として返す（読んでも何も変えない）
    let log = if path.exists() {
        match SqliteLog::open(&path) {
            Ok(l) => Some(l),
            Err(message) => {
                return refuse(Refused {
                    code: "log",
                    message,
                    ..Refused::default()
                });
            }
        }
    } else {
        None
    };
    if let Some(id) = operation {
        let found = match &log {
            Some(l) => read_operation(l, &id),
            None => Ok(None),
        };
        return match found {
            Ok(Some(d)) => {
                output::print_operation_detail(&d);
                ExitCode::SUCCESS
            }
            Ok(None) => refuse(Refused {
                code: "not_found",
                message: format!("操作 {id} は記録に無い"),
                ..Refused::default()
            }),
            Err(message) => refuse(log_refused(message)),
        };
    }
    let found = match &log {
        Some(l) => read_operations(l, &filter),
        None => Ok(Vec::new()),
    };
    match found {
        Ok(rows) => {
            for o in &rows {
                output::print_operation(o);
            }
            ExitCode::SUCCESS
        }
        Err(message) => refuse(log_refused(message)),
    }
}

/// --folder の値を、sessions の folder と比べられる形（実体の絶対の経路）にする。無いフォルダは、作業フォルダから引いた絶対の経路。
fn normalized_folder(given: &str) -> String {
    let p = PathBuf::from(given);
    if let Ok(real) = std::fs::canonicalize(&p) {
        return real.display().to_string();
    }
    let absolute = if p.is_absolute() {
        p
    } else {
        std::env::current_dir().unwrap_or_default().join(p)
    };
    let mut out = PathBuf::new();
    for part in absolute.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out.display().to_string()
}

/// UC7 の --since に渡せる時刻か（UTC の RFC 3339。日付だけ ・ 分まで ・ 秒まででもよい）。記録の時刻と文字の順で比べる。
fn is_time(t: &str) -> bool {
    let shape = "dddd-dd-ddTdd:dd:ddZ";
    let digits = |s: &str, at: &str| {
        s.len() == at.len()
            && s.chars()
                .zip(at.chars())
                .all(|(c, a)| if a == 'd' { c.is_ascii_digit() } else { c == a })
    };
    [10, 16, 19, 20].into_iter().any(|n| digits(t, &shape[..n]))
}

/// semantic-agent-schema search：呼ぶ側が渡した範囲を、構造で絞るか、意味で探す手順を返すか、渡された答えの出どころを確かめる（UC6）。記録しない。
fn run_search(args: &[String]) -> ExitCode {
    let (schemas, args) = match take_list(args, "--schemas") {
        Ok(v) => v,
        Err(m) => return usage(&m),
    };
    let mut semantic = false;
    let mut question = None;
    let mut answer = None;
    let mut conditions: Vec<(String, Value)> = Vec::new();
    let mut paths = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let slot = match a.as_str() {
            "--semantic" => {
                semantic = true;
                continue;
            }
            "--question" => &mut question,
            "--answer" => &mut answer,
            "--where" => {
                let Some(w) = it.next().filter(|w| !w.is_empty()) else {
                    return usage("--where に <JSON Pointer>=<値> がありません");
                };
                // 値は JSON だけを受け取る（文字列は "summary" のように JSON で書く）
                let Some((at, raw)) = w.split_once('=') else {
                    return usage(&format!(
                        "--where は <JSON Pointer>=<値> の形で渡してください: {w}"
                    ));
                };
                if !at.is_empty() && !at.starts_with('/') {
                    return usage(&format!(
                        "--where の所は JSON Pointer（/ で始まる）で渡してください: {at}"
                    ));
                }
                let Ok(value) = serde_json::from_str::<Value>(raw) else {
                    return usage(&format!(
                        "--where の値は JSON で渡してください（文字列は \"…\" で囲む）: {raw}"
                    ));
                };
                conditions.push((at.to_string(), value));
                continue;
            }
            s if s.starts_with("--") => return usage(&format!("知らない指定です: {s}")),
            p => {
                paths.push(p.to_string());
                continue;
            }
        };
        match it.next() {
            Some(v) if !v.is_empty() => *slot = Some(v.clone()),
            _ => return usage(&format!("{a} に値がありません")),
        }
    }
    // 範囲は必ず渡す（経路の並び。既定の範囲は持たない）
    if paths.is_empty() {
        return usage("探す範囲を、文書の経路の並びで渡してください");
    }
    let ways = [!conditions.is_empty(), semantic, answer.is_some()];
    match ways.iter().filter(|w| **w).count() {
        0 => {
            return usage(
                "探し方を、--where ・ --semantic --question ・ --answer のどれかで渡してください",
            );
        }
        1 => {}
        _ => return usage("--where ・ --semantic ・ --answer は一緒に渡せません"),
    }
    if semantic != question.is_some() {
        return usage("意味で探すときは、--semantic と --question <問い> を一緒に渡してください");
    }
    let answer_text = match &answer {
        Some(from) => match read_answer(from) {
            Ok(t) => Some(t),
            Err(r) => return refuse(r),
        },
        None => None,
    };
    let schemas = match schema_files(schemas) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let range = Range { paths, schemas };
    // 検索は誰が打ったかを取らず、記録を開かない
    let (store, validator) = match read_only() {
        Ok(a) => a,
        Err(r) => return refuse(r),
    };
    if let Some(q) = question {
        return match search_procedure(&range, &q, &store) {
            Ok(p) => {
                output::print_procedure(&p);
                ExitCode::SUCCESS
            }
            Err(r) => refuse(r),
        };
    }
    let result = match answer_text {
        Some(text) => search_with_answer(&range, &text, &store, &validator),
        None => search_where(&range, &conditions, &store),
    };
    match result {
        Ok(f) => {
            output::print_search(&f);
            ExitCode::SUCCESS
        }
        Err(r) => refuse(Refused::from(r)),
    }
}

/// 経路のファイルが、デザインシステムの受け取りの型を申告しているか。
fn is_design_system(path: &str) -> bool {
    std::fs::read_to_string(root().join(path))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| {
            v.get("$schema")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .is_some_and(|s| s == DESIGN_SYSTEM)
}

/// 置き場所を組み立て、誰が打ったかを決めて sessions に足してから（最初に semantic-agent-schema を打ったときに1行）、その値で f を呼ぶ。
/// f の断った理由（各モジュールの Refused）は、出力の形（output::Refused）へ移して返す。
fn run_with<T, E>(
    f: impl FnOnce(&WorkingTree, &Validator, &Caller, &mut SqliteLog) -> Result<T, E>,
    who: Who,
) -> Result<T, Refused>
where
    Refused: From<E>,
{
    let (store, validator, mut log) = adapters()?;
    let caller = caller(who)?;
    register(&caller, &mut log).map_err(log_refused)?;
    f(&store, &validator, &caller, &mut log).map_err(Refused::from)
}

fn refuse(r: impl Into<Refused>) -> ExitCode {
    let r = r.into();
    output::print_refused(&r);
    ExitCode::from(if r.code == "usage" { 2 } else { 1 })
}
