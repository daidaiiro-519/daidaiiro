// SPDX-License-Identifier: MIT
//! 音声の長さの測り方を、実物で検証する。
//!
//!     cargo test -p nar_business_logic
//!
//! **実物で測る。** 作った音声で測ると、作り方の誤りが検査を通ってしまう。

use std::path::PathBuf;

use nar_business_logic::mp3::duration_ms;

/// 実物の置き場所。**無ければ、その事例は飛ばす** ── 音声は追跡していない。
fn samples() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../..");
    let dir = root.join(".brainstorming-board/narration-skill/trial");
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "mp3"))
        .collect();
    out.sort();
    out
}

#[test]
fn a_real_recording_has_a_length() {
    for path in samples() {
        let data = std::fs::read(&path).expect("読める");
        let ms = duration_ms(&data);
        assert!(ms > 0, "{} の長さが0である", path.display());
        assert!(
            ms < 60 * 60 * 1000,
            "{} が1時間を超えている",
            path.display()
        );
    }
}

#[test]
fn an_empty_input_is_zero() {
    assert_eq!(duration_ms(&[]), 0);
    assert_eq!(
        duration_ms(&[0x00, 0x01, 0x02]),
        0,
        "フレームが無ければ0である"
    );
}

#[test]
fn a_tag_is_skipped() {
    // ID3 のタグだけで、フレームが無い ── 長さは0である
    let mut data = b"ID3\x04\x00\x00".to_vec();
    data.extend_from_slice(&[0, 0, 0, 10]); // 同期安全整数で10
    data.extend_from_slice(&[0_u8; 10]);
    assert_eq!(duration_ms(&data), 0);
}

#[test]
fn a_reserved_version_is_not_a_frame() {
    // 版が予約（1）のときは、フレームとして読まない
    let head = [0xFF_u8, 0xEA, 0x90, 0x00];
    assert_eq!(duration_ms(&head), 0);
}
