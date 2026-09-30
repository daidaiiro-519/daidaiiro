// SPDX-License-Identifier: MIT
//! MP3 の長さを、フレームの並びから測る。**外の道具に依存しない。**
//!
//! 読みの印の最後は、最後の語が**始まる**時刻である ── 音声の終わりではない。
//! **長さは音声そのものから測る。**

use std::io;
use std::path::Path;

use crate::data_access::files;

/// ビット率の表（MPEG1 Layer III）。
const BITRATE_V1: [u32; 16] = [
    0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0,
];
/// ビット率の表（MPEG2 ・ 2.5 Layer III）。
const BITRATE_V2: [u32; 16] = [
    0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0,
];

/// 標本化の周波数。**版ごとに違う。**
fn rate_of(version: u8, index: u8) -> Option<u32> {
    let table: [u32; 3] = match version {
        3 => [44100, 48000, 32000],
        2 => [22050, 24000, 16000],
        0 => [11025, 12000, 8000],
        _ => return None,
    };
    table.get(index as usize).copied()
}

/// 1フレームが含む標本の数。
const fn samples_of(version: u8) -> u32 {
    if version == 3 {
        1152
    } else {
        576
    }
}

/// 1フレームの長さと、含む標本の数と、周波数を返す。**フレームでなければ返さない。**
fn frame(head: &[u8]) -> Option<(usize, u32, u32)> {
    if head.len() < 4 || head[0] != 0xFF || (head[1] & 0xE0) != 0xE0 {
        return None;
    }
    let version = (head[1] >> 3) & 3;
    let layer = (head[1] >> 1) & 3;
    // 予約された版と、Layer III 以外は読まない
    if version == 1 || layer != 1 {
        return None;
    }
    let bitrate_index = head[2] >> 4;
    let rate_index = (head[2] >> 2) & 3;
    let padding = u32::from((head[2] >> 1) & 1);
    if bitrate_index == 0 || bitrate_index == 15 || rate_index == 3 {
        return None;
    }
    let rate = rate_of(version, rate_index)?;
    let table = if version == 3 { BITRATE_V1 } else { BITRATE_V2 };
    let kbps = *table.get(bitrate_index as usize)?;
    let samples = samples_of(version);
    let size = (samples / 8) * kbps * 1000 / rate + padding;
    if size > 4 {
        Some((size as usize, samples, rate))
    } else {
        None
    }
}

/// タグの長さを読む。**同期安全整数である** ── 各バイトの最上位は使わない。
fn tag_len(data: &[u8]) -> usize {
    if data.len() < 10 || &data[..3] != b"ID3" {
        return 0;
    }
    let raw = u32::from_be_bytes([data[6], data[7], data[8], data[9]]);
    let size = (raw & 0x7F)
        | ((raw >> 8 & 0x7F) << 7)
        | ((raw >> 16 & 0x7F) << 14)
        | ((raw >> 24 & 0x7F) << 21);
    10 + size as usize
}

/// 音声の長さをミリ秒で返す。
///
/// **フレームを数え上げる**ので、可変ビット率でも合う。
#[must_use]
pub fn duration_ms(data: &[u8]) -> u64 {
    let mut i = tag_len(data);
    let mut ms = 0.0_f64;
    while i + 4 <= data.len() {
        let Some((size, samples, rate)) = frame(&data[i..i + 4]) else {
            i += 1;
            continue;
        };
        ms += f64::from(samples) * 1000.0 / f64::from(rate);
        i += size;
    }
    // **四捨五入する** ── 切り捨てると、枚を重ねたときに尺が短く出る
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    {
        ms.round() as u64
    }
}

/// 音声のファイルを読み、長さをミリ秒で返す。**読むのはデータアクセス層である。**
///
/// # Errors
///
/// 読めないときに返す。
pub fn measure(path: &Path) -> io::Result<u64> {
    files::read(path).map(|data| duration_ms(&data))
}
