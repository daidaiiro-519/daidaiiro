//! catalog/ のファイルを数え、取り込む一覧を Rust のコードにして OUT_DIR に書く。

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("catalog");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("{} を読めない：{error}", dir.display()))
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();
    files.sort();
    let mut out = String::from("pub static CATALOG: &[(&str, &str)] = &[\n");
    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());
        let path = path.canonicalize().unwrap();
        let file = path.file_name().unwrap().to_string_lossy();
        writeln!(
            out,
            "    ({file:?}, include_str!({:?})),",
            path.display().to_string()
        )
        .unwrap();
    }
    out.push_str("];\n");
    let target = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("catalog.rs");
    fs::write(target, out).unwrap();
}
