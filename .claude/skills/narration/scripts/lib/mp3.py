# SPDX-License-Identifier: MIT
"""MP3 の長さを、フレームの並びから測る。外部の道具に依存しない。

speech marks の最後の印は、最後の語が**始まる**時刻である ── 音声の終わりではない。
長さは音声そのものから測る。
"""
from __future__ import annotations

import pathlib

_BITRATE = {
    1: [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0],   # MPEG1 Layer III
    2: [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0],       # MPEG2/2.5 Layer III
}
_RATE = {3: [44100, 48000, 32000], 2: [22050, 24000, 16000], 0: [11025, 12000, 8000]}
_SAMPLES = {3: 1152, 2: 576, 0: 576}


def _frame(head: bytes) -> tuple[int, int] | None:
    """1フレームの長さと、含む標本の数を返す。フレームでなければ None。"""
    if len(head) < 4 or head[0] != 0xFF or (head[1] & 0xE0) != 0xE0:
        return None
    ver, layer = (head[1] >> 3) & 3, (head[1] >> 1) & 3
    if ver == 1 or layer != 1:                      # 予約 ／ Layer III 以外
        return None
    bi, ri, pad = head[2] >> 4, (head[2] >> 2) & 3, (head[2] >> 1) & 1
    if bi in (0, 15) or ri == 3:
        return None
    rate = _RATE[ver][ri]
    kbps = _BITRATE[1 if ver == 3 else 2][bi]
    samples = _SAMPLES[ver]
    size = int(samples / 8 * kbps * 1000 / rate) + pad
    return (size, samples) if size > 4 else None


def duration_ms(path: str | pathlib.Path) -> int:
    """音声の長さをミリ秒で返す。フレームを数え上げるので、可変ビットレートでも合う。"""
    data = pathlib.Path(path).read_bytes()
    i, ms = 0, 0.0
    if data[:3] == b'ID3':                          # タグを飛ばす
        size = int.from_bytes(data[6:10], 'big')    # 同期安全整数
        i = 10 + ((size & 0x7F) | ((size >> 8 & 0x7F) << 7) |
                  ((size >> 16 & 0x7F) << 14) | ((size >> 24 & 0x7F) << 21))
    while i + 4 <= len(data):
        f = _frame(data[i:i + 4])
        if f is None:
            i += 1
            continue
        size, samples = f
        head = data[i:i + 4]
        rate = _RATE[(head[1] >> 3) & 3][(head[2] >> 2) & 3]
        ms += samples * 1000 / rate
        i += size
    return round(ms)
