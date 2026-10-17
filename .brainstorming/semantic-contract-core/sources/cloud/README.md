# AWS Lambda の Rust 対応の原文

Schema 駆動の基盤（Rust 製）を AWS Lambda で動かせるかを判断するために取得した原文である。
どれも `fact-check fetch` で取得し、引用は `fact-check verify --as quote` で1文字ずつ照合した（13件すべて合格）。

## 取得したもの

| ファイル | URL | 取得した日 | 行数 |
|---|---|---|---|
| `aws.amazon.com_about-aws_whats-new_2025_11_aws-lambda-rust` | https://aws.amazon.com/about-aws/whats-new/2025/11/aws-lambda-rust/ （HTML のまま） | 2026-10-17 | 1,825 |
| `docs.aws.amazon.com_lambda_latest_dg_lambda-rust.html.md` | https://docs.aws.amazon.com/lambda/latest/dg/lambda-rust.html.md | 2026-10-17 | 33 |
| `docs.aws.amazon.com_lambda_latest_dg_rust-package.html.md` | https://docs.aws.amazon.com/lambda/latest/dg/rust-package.html.md | 2026-10-17 | 148 |
| `docs.aws.amazon.com_lambda_latest_dg_rust-http-events.html.md` | https://docs.aws.amazon.com/lambda/latest/dg/rust-http-events.html.md | 2026-10-17 | 69 |
| `docs.aws.amazon.com_lambda_latest_dg_runtimes-provided.html.md` | https://docs.aws.amazon.com/lambda/latest/dg/runtimes-provided.html.md | 2026-10-17 | 24 |
| `docs.aws.amazon.com_lambda_latest_dg_lambda-runtimes.html.md` | https://docs.aws.amazon.com/lambda/latest/dg/lambda-runtimes.html.md | 2026-10-17 | 210 |
| `docs.aws.amazon.com_lambda_latest_dg_lambda-runtime-environment.html.md` | https://docs.aws.amazon.com/lambda/latest/dg/lambda-runtime-environment.html.md | 2026-10-17 | 293 |
| `docs.aws.amazon.com_lambda_latest_dg_gettingstarted-limits.html.md` | https://docs.aws.amazon.com/lambda/latest/dg/gettingstarted-limits.html.md | 2026-10-17 | 252 |
| `docs.aws.amazon.com_lambda_latest_dg_configuration-ephemeral-storage.html.md` | https://docs.aws.amazon.com/lambda/latest/dg/configuration-ephemeral-storage.html.md | 2026-10-17 | 84 |
| `docs.aws.amazon.com_lambda_latest_dg_configuration-function-zip.html.md` | https://docs.aws.amazon.com/lambda/latest/dg/configuration-function-zip.html.md | 2026-10-17 | 224 |
| `docs.aws.amazon.com_lambda_latest_dg_troubleshooting-deployment.html.md` | https://docs.aws.amazon.com/lambda/latest/dg/troubleshooting-deployment.html.md | 2026-10-17 | 213 |
| `docs.aws.amazon.com_lambda_latest_dg_urls-configuration.html.md` | https://docs.aws.amazon.com/lambda/latest/dg/urls-configuration.html.md | 2026-10-17 | 264 |
| `docs.aws.amazon.com_lambda_latest_dg_web-functions.html.md` | https://docs.aws.amazon.com/lambda/latest/dg/web-functions.html.md | 2026-10-17 | 62 |
| `docs.aws.amazon.com_lambda_latest_dg_web-functions-runtimes.html.md` | https://docs.aws.amazon.com/lambda/latest/dg/web-functions-runtimes.html.md | 2026-10-17 | 90 |
| `raw.githubusercontent.com_awslabs_aws-lambda-rust-runtime_main_README.md` | https://raw.githubusercontent.com/awslabs/aws-lambda-rust-runtime/main/README.md （リポジトリは aws/aws-lambda-rust-runtime へ移っている。取得した日の main は `5670a2ed2efae728bf01b4de78297b670696d225`、`git ls-remote` で確認） | 2026-10-17 | 613 |

`sha256` とバイト数は、それぞれの `<ファイル>.meta.json` が持つ。

## 主張と引用

| # | 主張 | 引用（原文のまま） | 出どころ |
|---|---|---|---|
| 1 | Rust は 2025-11-14 に正式な対応（GA）になった。それまでは experimental だった | Previously, AWS classified Rust support in Lambda as ‘experimental’ and did not recommend using Rust for production workloads. With this launch, Rust support in Lambda is now Generally Available, backed by AWS Support and the Lambda SLA. ／ Posted on: <!-- -->Nov 14, 2025 | `aws.amazon.com_about-aws_whats-new_2025_11_aws-lambda-rust:1625` ・ `:1622` |
| 2 | Rust は専用のランタイムを持たず、Rust runtime client で作って OS のみのランタイムへ載せる | Because Rust compiles to native code, you don't need a dedicated runtime to run Rust code as a Lambda Event Function. Instead, use the [Rust runtime client](https://github.com/aws/aws-lambda-rust-runtime) to build your project locally, and then deploy it to Lambda using an [OS-only runtime](runtimes-provided.md). | `docs.aws.amazon.com_lambda_latest_dg_lambda-rust.html.md:6` |
| 3 | いま使える OS のみのランタイムは `provided.al2023`（廃止 2029-06-30）。`provided.al2` は 2026-07-31 に廃止 | \| OS-only Runtime \| `provided.al2023` \| Amazon Linux 2023 \|  Jun 30, 2029  \|  Jul 31, 2029  \|  Aug 31, 2029  \| ／ \| OS-only Runtime \|  provided.al2  \| Amazon Linux 2 \|  Jul 31, 2026  \|  Jul 29, 2027  \|  Aug 31, 2027  \| | `docs.aws.amazon.com_lambda_latest_dg_runtimes-provided.html.md:20` ・ `docs.aws.amazon.com_lambda_latest_dg_lambda-runtimes.html.md:171` |
| 4 | 実行ファイルは Linux 向けに、x86_64 か arm64 でビルドする | You must compile your binary for a Linux environment and for the same instruction set architecture that you plan to use for the function (x86\_64 or arm64). | `docs.aws.amazon.com_lambda_latest_dg_runtimes-provided.html.md:11` |
| 5 | AWS の文書は、Rust runtime client を Lambda のツールとして挙げ、Cargo Lambda を第三者の OSS と位置づける | [Rust runtime client for Lambda](https://github.com/aws/aws-lambda-rust-runtime): The Rust runtime client makes it easy to run Lambda functions written in Rust. ／ [Cargo Lambda](https://www.cargo-lambda.info/guide/what-is-cargo-lambda.html): This is a third-party open-source extension to the Cargo command-line tool that simplifies building and deploying Rust Lambda functions. | `docs.aws.amazon.com_lambda_latest_dg_lambda-rust.html.md:10` ・ `:11` |
| 6 | runtime の crate は `lambda-runtime`、HTTP は `lambda-http`（API Gateway ・ ALB ・ 関数の URL ・ VPC Lattice）。始めるには Cargo Lambda を勧める | \*\*`lambda-runtime`\*\* is a library that provides a Lambda runtime for applications written in Rust. ／ \*\*`lambda-http`\*\* is a library for writing HTTP Lambda functions in Rust, with support for upstream API Gateway, ALB, Lambda Function URLs, and VPC Lattice. ／ The easiest way to start writing Lambda functions with Rust is by using [Cargo Lambda](https://www.cargo-lambda.info/), a related project. | `raw.githubusercontent.com_awslabs_aws-lambda-rust-runtime_main_README.md:7` ・ `:8` ・ `:15` |
| 7 | パッケージの上限は、zip で 50 MB（API ・ SDK 経由。大きいものは S3）、展開後はレイヤーを含めて 250 MB | 50 MB (zipped, when uploaded through the Lambda API or SDKs). Upload larger files with Amazon S3. ／ 250 MB The maximum size of the contents of a deployment package, including layers and custom runtimes. (unzipped) | `docs.aws.amazon.com_lambda_latest_dg_gettingstarted-limits.html.md:64` |
| 8 | `/tmp` は実行環境ごとの一時の領域で、512 MB〜10,240 MB。プロセスとスレッドは合わせて 1,024 まで | Lambda provides ephemeral storage for functions in the `/tmp` directory. This storage is temporary and unique to each execution environment. ／ \| `/tmp` directory storage \| Between 512 MB and 10,240 MB, in 1-MB increments \| ／ \| Execution processes/threads \| 1,024 | `docs.aws.amazon.com_lambda_latest_dg_configuration-ephemeral-storage.html.md:6` ・ `docs.aws.amazon.com_lambda_latest_dg_gettingstarted-limits.html.md:68` ・ `:70` |
| 9 | 同梱する実行ファイルは 755 の権限にする | In Linux permissions octal notation, Lambda needs 644 permissions for non-executable files (rw-r--r--) and 755 permissions (rwxr-xr-x) for directories and executable files. | `docs.aws.amazon.com_lambda_latest_dg_troubleshooting-deployment.html.md:38` |
| 10 | HTTP で呼ぶ手段は関数の URL と API Gateway（ほかに ALB も HTTP の事象を送れる）。Web Functions は Node.js だけ | Lambda offers two ways to invoke your function through an HTTP endpoint: function URLs and Amazon API Gateway. ／ Amazon API Gateway APIs, Application Load Balancers, and [Lambda function URLs](urls-configuration.md) can send HTTP events to Lambda. ／ Lambda Web Functions currently support Node.js. | `docs.aws.amazon.com_lambda_latest_dg_urls-configuration.html.md:9` ・ `docs.aws.amazon.com_lambda_latest_dg_rust-http-events.html.md:6` ・ `docs.aws.amazon.com_lambda_latest_dg_web-functions-runtimes.html.md:6` |

## 推測（引用を添えられない）

- 関数の中で、同梱した実行ファイルを子プロセスとして起動できること。読んだ範囲に、これを直接述べる文は無い。#8 のプロセスの上限と #9 の実行ファイルの権限を繋いで導いた
- デプロイしたパッケージの置き場所（`/var/task`）が読み取り専用であること。読んだ範囲に「read-only」の語は無い
