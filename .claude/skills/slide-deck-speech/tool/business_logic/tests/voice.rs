// SPDX-License-Identifier: MIT
//! 音声の合成を事例で検証する。

#[test]
fn the_aws_command_passed_in_is_the_one_run() {
    // **Polly は注入されたコマンドで呼ぶ** ── 名前を直書きしないので、利用者は tool.json で差し替えられる
    let got = sds_business_logic::voice::put_lexicon(
        "/no/such/aws",
        std::path::Path::new("/tmp/none.pls"),
        "none",
    );
    assert!(got.is_err(), "渡したコマンドが無ければ、登録できない");
}
