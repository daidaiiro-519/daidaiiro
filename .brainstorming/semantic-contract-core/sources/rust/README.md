# Rust と WASM の原典の照合

ボード semantic-contract-core の Q3 ・ Q4 の根拠。原文は fact-check fetch で取得し（このフォルダ。原文そのものは git で追跡しない）、引用は fact-check verify --as quote で照合した。

## 取得したもの

| URL | 取得した日 | 行数 | sha256 |
|---|---|---|---|
| https://doc.rust-lang.org/book/ch06-02-match.html |  | 605 | `2cae7e82ab7ab81c…` |
| https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html |  | 299 | `5884dbdf6a413cb6…` |
| https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html |  | 409 | `0866f4c6eb8d9bb1…` |
| https://doc.rust-lang.org/book/ch10-02-traits.html |  | 803 | `998c5a1c20235f25…` |
| https://doc.rust-lang.org/cargo/reference/features.html |  | 637 | `78c663fe82a9c62d…` |
| https://doc.rust-lang.org/cargo/reference/workspaces.html |  | 511 | `e40bb2c80864709a…` |
| https://raw.githubusercontent.com/bytecodealliance/wasmtime/main/README.md |  | 185 | `61b003ff4ddfbe61…` |
| https://raw.githubusercontent.com/bytecodealliance/wit-bindgen/main/README.md |  | 570 | `35e8ad58e7370419…` |

## 主張と引用

| 主張 | 引用（原文のまま） | 出どころ |
|---|---|---|
| crate はコンパイラが一度に扱うコードの最小の単位 | <p>A <em>crate</em> is the smallest amount of code that the Rust compiler considers at a | doc.rust-lang.org_book_ch07-01-packages-and-crates.html:186 |
| package は1つ以上の crate の束で、Cargo.toml を持つ | <p>A <em>package</em> is a bundle of one or more crates that provides a set of | 同:206 |
| module の中のものは、既定で外から見えない | module is private by default. Private items are internal implementation details | doc.rust-lang.org_book_ch07-02-defining-modules-to-control-scope-and-privacy.html:273 |
| trait は、型が持ちほかの型と共有できる機能を定める | <p>A <em>trait</em> defines the functionality a particular type has and can share with | doc.rust-lang.org_book_ch10-02-traits.html:187 |
| match はすべての場合を尽くさなければならない | pattern we forgot! Matches in Rust are <em>exhaustive</em>: We must exhaust every last | doc.rust-lang.org_book_ch06-02-match.html:465 |
| feature は条件付きのコンパイルと、任意の依存を表す | A package defines a set of | doc.rust-lang.org_cargo_reference_features.html:189〜190 |
| workspace は、一緒に管理する1つ以上の package の集まり | <p>A <em>workspace</em> is a collection of one or more packages, called <em>workspace | doc.rust-lang.org_cargo_reference_workspaces.html:189 |
| wit-bindgen は WASM へコンパイルされる側（guest）の結び付けを作り、Rust を含む | The `wit-bindgen` repository is currently focused on **guest** programs which | raw.githubusercontent.com_bytecodealliance_wit-bindgen_main_README.md:32〜34 |
| wasmtime は、実装を埋め込んでいろいろな言語から使える | You can use Wasmtime from a variety of different languages through embeddings of | raw.githubusercontent.com_bytecodealliance_wasmtime_main_README.md:136 |
| Rust からは wasmtime の crate で使える | * **[Rust]** - the [`wasmtime` crate] | 同:141 |

## 照合の結果

引用10件を `--as quote` で照合し、10件とも原文に在った。読めなかったファイルは無い。読んだのは、引用した段落の前後だけである。
