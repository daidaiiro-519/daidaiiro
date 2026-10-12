//! schemas/ のファイルを数え、取り込む一覧を Rust のコードにして OUT_DIR に書く。
//! スキーマは自分の `$id` を名前として持つ。名前の無いスキーマがあれば、ビルドを止める。
//! clippy.toml の禁止は domain と application のためのもので、ビルドのときにファイルを読むこの build.rs には当てない。
#![allow(clippy::disallowed_methods)]

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

fn files_in(dir: &Path) -> Vec<PathBuf> {
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("{} を読めない：{error}", dir.display()))
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();
    files.sort();
    for file in &files {
        println!("cargo:rerun-if-changed={}", file.display());
    }
    files
}

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut out = String::from("pub static SCHEMAS: &[Schema] = &[\n");
    for path in files_in(&root.join("schemas")) {
        let path = path.canonicalize().unwrap();
        let schema: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path).unwrap())
            .unwrap_or_else(|error| panic!("{} が JSON でない：{error}", path.display()));
        let name = schema["$id"]
            .as_str()
            .unwrap_or_else(|| panic!("{} に $id が無い", path.display()));
        let file = path.file_name().unwrap().to_string_lossy();
        writeln!(
            out,
            "    Schema {{ name: {name:?}, file: {file:?}, text: include_str!({:?}) }},",
            path.display().to_string()
        )
        .unwrap();
    }
    out.push_str("];\n");
    let target = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("schemas.rs");
    fs::write(target, out).unwrap();
}
