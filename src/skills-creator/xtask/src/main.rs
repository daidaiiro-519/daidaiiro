//! cargo xtask の入口。作業は lib.rs の perform が行い、ここは結果を出して終了コードを返すだけにする。

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (code, out) = xtask::perform(&args);
    println!("{out}");
    std::process::exit(code);
}
