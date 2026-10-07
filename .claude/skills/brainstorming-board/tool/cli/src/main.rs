//! ブレストボードの CLI。転写した schema-driven のツール（作成 ・ 取得 ・ 更新 ・ 検査 ・ 承認 ・ 描画）を、
//! ボードの Design で使う。

use brainstorming_board::design::BoardDesign;
use brainstorming_board::tools::BoardTools;
use schema_driven_adapters::inbound::{cli, tools::Toolbox};
use schema_driven_adapters::outbound::document_design::DocumentDesign;
use schema_driven_adapters::outbound::fs::{master_root, FileSystem};
use schema_driven_adapters::outbound::jmespath::Jmespath;
use schema_driven_core::application::checks::Checks;
use schema_driven_core::application::instances::Instances;
use schema_driven_core::application::renders::Renders;
use std::sync::Arc;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (files, query) = (Arc::new(FileSystem), Arc::new(Jmespath));
    let instances = Instances::new(files.clone(), files.clone(), query.clone());
    let checks = Checks::new(files.clone(), files.clone(), query.clone());
    let renders = Renders::new(
        files.clone(),
        files,
        query,
        Arc::new(DocumentDesign),
        Some(Arc::new(BoardDesign)),
        None,
    );
    // ボードに固有のツール（init ・ inspect ・ migrate ・ serve）を、基盤のツールのあとに足す
    let toolbox = match Toolbox::with(Arc::new(BoardTools::new(master_root()))) {
        Ok(toolbox) => toolbox.with_render(Arc::new(renders)),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    let (code, out) = cli::run_in(&toolbox, &args, &instances, &checks);
    println!("{out}");
    std::process::exit(code);
}
