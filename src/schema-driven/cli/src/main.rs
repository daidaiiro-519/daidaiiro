//! CLI の実行ファイル。アダプタを作り、入ってくる側の受け口へ渡す。

use schema_driven_adapters::inbound::{cli, tools::Toolbox};
use schema_driven_adapters::outbound::catalog::Catalog;
use schema_driven_adapters::outbound::fs::FileSystem;
use schema_driven_adapters::outbound::jmespath::Jmespath;
use schema_driven_core::application::checks::Checks;
use schema_driven_core::application::instances::Instances;
use schema_driven_core::application::renders::Renders;
use std::sync::Arc;

fn main() {
    let all: Vec<String> = std::env::args().skip(1).collect();
    let (skill, args) = cli::split_skill(&all);
    let (files, query) = (Arc::new(FileSystem), Arc::new(Jmespath));
    let mut use_cases = Instances::new(files.clone(), files.clone(), query.clone());
    let mut checks = Checks::new(files.clone(), files.clone(), query.clone());
    // 部品の見た目と操作は、基盤のカタログだけが持つ
    let mut renders = Renders::new(files.clone(), files.clone(), query, Arc::new(Catalog));
    // スキーマの名前は、--skill の Skill から引く
    if let Some(dir) = &skill {
        use_cases = use_cases.with_skill(dir);
        checks = checks.with_skill(dir);
        renders = renders.with_skill(dir);
    }
    let mut toolbox = Toolbox::base().with_render(Arc::new(renders));
    if let Some(dir) = &skill {
        toolbox = toolbox.with_skill(dir);
    }
    let (code, out) = cli::run_in(&toolbox, &args, &use_cases, &checks);
    println!("{out}");
    std::process::exit(code);
}
