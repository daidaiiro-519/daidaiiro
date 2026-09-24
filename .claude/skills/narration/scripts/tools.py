# SPDX-License-Identifier: MIT
"""narration の道具の宣言。**能力の正本はここである。**

CLI も MCP も、この宣言から組む ── 能力を2回書くと、片方だけが古くなる。
"""
from __future__ import annotations

import pathlib
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from contract import Arg, Tool, result  # noqa: E402
from lib import polly  # noqa: E402


def plan(directory: str, voice: str = '') -> dict:
    """合成せずに、どの枚が合成になり、どの枚が取り出しになるかを出す。"""
    rows = polly.plan(directory, voice or None)
    findings = [f'{r["id"]}　{"取り出す" if r["cached"] else "合成する"}　{r["key"]}' for r in rows]
    return result(ok=True, findings=findings,
                  rows=[{k: v for k, v in r.items() if k != 'text'} for r in rows])


def _human_plan(res: dict) -> str:
    rows = res['data']['rows']
    new = sum(1 for r in rows if not r['cached'])
    return f'{len(rows)} 枚　／　合成する {new} 枚　／　取り出す {len(rows) - new} 枚'


def synth(directory: str, voice: str = '') -> dict:
    """原稿から、枚ごとの音声と尺を作り、継ぎ目の出力を書く。"""
    try:
        data = polly.run(directory, voice or None)
    except subprocess.CalledProcessError as e:
        return result(ok=False, findings=[e.stderr.decode('utf-8', 'replace').strip()[:300]])
    findings = [f'{i["id"]}　{i["durationMs"]} ms　{i["audio"]}' for i in data['items']]
    return result(ok=True, findings=findings, **data)


def _human_synth(res: dict) -> str:
    if not res['ok']:
        return '合成できなかった'
    d = res['data']
    total = sum(i['durationMs'] for i in d['items'])
    return (f'{len(d["items"])} 枚　／　合成 {len(d["made"])} 枚　／　取り出し {len(d["taken"])} 枚'
            f'　／　合計 {total / 1000:.1f} 秒')


def lexicon(path: str, name: str) -> dict:
    """読みの辞書を登録する。合成の前に、入力へ適用される。"""
    try:
        subprocess.run(['aws', 'polly', 'put-lexicon', '--name', name,
                        '--content', f'file://{path}'], check=True, capture_output=True,
                       stdin=subprocess.DEVNULL, timeout=60)
    except subprocess.CalledProcessError as e:
        return result(ok=False, findings=[e.stderr.decode('utf-8', 'replace').strip()[:300]])
    return result(ok=True, findings=[], name=name, path=path)


def _human_lexicon(res: dict) -> str:
    return f'登録した：{res["data"]["name"]}' if res['ok'] else '登録できなかった'


def measure(path: str) -> dict:
    """音声の長さを測る。**speech marks の最後の印ではなく、音声そのものから測る。**"""
    from lib.mp3 import duration_ms
    ms = duration_ms(path)
    return result(ok=True, findings=[], path=path, durationMs=ms)


def _human_measure(res: dict) -> str:
    return f'{res["data"]["durationMs"]} ms'


TOOLS = [
    Tool(name='plan', summary='合成せずに、合成と取り出しの内訳を出す',
         args=[Arg('directory', 'narration.json を置いたフォルダ'),
               Arg('voice', '声。省略すると narration.json の指定', required=False, default='')],
         run=plan, human=_human_plan),
    Tool(name='synth', summary='原稿から、枚ごとの音声と尺を作る',
         args=[Arg('directory', 'narration.json を置いたフォルダ'),
               Arg('voice', '声。省略すると narration.json の指定', required=False, default='')],
         run=synth, human=_human_synth),
    Tool(name='lexicon', summary='読みの辞書を登録する',
         args=[Arg('path', 'PLS準拠の辞書のファイル'), Arg('name', '辞書の名前')],
         run=lexicon, human=_human_lexicon),
    Tool(name='measure', summary='音声の長さを測る',
         args=[Arg('path', '音声のファイル')],
         run=measure, human=_human_measure),
]
