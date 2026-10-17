# sources/adapters：標準技術アダプターの技術選定の原典

ボード semantic-contract-core の論点「標準技術アダプターの技術選定」の根拠にする原典の置き場所である。構想 Semantic Contract Core v0.3 は、JSON Schema・JSON-LD・SHACL・RDF／SPARQL を、Rust のエンジンの部品として使うとしている（`../v0.3/docs/13_rust-engine-language-neutral-protocol.md:46〜54`、`../v0.3/images/semantic-contract-core-overview-illustrated.png` の「標準技術アダプター（部品として利用）」）。ここでは、その部品の候補が何をし、何をしないかを原文で照合した。

読み方は次のとおりである。

- 文書の頁は fact-check の `fetch` で取得した。OSS は `git clone --depth 1` で取得し、読んだ commit を記録した。
- 引用はすべて `fact-check verify --as quote` で原文と1文字ずつ照合した。識別子は `--as identifier` で照合した。`--as text` は使っていない。
- 「推測」と印を付けたものは、原文に書いていないことである。原典の記述として読まない。

---

## 1. 取得したもの

取得した日はすべて 2026-10-17（JST）。

### a. 頁（`fetch` で取得）

| ファイル | URL | 行数 | sha256 |
|---|---|---|---|
| `docs.rs_jsonschema_0.58.5_jsonschema` | https://docs.rs/jsonschema/0.58.5/jsonschema/ | 1,311 | `f23ac315df411561ebc508ec54db14ed4dade93a0520fb0be1db8fe393de7edb` |
| `docs.rs_json-ld_latest_json_ld` | https://docs.rs/json-ld/latest/json_ld/ | 573 | `cea20b00d78cbeb21c859dbdca01fe2382d5e509daa269ffb612b419431af41d` |
| `www.w3.org_TR_json-ld11-api` | https://www.w3.org/TR/json-ld11-api/ | 8,498 | `380ddb64232c701a8ff969e22646451e20fe66d264e34a5bc4c5b018e8c77f55` |
| `jena.apache.org_documentation_shacl` | https://jena.apache.org/documentation/shacl/ | 452 | `14af4c235b766c436c54500a3f8d2728566c9f2facec14af829e85cdbc68dc2e` |
| `crates.io_api_v1_crates_jsonschema` | https://crates.io/api/v1/crates/jsonschema | 1 | `a2c5f6cfeef05d65a68948ee1e058bcd39b6ec4c930234779803d04e10924206` |
| `crates.io_api_v1_crates_json-ld` | https://crates.io/api/v1/crates/json-ld | 1 | `bdd1378dc189926a4ee4c37c10b64320058489ce107a4c8aa888640299cb55e9` |
| `crates.io_api_v1_crates_oxigraph` | https://crates.io/api/v1/crates/oxigraph | 1 | `1d6b50ea90f96bdbc3770679c0c336f688423c5e67337df5a2bdfcab32ebd334` |
| `crates.io_api_v1_crates_shacl` | https://crates.io/api/v1/crates/shacl | 1 | `8a1ad97b7c4a948634f25f1c9e5e628baf3f4f5a9074b6ecd498ffbed0aa9d53` |
| `crates.io_api_v1_crates_shacl_validation` | https://crates.io/api/v1/crates/shacl_validation | 1 | `d34f71179267d0a10b7c29ddbf0423f1b3116e80835602048fa23f9ed9816307` |
| `crates.io_api_v1_crates_serde` | https://crates.io/api/v1/crates/serde | 1 | `72fb2287e1c089cd8362e0ccf22622a6965a432b8ef48f168abb72fc8ebc5ac1` |
| `crates.io_api_v1_crates_serde_json` | https://crates.io/api/v1/crates/serde_json | 1 | `822dd0c89815eb4db5fb2d16d3a490bd25bd78f2d0e4e60b627f4a8feae21fd2` |

- crates.io の5件は1行の JSON である。行番号は付けられないので、出どころはファイル名だけを書く。
- JSON-LD 1.1 本文（`../www.w3.org_TR_json-ld11`）と SHACL 本文（`../www.w3.org_TR_shacl`）は、取得済みのものを使った。取得の記録は `../README.md` の表にある。
- `.md` は試していない。

### b. OSS（`git clone --depth 1`）

| 置き場所 | リポジトリ | 読んだ commit | commit の日付 | リポジトリ上の版 |
|---|---|---|---|---|
| `repos/jsonschema-rust-v0.58.5/` | https://github.com/Stranger6667/jsonschema（タグ `rust-v0.58.5`） | `21a061d68984c7b1fb1aad3398a260c585ca89a9` | 2026-10-02 | `0.58.5` |
| `repos/json-ld/` | https://github.com/timothee-haudebourg/json-ld | `17854d02835dcac70f6e9b40f028e1b61e6ae8fd` | 2026-02-19 | `0.21.4` |
| `repos/oxigraph/` | https://github.com/oxigraph/oxigraph | `2283f42a7eb4ae40330a9c9ce7ee9585ff72daee` | 2026-10-12 | `0.6.0-dev` |
| `repos/rudof/` | https://github.com/rudof-project/rudof | `f3ec54562aa5c166ee83fc3ec28e234eaafcd036` | 2026-10-16 | `0.3.26` |
| `repos/pySHACL/` | https://github.com/RDFLib/pySHACL | `469cca7a22a078b36c167c1e8dadecf5e5ec6c75` | 2026-07-28 | `0.40.1` |

引用したファイルの行数と sha256 は次のとおりである。

| ファイル | 行数 | sha256 |
|---|---|---|
| `repos/jsonschema-rust-v0.58.5/README.md` | 185 | `c3bc4485c89eaf31dfe7a2d3ab47a505cc042f80b744bd9c7e5b897959efdc49` |
| `repos/jsonschema-rust-v0.58.5/crates/jsonschema/src/lib.rs` | 7,278 | `1218eff99cbb7e2513eae52362ad996ab9555cf7179d63cc15ecf3d242f4f1ae` |
| `repos/json-ld/README.md` | 324 | `d3c2f70ce52e8b1234b8581f09189a101a847baa99d8b5ec95d919db4bc79c63` |
| `repos/json-ld/crates/core/src/loader/mod.rs` | 550 | `9b63867680c41d15076f241862d798ba3f0b0a20ff3e9de5a9b7b29cea39c16b` |
| `repos/oxigraph/README.md` | 98 | `fd880759adcadb7a4b0ff479d9fe24f5113b2a87384e814b34e9b3091f461319` |
| `repos/oxigraph/lib/oxigraph/README.md` | 96 | `f4524367a360d37008c661ac68f68b33d1a6e64097c60e6cd82f2a48c1bd249f` |
| `repos/rudof/README.md` | 468 | `fd854dcf48a0d144e295f65b792ba66559eba242cf2cf8d5ff549545fbe81909` |
| `repos/rudof/shacl/README.md` | 31 | `5f9064ae7a37ae1a6370388c857a880b1a7f510c46735fe2718cc303d0d10d79` |
| `repos/pySHACL/README.md` | 432 | `e4b7290ef94fe7088a565ee837cb394c00b633662ce7e6fd0927a5ee59a56642` |
| `repos/pySHACL/FEATURES.md` | 282 | `10ff16d56036a6aeac8024b1144fa54f45f2a9a85f5072cc0c0bce9d854cae89` |

### c. 版について注意すること

| 対象 | 読んだ版 | 公開されている最新の版 | 出どころ |
|---|---|---|---|
| jsonschema | 0.58.5（いまの基盤が使う版） | `"max_stable_version":"0.59.0"` | `crates.io_api_v1_crates_jsonschema` |
| json-ld | 0.21.4 | `"max_stable_version":"0.21.4"` | `crates.io_api_v1_crates_json-ld` |
| oxigraph | `main` の `0.6.0-dev` | `"max_stable_version":"0.5.11"` | `crates.io_api_v1_crates_oxigraph`、`repos/oxigraph/Cargo.toml:25` |
| rudof の `shacl` | `main` の `0.3.26` | `"max_stable_version":"0.3.25"` | `crates.io_api_v1_crates_shacl`、`repos/rudof/Cargo.toml:35` |
| pySHACL | `master` の `0.40.1` | PyPI は取得していない | `repos/pySHACL/pyproject.toml:8` |

- jsonschema は、基盤が使う 0.58.5 をタグで読んだ。0.59.0 は読んでいない。
- oxigraph と rudof は、公開済みの版ではなく `main` を読んだ。公開済みの版と記述が違う可能性がある。

---

## 2. 照合した主張

### a. Rust の jsonschema 0.58.5

| 主張 | 引用（原文そのまま） | 出どころ |
|---|---|---|
| 対応する draft を並べている | The following drafts are supported: | `repos/jsonschema-rust-v0.58.5/README.md:92` |
| 対応の度合いは draft ごとに違い、新しい draft には実装していないキーワードがある | Compliance levels vary across drafts, with newer versions having some unimplemented keywords. | `…/crates/jsonschema/src/lib.rs:19` |
| 独自のキーワードで検証を足せる | `jsonschema` allows you to extend its functionality by implementing custom validation logic through custom keywords. | `…/src/lib.rs:839` |
| 独自のキーワードは `with_keyword` で登録する | 3. Register the custom keyword with the [`Validator`] instance using the [`ValidationOptions::with_keyword`] method | `…/src/lib.rs:846` |
| （コンパイル時の validator マクロ（`#[jsonschema::validator]`）の制限として）独自のキーワードで組み込みのキーワードを置き換えることはできず、両方が動く | - Custom keywords cannot override built-in ones: a `keywords` entry named like a built-in keyword runs in //! addition to the built-in check, not instead of it. | `…/src/lib.rs:249〜250`（見出し `//! ## Limitations` は `lib.rs:246`、その上の見出しは `lib.rs:149` の `Compile-Time Validator Macro`） |
| この crate が実装していない語彙を、対応すると宣言する口がある | /// Declare support for a vocabulary this crate does not implement itself. | `…/src/options.rs:409` |
| 未知の語彙を必須にしたメタスキーマは拒否される | /// A meta-schema that requires an unknown vocabulary is rejected. Use this together with | `…/src/options.rs:411` |
| 拒否するときのメッセージ | format!("Unknown vocabulary: '{uri}' is required by the meta-schema. Adjust configuration to declare support for it"), | `…/src/compiler.rs:1197` |
| 拒否するのは、実装していない語彙を必須にしたメタスキーマに対して書かれたスキーマである | /// A meta-schema that requires a vocabulary this crate does not implement cannot be honored, | `…/src/compiler.rs:1184` |
| （同上） | /// so the schemas written against it are refused. | `…/src/compiler.rs:1185` |
| `$vocabulary` の値が true の語彙だけを集める（Format-Assertion は値によらず集める） | if enabled.as_bool().unwrap_or(false) \|\| vocabulary == Vocabulary::FormatAssertion { | `…/crates/jsonschema-referencing/src/vocabularies.rs:191` |
| 集めた未知の語彙は「必須で、実装していないもの」である | /// Vocabularies that were required by the meta-schema but are not implemented here. | `…/jsonschema-referencing/src/vocabularies.rs:141` |
| 出力は JSON Schema Output v1 の形を返せる | The `evaluate()` method provides access to structured validation output formats defined by | `…/src/lib.rs:256` |
| 出力の各項目は評価の場所・スキーマの場所・インスタンスの場所を持つ（例） | "evaluationPath": "/items/type", | `…/src/lib.rs:339` |
| （同上） | "schemaLocation": "/items/type", | `…/src/lib.rs:340` |
| （同上） | "instanceLocation": "/1", | `…/src/lib.rs:341` |
| エラーごとにインスタンスの場所を取り出せる | eprintln!("Location: {}", error.instance_path()); | `…/README.md:31` |
| 既定では、HTTP の参照を `reqwest` で、ファイルの参照をファイルシステムから解決する | By default, `jsonschema` resolves HTTP references using `reqwest` and file references from the local file system. | `…/src/lib.rs:554` |
| 既定の機能の一覧 | default = ["resolve-http", "resolve-file", "tls-aws-lc-rs", "idna"] | `…/crates/jsonschema/Cargo.toml:17` |
| `resolve-http` は既定で有効で、HTTP で外の参照を取得する | \| `resolve-http` \| ✅ \| Fetching remote references over HTTP \| | `…/README.md:123` |
| `resolve-file` は既定で有効で、ファイルシステムから参照を読む | \| `resolve-file` \| ✅ \| Reading references from the filesystem \| | `…/README.md:124` |
| `default-features = false` にすると、参照の解決がすべて無効になる | - Disable all resolving: `default-features = false` | `…/src/lib.rs:588` |
| `resolve-http` が無いとき、HTTP の参照は失敗する | Err("`resolve-http` feature or a custom resolver is required to resolve external schemas via HTTP".into()) | `…/src/retriever.rs:318` |
| `idna` を外すと、`idn-hostname` と `idn-email` は未知の形式として扱われる | Turning off `idna` drops the IDNA and ICU dependencies; `idn-hostname` and `idn-email` are then treated as unknown formats. | `…/README.md:135〜136` |
| `offline` を使うと、登録に無い参照は取得せずに構築の時点で失敗する | [`ValidationOptions::offline`] disables retrieval, so a reference that is not already in the //! registry fails at construction instead of being fetched: | `…/src/lib.rs:759〜760` |
| `offline` のときのメッセージ | Err(format!("Retrieval is disabled, cannot fetch {uri}").into()) | `…/src/retriever.rs:111` |
| 取得の仕方は自分で実装して差し替えられる | A retriever is for documents fetched on demand — from a database, an embedded asset, a cache. | `…/src/lib.rs:592` |
| 2020-12 のメタスキーマは crate の中に持っている | schema!(pub DRAFT202012, "../metaschemas/draft2020-12/schema.json"); | `…/crates/jsonschema-referencing/src/meta.rs:48` |
| 必要な Rust の版 | This crate requires Rust 1.85.0 or later. | `…/README.md:140` |

docs.rs の頁（`docs.rs_jsonschema_0.58.5_jsonschema`）は、同じ `lib.rs` の文書から作られている。版の表示は `docs.rs_jsonschema_0.58.5_jsonschema:44` の `"version": "0.58.5"`、`default-features = false` の記述は同 `:941` にある。

### b. JSON-LD の Rust 実装 json-ld 0.21.4

| 主張 | 引用（原文そのまま） | 出どころ |
|---|---|---|
| 版 | version = "0.21.4" | `repos/json-ld/Cargo.toml:60` |
| 入口は `JsonLdProcessor` で、Context の処理・Expansion・Compaction などを持つ | The entry point for this library is the `JsonLdProcessor` trait that provides an access to all the JSON-LD transformation algorithms (context processing, expansion, compaction, etc.). | `repos/json-ld/README.md:29〜31` |
| Compaction を持つ | The JSON-LD Compaction is a transformation that consists in applying a | `repos/json-ld/README.md:158` |
| JSON-LD 1.1 の機能は、処理モードを 1.0 にしない限り使える | /// New features defined in JSON-LD 1.1 are available unless the processing mode is set to [`ProcessingMode::JsonLd1_0`]. | `repos/json-ld/crates/core/src/mode.rs:7` |
| ローダーが、`@context` で読み込むリモートの Context をすべて取得する | /// fetching all the remote contexts imported in a `@context` entry. | `repos/json-ld/crates/core/src/loader/mod.rs:438` |
| ローダーは trait で、差し替えられる | pub trait Loader { | `…/loader/mod.rs:453` |
| ローダーは非同期の関数である | async fn load(&self, url: &Iri) -> Result<RemoteDocument<IriBuf>, LoadError>; | `…/loader/mod.rs:468` |
| `NoLoader` は何も読み込まず、常に失敗する | ///   - [`NoLoader`] dummy loader that always fail. Perfect if you are certain | `…/loader/mod.rs:441` |
| IRI から読み込み済みの文書への対応表を使えば、ネットワークに出ない | ///     documents. This way no network calls are performed and the loaded | `…/loader/mod.rs:445` |
| `FsLoader` はローカルのフォルダへ向け、ネットワークに出ない | ///     directory on the file system. This also avoids network calls. The loaded | `…/loader/mod.rs:448` |
| `ReqwestLoader` は実際にダウンロードする | ///   - `ReqwestLoader` actually downloading the remote documents using the | `…/loader/mod.rs:450` |
| `ReqwestLoader` は `reqwest` 機能を有効にしたときだけ使える | ///     This requires the `reqwest` feature to be enabled. | `…/loader/mod.rs:452` |
| core の既定の機能は空である | default = [] | `repos/json-ld/crates/core/Cargo.toml:15` |
| 最後に公開された版と日付 | "max_stable_version":"0.21.4" | `crates.io_api_v1_crates_json-ld` |
| （同上） | "created_at":"2026-02-19T17:08:34.498193Z" | `crates.io_api_v1_crates_json-ld` |
| CHANGELOG の最新の見出しは 0.21.2 のままである | ## [0.21.2] - 2024-12-19 | `repos/json-ld/CHANGELOG.md:8` |
| W3C の JSON-LD API のテストスイートを取り込んでテストを作る | [W3C JSON-LD API repository](https://github.com/w3c/json-ld-api) containing the | `repos/json-ld/README.md:300` |

### c. JSON-LD 1.1 本文と API：リモートの Context の読み込み

| 主張 | 引用（原文そのまま） | 出どころ |
|---|---|---|
| API の仕様が、リモートの文書と Context を読み込む仕組みを定める（この節は非規範） | This includes a general mechanism for loading remote documents, | `../www.w3.org_TR_json-ld11:9861` |
| （同上） | including referenced JSON-LD documents and remote contexts, | `../www.w3.org_TR_json-ld11:9862` |
| リモートの Context はキャッシュするべきである | &lt;li&gt;Remote context documents should be cached to prevent overloading the | `../www.w3.org_TR_json-ld11:9871` |
| プライバシーの節：リモートの Context をキャッシュするか、documentLoader でローカルの版を持つことを考えるべきである | To protect against this, publishers should consider caching remote contexts for future use, | `../www.w3.org_TR_json-ld11:12634` |
| リモートの Context は、利用者が1件ずつ求めなくても自動で辿られるのがふつうである | &lt;p&gt;When processing JSON-LD documents, links to remote contexts and frames are | `../www.w3.org_TR_json-ld11:13063` |
| （同上） | typically followed automatically, resulting in the transfer of files | `../www.w3.org_TR_json-ld11:13064` |
| （同上） | without the explicit request of the user for each one. If remote | `../www.w3.org_TR_json-ld11:13065` |
| 重要な用途では、リモートの Context を確かめてキャッシュしてから使うよう勧めている | depends on a remote context for mission critical purposes vet and | `../www.w3.org_TR_json-ld11:13075` |
| （同上） | cache the remote context before allowing the system to use it.&lt;/p&gt; | `../www.w3.org_TR_json-ld11:13076` |
| API：`documentLoader` を渡せばそれを使い、渡さなければ処理系の組み込みのローダーを使う | &lt;dd&gt;The callback of the loader to be used to retrieve remote documents and contexts, | `www.w3.org_TR_json-ld11-api:7448` |
| （同上） | otherwise, if not specified, the processor's built-in loader is used.&lt;/dd&gt; | `www.w3.org_TR_json-ld11-api:7451` |
| API：Context を取得できなければ、`loading remote context failed` として処理を止める | &lt;li id="alg-context-deref-error"&gt;If &lt;var&gt;context&lt;/var&gt; cannot be dereferenced, | `www.w3.org_TR_json-ld11-api:2547`（続きは `:2552〜2553`） |
| （同上）エラーの定義 | &lt;dd&gt;There was a problem encountered loading a remote context.&lt;/dd&gt; | `www.w3.org_TR_json-ld11-api:7980` |
| API の版 | "generatedSubtitle": "Recommendation 16 July 2020" | `www.w3.org_TR_json-ld11-api:983` |

HTML の原文の `<` `>` は、表を崩さないように `&lt;` `&gt;` と書いた。照合は `<` `>` のままの文字列で行った。

### d. RDF と SPARQL の Rust 実装 oxigraph（`main`、0.6.0-dev）

| 主張 | 引用（原文そのまま） | 出どころ |
|---|---|---|
| グラフデータベースであり、RDF と SPARQL のツールキットである | Oxigraph is both a graph database and a [RDF](https://www.w3.org/TR/rdf11-primer/) and [SPARQL](https://www.w3.org/TR/sparql11-overview/) toolkit. | `repos/oxigraph/README.md:13` |
| RocksDB を土台にする | Its goal is to provide a compliant, safe, and fast graph database based on the [RocksDB](https://rocksdb.org/) key-value store. | `repos/oxigraph/README.md:15` |
| 開発が盛んな段階で、SPARQL の評価はまだ最適化していない | Oxigraph is in heavy development and SPARQL query evaluation has not been optimized yet. | `repos/oxigraph/README.md:18` |
| SPARQL 1.1 の Query・Update・Federated Query を実装する | * [SPARQL 1.1 Query](https://www.w3.org/TR/sparql11-query/), [SPARQL 1.1 Update](https://www.w3.org/TR/sparql11-update/), and [SPARQL 1.1 Federated Query](https://www.w3.org/TR/sparql11-federated-query/). | `repos/oxigraph/lib/oxigraph/README.md:21` |
| ほとんどの実装は 1.1 の勧告にほぼ完全に適合し、1.2 の草案に予備的に対応する | Most implementations are nearly fully conformant with the latest recommendations (1.1 for RDF, SPARQL and JSON-LD) with preliminary support for 1.2 RDF and SPARQL drafts. | `repos/oxigraph/README.md:28` |
| RDF 1.2 と SPARQL 1.2 は `rdf-12` 機能で使える | Support for [RDF 1.2](https://www.w3.org/TR/rdf12-concepts/) and [SPARQL 1.2](https://www.w3.org/TR/sparql12-query/) is also available behind the `rdf-12` feature. | `repos/oxigraph/lib/oxigraph/README.md:25` |
| JSON-LD を RDF の直列化の形式として読み書きする | and [JSON-LD](https://www.w3.org/TR/json-ld/) RDF serialization formats for both data ingestion and retrieval. | `repos/oxigraph/README.md:24` |
| Rust のライブラリとして、プロセスの中で `Store` を作って使う | let store = Store::new().unwrap(); | `repos/oxigraph/lib/oxigraph/README.md:35` |
| 既定の機能は `rocksdb` である | default = ["rocksdb"] | `repos/oxigraph/lib/oxigraph/Cargo.toml:17` |
| `rocksdb` を外すとメモリの上だけで動く | It is possible to disable the RocksDB storage backend to only use the in-memory fallback by disabling the `rocksdb` default feature: | `repos/oxigraph/lib/oxigraph/README.md:76` |
| （同上）書き方 | oxigraph = { version = "*", default-features = false } | `repos/oxigraph/lib/oxigraph/README.md:78` |
| RocksDB は既定でこの crate の中でビルドする | By default it builds RocksDB as part of this crate. | `repos/oxigraph/oxrocksdb-sys/README.md:6` |
| そのビルドは C++ をコンパイルする | config.cpp(true).file("api/c.cc").compile("oxrocksdb_api"); | `repos/oxigraph/oxrocksdb-sys/build.rs:58` |
| HTTP の取得は `http-client` 機能で、既定には入っていない | http-client = ["oxhttp"] | `repos/oxigraph/lib/oxigraph/Cargo.toml:20`（既定は同 `:17`） |

### e. SHACL の実装

**SHACL 本文（`../www.w3.org_TR_shacl`）**

| 主張 | 引用（原文そのまま） | 出どころ |
|---|---|---|
| SHACL は SHACL Core と SHACL-SPARQL に分かれる | The SHACL specification is divided into SHACL Core and SHACL-SPARQL. | `../www.w3.org_TR_shacl:1056` |
| すべての実装は少なくとも SHACL Core を実装しなければならない | least implement SHACL Core. | `../www.w3.org_TR_shacl:1058`（前の行 `:1057` の `All SHACL implementations` と `MUST` に続く） |
| shapes graph は RDF グラフである | A &lt;dfn data-dfn-type="dfn" id="dfn-shapes-graph"&gt;shapes graph&lt;/dfn&gt; is an RDF graph containing zero or more shapes that is passed into a SHACL &lt;a href="#dfn-validation" class="internalDFN" data-link-type="dfn"&gt;validation&lt;/a&gt; process so | `../www.w3.org_TR_shacl:2259` |
| data graph は任意の RDF グラフである | Any RDF graph can be a &lt;dfn data-dfn-type="dfn" id="dfn-data-graph"&gt;data graph&lt;/dfn&gt;. | `../www.w3.org_TR_shacl:2277` |
| shapes graph は `owl:imports` で別の shapes graph を参照でき、処理系は検証の前にそれを辿って取り込むべきである | Shapes graphs can be reusable validation modules that can be cross-referenced with the predicate &lt;a href="https://www.w3.org/TR/owl2-syntax/#Imports"&gt;&lt;code&gt;owl:imports&lt;/code&gt;&lt;/a&gt;. As a pre-validation step, SHACL processors &lt;em class="rfc2119" | `../www.w3.org_TR_shacl:2264`（`SHOULD` と「transitively following and importing」は `:2265`） |

**rudof の `shacl`（Rust、`main`、0.3.26）**

| 主張 | 引用（原文そのまま） | 出どころ |
|---|---|---|
| RDF と知識グラフを処理する、Rust で書いたツールである | This repo contains an RDF and Knowledge Graphs processing tool implemented in Rust. | `repos/rudof/README.md:11` |
| Rust のライブラリとして使える | The code can be used as a Rust library | `repos/rudof/README.md:20` |
| Linux・Windows・Mac・Docker のバイナリを配る | We provide binaries for Linux, Windows, Mac and Docker | `repos/rudof/README.md:24` |
| SHACL に対応する | - SHACL | `repos/rudof/README.md:54` |
| SHACL の crate の名前は `shacl` である | name = 'shacl' | `repos/rudof/shacl/Cargo.toml:2` |
| SHACL の実装 | [SHACL](https://www.w3.org/TR/shacl/) implementation in Rust. | `repos/rudof/shacl/README.md:3` |
| SHACL core の抽象構文を表し、検証の手順を実装する | - [SHACL](https://docs.rs/shacl), that represents the [SHACL core abstract syntax](https://www.w3.org/TR/shacl) and implements the SHACL validation algorithm. | `repos/rudof/docs/src/overview.md:110` |
| `Native` はメモリの上のグラフを Rust で検証する | - **`Native`** — designed for in-memory graphs. Constraints are evaluated with Rust | `repos/rudof/shacl/README.md:11` |
| `sparql` 機能を有効にすると、`Native` でも `sh:sparql`（SHACL-SPARQL）の制約を検証する | logic. When the `sparql` Cargo feature is enabled, inline `sh:sparql` (SHACL-SPARQL) constraints are routed to the SPARQL execution path so that in-memory | `repos/rudof/shacl/README.md:12〜13` |
| `Sparql` は外の SPARQL エンドポイントに対して検証する | - **`Sparql`** — designed for validating against a remote SPARQL endpoint. Every | `repos/rudof/shacl/README.md:17` |
| `shacl` の既定の機能は `sparql` である | default = ["sparql"] | `repos/rudof/shacl/Cargo.toml:14` |
| `sparql` 機能は `rudof_rdf/sparql` を有効にする | sparql = ["dep:sparql_service", "sparql_service/sparql", "rudof_rdf/sparql"] | `repos/rudof/shacl/Cargo.toml:15` |
| `rudof_rdf` の `sparql` 機能は oxigraph と reqwest を依存に入れる | "dep:oxigraph", | `repos/rudof/rudof_rdf/Cargo.toml:15` |
| （同上） | "dep:reqwest", | `repos/rudof/rudof_rdf/Cargo.toml:16` |
| 実装していない部分がコードに残っている（例） | todo!("shnex:matchAll is not yet implemented"); | `repos/rudof/shacl/src/rdf/parsers/node_expr/match_all.rs:12` |
| 0.2.19〜0.3.0 の版は使わないよう求めている | >The use of versions between 0.2.19 and 0.3.0 (inclusive) is discouraged due to some problems with the CI/CD pipeline. | `repos/rudof/README.md:43` |
| README の部品の一覧は、今は無いフォルダ `shacl_validation` を指したままである | - [shacl_validation](https://github.com/rudof-project/rudof/tree/master/shacl_validation) contains the code required to validate RDF using SHACL. | `repos/rudof/README.md:276` |
| crates.io の `shacl_validation` は 0.2.12 で止まっている | "max_stable_version":"0.2.12" | `crates.io_api_v1_crates_shacl_validation` |
| （同上）最後の更新 | "updated_at":"2026-04-22T05:00:33.005486Z" | `crates.io_api_v1_crates_shacl_validation` |
| crates.io の `shacl` の版 | "max_stable_version":"0.3.25" | `crates.io_api_v1_crates_shacl` |

依頼では `shacl_validation` を候補に挙げていた。読んだ commit には `shacl_validation` というフォルダが無く、SHACL の検証は `shacl` の crate にある（`repos/rudof/shacl/Cargo.toml:2`）。

**pySHACL（Python、`master`、0.40.1）**

| 主張 | 引用（原文そのまま） | 出どころ |
|---|---|---|
| Python だけで書かれ、RDF グラフを SHACL のグラフで検証する | This is a pure Python module which allows for the validation of [RDF](https://www.w3.org/2001/sw/wiki/RDF) graphs against Shapes Constraint Language ([SHACL](https://www.w3.org/TR/shacl/)) graphs. | `repos/pySHACL/README.md:10` |
| SHACL の勧告に従うよう開発している | This module is developed to adhere to the SHACL Recommendation: | `repos/pySHACL/README.md:12` |
| SPARQL による制約 `sh:sparql` は完成している | \| `sh:sparql`             \| `SPARQLConstraintComponent`   \| [▶][SPARQLConstraintComponent]    \| ![status-complete]	\|                            \| | `repos/pySHACL/FEATURES.md:81` |
| SHACL Advanced Features と SHACL-JS を有効にする旗がある | - `-a` enable SHACL Advanced Features | `repos/pySHACL/README.md:70` |
| （同上） | - `-j` enable SHACL-JS Features (if `pyshacl[js]` is installed) | `repos/pySHACL/README.md:71` |
| SHACL のテストスイートで通らないものが2件ある | - core/property/datatype-ill-formed.ttl : Waiting on RDFLib support for determining ill-formed Literals https://github.com/RDFLib/rdflib/issues/848 | `repos/pySHACL/FEATURES.md:201` |
| （同上） | - sparql/pre-binding/shapesGraph-001.ttl : Prebinding to $shapesGraph is currently unsupported. This will be supported in the future. | `repos/pySHACL/FEATURES.md:202` |
| `owl:imports` で別のグラフを取り込む機能がある | * `do_owl_imports`: Enable the feature to allow the import of subgraphs using `owl:imports` for the shapes graph and the ontology graph. | `repos/pySHACL/README.md:237` |
| 必要な Python の版 | requires-python = ">=3.9" | `repos/pySHACL/pyproject.toml:9` |

**Apache Jena の jena-shacl**

| 主張 | 引用（原文そのまま） | 出どころ |
|---|---|---|
| W3C の SHACL の実装である | &lt;p&gt;&lt;code&gt;jena-shacl&lt;/code&gt; is an implementation of the | `jena.apache.org_documentation_shacl:194` |
| SHACL Core と SHACL SPARQL Constraints を実装する | It implements SHACL Core and SHACL SPARQL Constraints.&lt;/p&gt; | `jena.apache.org_documentation_shacl:196` |

### f. 構想の docs/13 が挙げる候補の識別子

`../v0.3/docs/13_rust-engine-language-neutral-protocol.md` を `--as identifier` で照合した。5個とも在った。

| 識別子 | 構想の行 | crates.io の名前（`--as quote` で照合） | 公開されている最新の版 |
|---|---|---|---|
| `serde` | `:48` | `"name":"serde"` | `"max_stable_version":"1.0.229"` |
| `serde_json` | `:48` | `"name":"serde_json"` | `"max_stable_version":"1.0.154"` |
| `jsonschema` | `:49` | `"name":"jsonschema"` | `"max_stable_version":"0.59.0"` |
| `json-ld` | `:50` | `"name":"json-ld"` | `"max_stable_version":"0.21.4"` |
| `oxigraph` | `:51` | `"name":"oxigraph"` | `"max_stable_version":"0.5.11"` |

構想の該当行は次のとおりである。

| 引用（原文そのまま） | 出どころ |
|---|---|
| - `json-ld`: Linked Dataの処理。具体的な互換・成熟度はPoCで確認。 | `../v0.3/docs/13_rust-engine-language-neutral-protocol.md:50` |
| - `oxigraph`: 必要な場合のRDF/SPARQL用候補。 | 同 `:51` |
| - SHACL処理: Rustライブラリまたは外部の成熟実装をAdapter越しに活用（全仕様対応を前提としない）。 | 同 `:52` |
| 既存ライブラリの選択は内部実装の問題。**言語非依存の公開仕様を変更する理由にしてはならない。** | 同 `:54` |
| - 安全性: 外部Interpreterへのアクセス権、ネットワーク解決可否、計算量と実行時間制約 | 同 `:35` |
| **SHACLは一般のJSONをそのまま検証する仕様ではない**。 | `../v0.3/docs/12_jsonld-rdf-shacl-concept-review.md:4` |

「SHACL処理」は crate の名前ではない。構想は SHACL の部品の名前を挙げていない。

---

## 3. 照合できなかったもの

| 何を | どうだったか |
|---|---|
| json-ld 0.21.4 が JSON-LD 1.1 のテストスイートにどれだけ通るか | README（`repos/json-ld/README.md`）と docs.rs の頁には、テストの動かし方（`:290〜305`）しか無く、通った割合は書いていない。読めた範囲には無い |
| jsonschema 0.58.5 の draft ごとの対応の度合い | README は Bowtie のバッジの画像を指すだけで（`README.md:94〜98`）、数値は画像の側にある。画像は取得していない |
| rudof の `shacl` が SHACL のテストスイートにどれだけ通るか | README は数値を書いていない。`repos/rudof/shacl/tests/` は読んでいない |
| Apache Jena の版 | 取得した頁に、どの版の記述かが書いていない |
| pySHACL の公開済みの版 | PyPI を取得していない。`master` の `pyproject.toml` だけを読んだ |
| oxigraph 0.5.11（公開済みの版）の README | 読んだのは `main`（0.6.0-dev）である |

---

## 4. 読めなかった範囲

- `git clone --depth 1` は submodule を取得していない。oxigraph の RocksDB のソース（`repos/oxigraph/oxrocksdb-sys/rocksdb`）、json-ld と rudof のテストスイートは空のままである。
- `repos/` の下で読んだのは、表に挙げたファイルと、行番号を添えたファイルだけである。それ以外のソースは読んでいない。
- jsonschema の Bowtie のバッジ（README の画像）は取得していない。

---

## 5. いまの基盤の約束とぶつかる点

いまの基盤の約束は「ネットワークに出ない」「言語に依存しない口」「3 OS で配布」の3つである。いまの基盤は `jsonschema = { version = "0.58.5", default-features = false }` を使う（`/src/schema-driven/crates/core/Cargo.toml:10`）。

**[推測]** と付けた行は原文に書いていないことである。

### a. JSON Schema（jsonschema）

| 約束 | 原文で言えること | 推測 |
|---|---|---|
| ネットワークに出ない | 既定の機能では HTTP で外の参照を取得する（`README.md:123`、`lib.rs:554`）。`default-features = false` で参照の解決がすべて無効になる（`lib.rs:588`）。`offline` を使えば、登録に無い参照は構築の時点で失敗する（`lib.rs:759〜760`） | **[推測]** いまの基盤は `default-features = false` なので、ここではぶつからない |
| 独自の語彙を足す | 独自のキーワードは `with_keyword` で足せる（`lib.rs:846`）。未知の語彙を必須にしたメタスキーマは、`with_vocabulary` で宣言しない限り拒否される（`options.rs:411`、`compiler.rs:1197`） | **[推測]** 独自の語彙を `$vocabulary` で true にすれば、宣言していない処理系では止まる。構想の fail-closed（`../v0.3/docs/14_standards-as-optional-adapters.md` の7原則の5）と同じ向きに働く |
| 言語に依存しない口 | 出力は JSON Schema Output v1 の形を返せる（`lib.rs:256`）。各項目は `evaluationPath`・`schemaLocation`・`instanceLocation` を持つ（`lib.rs:339〜341`） | **[推測]** 独自のキーワードは Rust の trait で書くので、同じ検証を別の言語で再現するには別に実装が必要になる |
| 3 OS で配布 | 原文に記述は無い | **[推測]** いまの基盤ですでに使っており、新しくぶつかる点は無い |

### b. JSON-LD（json-ld）

| 約束 | 原文で言えること | 推測 |
|---|---|---|
| ネットワークに出ない | JSON-LD 1.1 は、リモートの Context が自動で辿られるのがふつうだと書く（`json-ld11:13063〜13065`）。API は、渡したローダーを使い、渡さなければ組み込みのローダーを使う（`json-ld11-api:7448〜7451`）。Context を取得できなければ処理を止める（`json-ld11-api:2547`、`:7980`）。Rust の json-ld はローダーを trait で差し替えられ（`loader/mod.rs:453`）、対応表や `FsLoader` ならネットワークに出ない（`:445`、`:448`）。HTTP で取得する `ReqwestLoader` は `reqwest` 機能を有効にしたときだけ使える（`:452`）。core の既定の機能は空である（`crates/core/Cargo.toml:15`） | **[推測]** 使う Context を先にファイルとして持ち、対応表のローダーを渡せば、ネットワークに出ずに済む。持っていない Context を参照した文書は、処理が止まる |
| 言語に依存しない口 | ローダーは非同期の関数である（`loader/mod.rs:468`） | **[推測]** いまの core は同期で動いているので、呼ぶ側に非同期の実行の仕組みが必要になる。いまの基盤では tokio は adapters にだけある（`/src/schema-driven/crates/adapters/Cargo.toml:7`） |
| 保守の状態 | 最新の版は 0.21.4（2026-02-19）である（crates.io）。CHANGELOG の最新の見出しは 0.21.2（2024-12-19）のままである（`CHANGELOG.md:8`） | **[推測]** 構想の docs/13 が「具体的な互換・成熟度はPoCで確認」（`:50`）としているとおり、テストスイートの通り方は試して確かめる必要がある |
| 3 OS で配布 | 原文に記述は無い | **[推測]** C や C++ のビルドが必要かは、依存を読んでいないので分からない |

### c. RDF／SPARQL（oxigraph）

| 約束 | 原文で言えること | 推測 |
|---|---|---|
| ネットワークに出ない | HTTP の取得は `http-client` 機能で、既定には入っていない（`lib/oxigraph/Cargo.toml:17`、`:20`）。SPARQL 1.1 Federated Query を実装する（`lib/oxigraph/README.md:21`） | **[推測]** `http-client` を入れなければ、Federated Query の `SERVICE` は外へ出られない。その場合の振る舞いは読んでいない |
| 3 OS で配布 | 既定の機能は `rocksdb` で（`Cargo.toml:17`）、RocksDB をこの crate の中で C++ としてビルドする（`oxrocksdb-sys/README.md:6`、`build.rs:58`）。`default-features = false` でメモリの上だけで動く（`lib/oxigraph/README.md:76〜78`） | **[推測]** 既定のままだと、3 OS のそれぞれで C++ のコンパイラが必要になる。`default-features = false` にすればこの点は外れる |
| 成熟の度合い | 開発が盛んな段階で、SPARQL の評価はまだ最適化していない（`README.md:18`）。公開済みの版は 0.5.11、`main` は 0.6.0-dev である | **[推測]** 版が 1.0 に届いていないので、版を上げるたびに API が変わる前提で固定する必要がある |
| 言語に依存しない口 | 原文に記述は無い | ― |

### d. SHACL（rudof の `shacl`、pySHACL、jena-shacl）

| 約束 | 原文で言えること | 推測 |
|---|---|---|
| 入力の形 | shapes graph も data graph も RDF グラフである（`shacl:2259`、`:2277`）。構想も「SHACLは一般のJSONをそのまま検証する仕様ではない」と書く（`../v0.3/docs/12_jsonld-rdf-shacl-concept-review.md:4`） | **[推測]** JSON のインスタンスを SHACL で検証するには、先に JSON-LD などで RDF グラフへ変える処理が必要になる。SHACL を足すことは、JSON-LD か RDF の部品も足すことになる |
| ネットワークに出ない | 本文は、shapes graph の `owl:imports` を辿って取り込むべきだとする（`shacl:2264〜2265`）。rudof の `shacl` の既定の機能 `sparql` は、`rudof_rdf/sparql` を通して reqwest を依存に入れる（`shacl/Cargo.toml:14〜15`、`rudof_rdf/Cargo.toml:15〜16`）。`Sparql` の検証の仕方は外の SPARQL エンドポイントを相手にする（`shacl/README.md:17`） | **[推測]** rudof の `shacl` を使うなら、既定の機能を外し、`Native` の仕方だけを使う形を確かめる必要がある。`owl:imports` をどう扱うかも確かめる |
| 3 OS で配布 | rudof は Linux・Windows・Mac のバイナリを配っている（`README.md:24`）。pySHACL は Python だけで書かれ、Python 3.9 以上を必要とする（`README.md:10`、`pyproject.toml:9`）。jena-shacl は Apache Jena の一部である（`jena.apache.org_documentation_shacl:194`） | **[推測]** pySHACL と jena-shacl は Rust のエンジンに組み込めない。使うなら Python か Java の実行環境を配る側に求めることになり、「3 OS へバイナリ1つで配る」形とぶつかる。外の処理系として呼ぶなら、プロセスをまたぐ口が必要になる |
| 対応の範囲 | jena-shacl は SHACL Core と SHACL SPARQL Constraints を実装する（`jena…shacl:196`）。pySHACL は `sh:sparql` を完成とし、テストスイートで通らないものを2件挙げる（`FEATURES.md:81`、`:201〜202`）。rudof の `shacl` は SHACL core を実装するとし（`overview.md:110`）、`sparql` 機能では `sh:sparql` も検証する（`shacl/README.md:12〜13`）。実装していない部分がコードに残る（`match_all.rs:12`） | **[推測]** 3つとも SHACL-SPARQL の制約に触れているが、テストスイートの通り方を数で示しているのは、通らないものを挙げた pySHACL だけである。rudof の `shacl` の対応の範囲は、テストスイートを動かして確かめる必要がある |
| 保守の状態 | rudof は 0.2.19〜0.3.0 の版を使わないよう求めている（`README.md:43`）。README は今は無い `shacl_validation` を指したままである（`README.md:276`）。crates.io の `shacl_validation` は 0.2.12 で止まっている | **[推測]** 使うなら crate の名前は `shacl` で、版は 0.3 系を固定する |
