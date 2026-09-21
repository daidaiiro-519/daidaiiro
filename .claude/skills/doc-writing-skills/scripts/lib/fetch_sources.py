# -*- coding: utf-8 -*-
"""出典の原文を取得し、MANIFEST.json に URL ・ sha256 ・ 取得した日を残す。

  python3 scripts/cli.py sources             sources/ へ取得し、MANIFEST.json を書く
  python3 scripts/cli.py sources --check 1   手元のものが MANIFEST と一致するかを検査する

**この Skill は、外の Skill に依存しない。**
使うのは Python の標準ライブラリだけである。
PDF のテキスト化だけは `pdftotext` が在れば行う。無ければ PDF をそのまま残す。
"""
from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import urllib.request
from datetime import datetime, timezone

HERE = os.path.dirname(os.path.abspath(__file__))
SKILL = os.path.dirname(HERE)
DIR = os.path.join(SKILL, "sources")
MANIFEST = os.path.join(DIR, "MANIFEST.json")
UA = "Mozilla/5.0"

URLS = [
    "https://www.bunka.go.jp/seisaku/bunkashingikai/kokugo/hokoku/pdf/93651301_01.pdf",
    "https://www.jtf.jp/pdf/jtf_style_guide.pdf",
    "https://www.domainlanguage.com/wp-content/uploads/2016/05/DDD_Reference_2015-03.pdf",
    "https://developers.google.com/style/highlights",
    "https://developers.google.com/style/headings",
    "https://developers.google.com/style/translation",
    "https://learn.microsoft.com/en-us/style-guide/top-10-tips-style-voice",
    "https://learn.microsoft.com/en-us/style-guide/scannable-content/",
]


def source_url(url: str) -> str:
    """developers.google.com は <URL>.md で原文を返す。"""
    return url + ".md" if url.startswith("https://developers.google.com/style/") else url


def file_name(url: str, fetched: str) -> str:
    n = re.sub(r"^https?://", "", url).rstrip("/")
    n = re.sub(r"[/?&=]", "_", n)
    return n + ".md" if fetched.endswith(".md") and not n.endswith(".md") else n


def sha256_of(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def to_text(pdf: str) -> None:
    """`pdftotext` が在れば、同名の .txt を作る。無ければ何もしない。"""
    if not shutil.which("pdftotext"):
        return
    subprocess.run(["pdftotext", "-layout", pdf, pdf[:-4] + ".txt"], check=False)


def fetch() -> int:
    os.makedirs(DIR, exist_ok=True)
    entries = []
    for url in URLS:
        src = source_url(url)
        name = file_name(url, src)
        path = os.path.join(DIR, name)
        try:
            req = urllib.request.Request(src, headers={"User-Agent": UA})
            with urllib.request.urlopen(req, timeout=120) as r, open(path, "wb") as f:
                shutil.copyfileobj(r, f)
        except Exception as e:                                  # noqa: BLE001
            print(f"取得できない {url}\n  {e}", file=sys.stderr)
            continue
        size = os.path.getsize(path)
        print(f"取得した {name}  {size:,} バイト")
        if name.endswith(".pdf"):
            to_text(path)
        entries.append({"url": url, "fetched": src, "file": name,
                        "sha256": sha256_of(path), "bytes": size})

    with open(MANIFEST, "w", encoding="utf-8") as f:
        json.dump({"fetched_at": datetime.now(timezone.utc).astimezone().isoformat(),
                   "sources": entries}, f, ensure_ascii=False, indent=1)
        f.write("\n")
    print(f"書いた {os.path.relpath(MANIFEST, SKILL)}（{len(entries)} 本）")
    if not shutil.which("pdftotext"):
        print("**`pdftotext` が無いので、PDF はテキストにしていない**", file=sys.stderr)
    return 0 if len(entries) == len(URLS) else 1


def check() -> int:
    if not os.path.exists(MANIFEST):
        print("MANIFEST.json が無い。先に取得する", file=sys.stderr)
        return 1
    with open(MANIFEST, encoding="utf-8") as f:
        entries = json.load(f)["sources"]
    bad = 0
    for e in entries:
        path = os.path.join(DIR, e["file"])
        if not os.path.exists(path):
            print(f"無い     {e['file']}"); bad += 1; continue
        if sha256_of(path) == e["sha256"]:
            print(f"一致     {e['file']}")
        else:
            print(f"食い違う {e['file']}"); bad += 1
    print(f"── 一致 {len(entries) - bad} ／ 一致しない {bad}")
    return 1 if bad else 0
