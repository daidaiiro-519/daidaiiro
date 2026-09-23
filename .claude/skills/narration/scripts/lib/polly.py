# SPDX-License-Identifier: MIT
"""合成の実行と、作り直しの判定。

鍵は「読み上げる文・声・エンジン」から作る ── この3つが同じなら、音声は同じものになる。
鍵をそのまま音声の名前にするので、判定は存在の確認だけで済む。
"""
from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess

from .mp3 import duration_ms

MARKS = '["word", "sentence"]'


def key_of(text: str, voice: str, engine: str) -> str:
    """鍵を作る。文・声・エンジンのどれかが変われば、鍵も変わる。"""
    return hashlib.sha256(f'{text}|{voice}|{engine}'.encode()).hexdigest()[:16]


def load(directory: str | pathlib.Path) -> dict:
    """narration.json を読む。"""
    return json.loads((pathlib.Path(directory) / 'narration.json').read_text())


def _aws(args: list[str], out: pathlib.Path) -> None:
    subprocess.run(['aws', 'polly', 'synthesize-speech', *args, str(out)],
                   check=True, capture_output=True)


def synthesize(text: str, voice: str, engine: str, dest: pathlib.Path,
               lexicons: list[str] | None = None) -> None:
    """音声と、語と文の時刻を作る。どちらも同じ操作から得られる。"""
    common = ['--voice-id', voice, '--engine', engine, '--text', text]
    if lexicons:
        common += ['--lexicon-names', *lexicons]
    _aws([*common, '--output-format', 'mp3'], dest.with_suffix('.mp3'))
    _aws([*common, '--output-format', 'json', '--speech-mark-types', MARKS],
         dest.with_suffix('.marks.json'))


def plan(directory: str | pathlib.Path, voice: str | None = None) -> list[dict]:
    """枚ごとに、鍵と、合成が要るかどうかを出す。"""
    d = pathlib.Path(directory)
    doc = load(d)
    voice = voice or doc.get('voice', 'Takumi')
    engine = doc.get('engine', 'neural')
    cache = d / doc.get('cache', 'cache')
    rows = []
    for item in doc['items']:
        k = key_of(item['text'], voice, engine)
        rows.append({'id': item['id'], 'key': k, 'text': item['text'],
                     'audio': str((cache / f'{k}.mp3').relative_to(d)),
                     'marks': str((cache / f'{k}.marks.json').relative_to(d)),
                     'cached': (cache / f'{k}.mp3').exists()})
    return rows


def run(directory: str | pathlib.Path, voice: str | None = None) -> dict:
    """鍵が在る枚は取り出し、無い枚だけを合成して、継ぎ目の出力を書く。"""
    d = pathlib.Path(directory)
    doc = load(d)
    voice = voice or doc.get('voice', 'Takumi')
    engine = doc.get('engine', 'neural')
    lexicons = doc.get('lexicons') or []
    cache = d / doc.get('cache', 'cache')
    cache.mkdir(parents=True, exist_ok=True)
    made, taken, items = [], [], []
    for row in plan(d, voice):
        dest = cache / row['key']
        if dest.with_suffix('.mp3').exists():
            taken.append(row['id'])
        else:
            synthesize(row['text'], voice, engine, dest, lexicons)
            made.append(row['id'])
        items.append({'id': row['id'], 'key': row['key'], 'audio': row['audio'],
                      'marks': row['marks'], 'format': 'mp3',
                      'durationMs': duration_ms(dest.with_suffix('.mp3'))})
    out = {'voice': voice, 'engine': engine, 'items': items}
    (d / 'narration.out.json').write_text(
        json.dumps(out, ensure_ascii=False, indent=1) + '\n')
    return {'made': made, 'taken': taken, 'items': items}
