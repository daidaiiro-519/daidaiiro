//! brainstorming の CLI。結合を呼ぶだけ。

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(brainstorming_adapters::compose::main(&args));
}
