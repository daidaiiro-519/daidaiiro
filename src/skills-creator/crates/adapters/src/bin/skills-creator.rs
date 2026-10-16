//! skills-creator の CLI。結合と受け口を呼ぶだけ。

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (code, out) = skills_creator_adapters::inbound::cli::run(
        &args,
        &skills_creator_adapters::compose::own_dir(),
        &skills_creator_adapters::compose::env(),
    );
    println!("{out}");
    std::process::exit(code);
}
