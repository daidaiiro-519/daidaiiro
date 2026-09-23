# SPDX-License-Identifier: MIT
"""長さの測定を、実物の音声で検証する。"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from lib.mp3 import duration_ms  # noqa: E402

CASES = [  # 試作で ffprobe が測った値（ミリ秒）
    ('d352853ff5c14b1b.mp3', 13488),
    ('621b2e478dcfc782.mp3', 18264),
    ('a7c9d8473fe785e5.mp3', 41736),
]
ROOT = pathlib.Path(__file__).resolve().parents[5] / '.brainstorming-board/narration-skill/trial/cache'


def main() -> int:
    bad = 0
    for name, want in CASES:
        p = ROOT / name
        if not p.exists():
            print(f'とばした（音声が無い）: {name}')
            continue
        got = duration_ms(p)
        ok = abs(got - want) <= 60          # 60ms 以内なら一致とする
        print(f'{"OK " if ok else "×  "}{name}  測定 {got}ms ／ 期待 {want}ms')
        bad += 0 if ok else 1
    print(f'検証 {len(CASES)} 件 ／ 不一致 {bad} 件')
    return 1 if bad else 0


if __name__ == '__main__':
    raise SystemExit(main())
