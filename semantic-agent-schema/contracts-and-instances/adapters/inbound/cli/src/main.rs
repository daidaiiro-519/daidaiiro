//! sas コマンドの入口。ステップ0 では、どのコマンドもまだ持たない。

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("sas: コマンドを指定してください（ステップ0 では、まだコマンドがありません）");
    } else {
        eprintln!("sas: 知らないコマンドです: {}", args[0]);
    }
    ExitCode::from(2)
}
