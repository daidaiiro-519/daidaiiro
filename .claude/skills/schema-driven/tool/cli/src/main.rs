//! CLI の実行ファイル。アダプタを作り、入ってくる側の受け口へ渡す。

use schema_driven_adapters::inbound::{cli, tools::Toolbox};
use schema_driven_adapters::outbound::document_design::DocumentDesign;
use schema_driven_adapters::outbound::{fs::FileSystem, jmespath::Jmespath};
use schema_driven_core::application::checks::Checks;
use schema_driven_core::application::instances::Instances;
use schema_driven_core::application::renders::Renders;
use std::sync::Arc;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (files, query) = (Arc::new(FileSystem), Arc::new(Jmespath));
    let use_cases = Instances::new(files.clone(), files.clone(), query.clone());
    let checks = Checks::new(files.clone(), files.clone(), query.clone());
    // 基盤だけで使うときは、具体のデザインが無いので、基盤の文書だけを描画する
    let renders = Renders::new(
        files.clone(),
        files,
        query,
        Arc::new(DocumentDesign),
        None,
        None,
    );
    let toolbox = Toolbox::base().with_render(Arc::new(renders));
    let (code, out) = cli::run_in(&toolbox, &args, &use_cases, &checks);
    println!("{out}");
    std::process::exit(code);
}
