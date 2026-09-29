// SPDX-License-Identifier: MIT
//! テーマが満たすこと。
//!
//! **配色の正本は1つである** ── 図の側にも色を書くと、テーマを替えたときに図だけが
//! 前の配色のまま残る（実測 ── 4配色のうち1つで文字が消えた）。
//!
//!     cargo test -p sd_parts

use std::path::PathBuf;

use sd_parts::theme;

fn references() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

#[test]
fn every_theme_shares_the_same_keys() {
    let refs = references();
    let files = theme::theme_files(&refs);
    assert!(!files.is_empty(), "テーマが1本も無い");
    let first = theme::tokens(&std::fs::read_to_string(&files[0]).expect("読める"));
    for path in &files {
        let found = theme::tokens(&std::fs::read_to_string(path).expect("読める"));
        assert_eq!(
            found.keys().collect::<Vec<_>>(),
            first.keys().collect::<Vec<_>>(),
            "{} のキーが他と一致しない",
            path.display()
        );
    }
}

#[test]
fn nothing_is_found() {
    // **0件にできるものだけを検査している。**
    assert_eq!(theme::findings(&references(), &[]), Vec::<String>::new());
}

#[test]
fn a_theme_expands_into_role_colors() {
    let refs = references();
    for name in theme::theme_names(&refs) {
        let roles = theme::as_roles(&refs, &name).expect("複製できる");
        let got: Vec<&str> = roles.iter().map(|(k, _)| k.as_str()).collect();
        let want: Vec<&str> = theme::FIGURE_ROLES.iter().map(|(k, _)| *k).collect();
        assert_eq!(got, want, "{name}: 役割の並びが宣言と一致しない");
        assert!(
            roles.iter().all(|(_, v)| v.starts_with('#')),
            "{name}: 色になっていない値が在る"
        );
    }
}

#[test]
fn an_unknown_name_is_refused() {
    let why = theme::as_roles(&references(), "無いテーマ").expect_err("断る");
    assert!(why.contains("知らないテーマ"), "{why}");
    // **使える名前を添える** ── 断るだけでは、呼ぶ側が次に何を渡すか分からない
    assert!(why.contains("warm-paper"), "{why}");
}

#[test]
fn the_expanded_colors_are_a_copy_not_another_value() {
    let refs = references();
    let name = theme::theme_names(&refs)
        .first()
        .expect("1本は在る")
        .clone();
    let raw = theme::tokens(&theme::read(&refs, &name).expect("読める"));
    for (role, value) in theme::as_roles(&refs, &name).expect("複製できる") {
        let key = theme::FIGURE_ROLES
            .iter()
            .find(|(r, _)| *r == role)
            .map(|(_, k)| *k)
            .expect("宣言に在る");
        assert_eq!(value, raw[key], "{role} が正本と別の値である");
    }
}

#[test]
fn the_ratio_follows_the_source() {
    // 白と黒は 21、同じ色どうしは 1
    assert!((theme::ratio("#ffffff", "#000000") - 21.0).abs() < 0.01);
    assert!((theme::ratio("#808080", "#808080") - 1.0).abs() < 0.001);
    // **並びを入れ替えても同じ** ── 比であって差ではない
    assert!((theme::ratio("#000000", "#ffffff") - theme::ratio("#ffffff", "#000000")).abs() < 1e-9);
}

#[test]
fn a_three_digit_color_is_read() {
    assert!((theme::ratio("#fff", "#000") - 21.0).abs() < 0.01);
}

#[test]
fn only_a_declaration_is_read_as_a_color() {
    let found = theme::tokens("--ink: #111;\n--dim:#8a8;\n");
    assert_eq!(found["--ink"], "#111");
    assert_eq!(found["--dim"], "#8a8");
    // 宣言の形になっていないものは読まない
    assert!(theme::tokens("--ink #111;").is_empty(), "コロンが無い");
    assert!(theme::tokens("--ink: #111").is_empty(), "セミコロンが無い");
    assert!(theme::tokens("--ink: red;").is_empty(), "16進でない");
}

#[test]
fn the_last_declaration_is_the_one_that_holds() {
    let found = theme::tokens("--ink: #111;\n--ink: #222;\n");
    assert_eq!(found["--ink"], "#222", "後のものが残る");
}

#[test]
fn a_page_without_the_markers_is_found() {
    let dir = std::env::temp_dir().join("sd-theme");
    std::fs::create_dir_all(&dir).expect("作れる");
    let path = dir.join("nomark.html");
    std::fs::write(&path, "<html>#abcdef</html>").expect("書ける");
    let bad = theme::findings(&references(), &[path.display().to_string()]);
    assert!(
        bad.iter().any(|x| x.contains("テーマの区切りが無い")),
        "区切りが無いと、テーマの外の直書きを判定できない ── {bad:?}"
    );
}

#[test]
fn a_color_written_outside_the_theme_is_found() {
    let dir = std::env::temp_dir().join("sd-theme");
    std::fs::create_dir_all(&dir).expect("作れる");
    let path = dir.join("outside.html");
    std::fs::write(
        &path,
        "<style>.a{color:#abcdef}</style>\n▼ テーマ\n--ink: #111;\n▲ テーマここまで\n",
    )
    .expect("書ける");
    let bad = theme::findings(&references(), &[path.display().to_string()]);
    assert!(
        bad.iter().any(|x| x.contains("色の直書き #abcdef")),
        "{bad:?}"
    );
    // **テーマの内側は数えない** ── 数えると、正本そのものが違反になる
    assert!(!bad.iter().any(|x| x.contains("#111")), "{bad:?}");
}

#[test]
fn a_long_run_of_hex_is_not_a_color() {
    let dir = std::env::temp_dir().join("sd-theme");
    std::fs::create_dir_all(&dir).expect("作れる");
    let path = dir.join("long.html");
    std::fs::write(
        &path,
        "<p>#0123456789abcdef</p>\n▼ テーマ\n▲ テーマここまで\n",
    )
    .expect("書ける");
    let bad = theme::findings(&references(), &[path.display().to_string()]);
    assert!(
        !bad.iter().any(|x| x.contains("色の直書き")),
        "9桁以上の並びを色として数えない ── {bad:?}"
    );
}
