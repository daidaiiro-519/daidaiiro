//! CLI の実行ファイル。アダプタを組み立て、入ってくる側の受け口へ渡す。

use schema_driven_adapters::inbound::cli;
use schema_driven_adapters::outbound::{fs::FileSystem, jmespath::Jmespath};
use schema_driven_core::application::instances::Instances;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (files, query) = (FileSystem, Jmespath);
    let use_cases = Instances::new(&files, &query);
    let (code, out) = cli::run(&args, &use_cases);
    println!("{out}");
    std::process::exit(code);
}
