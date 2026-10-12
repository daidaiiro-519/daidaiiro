//! CLI の実行ファイル。起動時の結合（compose）を CLI の受け口へ渡し、結果を出す。

use schema_driven_adapters::compose;
use schema_driven_adapters::inbound::cli;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (code, out) = cli::run(&args, compose::toolbox);
    println!("{out}");
    std::process::exit(code);
}
