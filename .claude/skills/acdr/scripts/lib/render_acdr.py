# SPDX-License-Identifier: MIT
"""意思決定の記録を、節と差分を持つ1枚のHTMLへ組む。

    python3 scripts/cli.py render <記録のフォルダ>

**この道具は、変更後の中身を複製しない。** 節が持つのは決定であり、
変更そのものは対象の文書の上に印として出る（tabs.build が組む）。
対象が無い決定（新規）は、節だけの1枚になる ── 例外にせず、同じ器で空にする。

**HTML の形は、ここが持たない** ── `references/acdr.template.html` が持つ。
"""
from __future__ import annotations

import html as _h
import datetime as _dt
import hashlib
import json
import pathlib
import sys

from . import REFERENCES
from . import acdr_css as _css
from . import validate_input as _vi
from .template import part as _t
from .panes import build as build_docs, check as check_docs

# 承認の状態。これ以外を書かせない ── 状態が自由文になると、
# 「承認されているか」を読む側が判定することになる
STATUS = {
    "proposed": ("提案", "まだ承認を得ていない。適用してはならない"),
    "accepted": ("承認", "承認を得た。適用してよい"),
    "superseded": ("差し替え済み", "後の記録が、この決定を置き換えた"),
}

# 節。**欠けたら止まる** ── 欠けた記録は、あとから誰にも補えない。
# **正本は validate_input が保持する** ── 2か所に持つと、片方だけが動いても誰も検出しない
SECTIONS = list(_vi.SECTIONS)

# 見た目は acdr_css が保持する。**CSS の正本は1つである**
CSS = _css.SECTION


def _list(items: list[str]) -> str:
    if not items:
        return _t("none", text="無し")
    return _t("list", items="".join(_t("list-item", item=v) for v in items))


def _dropped(rows: list) -> str:
    """比較した案。**どれも反証を通過しなかったものである** ── 未解決の懸念ではない。"""
    if not rows:
        return _t("none", text="比較した案は無い")
    return _t("dropped", rows="".join(
        _t("dropped-row", option=r["option"], why_not=r["why_not"]) for r in rows))


def _shift(rows: list) -> str:
    """変更前と変更後を、抽象の側で並べる。

    **具体の差分は面が持つ。** ここが持つのは、何がどう変わるかの形である ──
    抽象の対比が無いと、下に並ぶ具体の差分が何のためかを読み手が復元することになる。
    """
    return _t("shift", rows="".join(
        _t("shift-row", what=r["what"], before=r["from"], after=r["to"])
        for r in rows))


def header(spec: dict) -> str:
    """節を組む。**欠けている節が在れば止まる。**

    題はここが持つ ── 面の見出しと二重に出さない。
    """
    missing = [k for k in SECTIONS if not str(spec.get(k, "")).strip()]
    if missing:
        raise ValueError(f"節が欠けている: {'・'.join(missing)} ── "
                         "欠けた記録は、あとから誰にも補えない")
    st = spec.get("status", "proposed")
    if st not in STATUS:
        raise ValueError(f"状態が「{st}」。使えるのは {'／'.join(STATUS)} である")
    label, note = STATUS[st]
    num = _h.escape(str(spec.get("no", "")))
    when = _h.escape(str(spec.get("date", "")))

    secs = [("なぜ、いま決めるのか", _t("para", body=spec["why"]))]
    if spec.get("shift"):
        secs.append(("形の変化", _shift(spec["shift"])))
    if spec.get("_図"):
        secs.append(("図で確認する", _t("figure", svg=spec["_図"],
                                    caption=spec.get("figure_caption", ""))))
    if spec.get("how"):
        secs.append(("実現の形", _t("para", body=spec["how"])))
    secs += [("適用先", _t("para", body=spec["applies_to"])),
             ("比較した案", _dropped(spec.get("alternatives", []))),
             ("承認後に実施すること", _list(spec.get("after_approval", [])))]
    if spec.get("supersedes"):
        secs.append(("supersedes", _t("para", body=spec["supersedes"])))
    body = "".join(_t("sec", heading=k, body=v) for k, v in secs)

    return _t("acdr", chip=_t("chip", no=num) if num else "",
              title=_h.escape(spec["title"]), status=st, label=label, note=note,
              when=when, decision=spec["decision"], body=body)


def build(spec: dict) -> tuple[str, int]:
    docs = spec.get("docs", [])
    head = header(spec)
    if not docs:
        # 新規の決定。差分が無いので、節だけの1枚になる
        from .panes import CSS as BASE
        return _t("page-bare", title=_h.escape(spec["title"]),
                  style=BASE + CSS, head=head), 0

    # 題は節が持つ。面の見出しと二重に出さない ── 面の見出しの場所を、節で差し替える。
    # 節と面のあいだに橋を1行置く ── 下に並ぶのは、この決定を採ったときの具体である
    inner = dict(spec)
    inner["_heading"] = head + _t("bridge")
    inner["_css"] = CSS
    return build_docs(inner)


def check(out: str, spec: dict, total: int) -> list[tuple[str, bool]]:
    ok = [("節が在る", '<div class="acdr">' in out),
          ("状態が付いている", 'class="st ' in out)]
    if spec.get("docs"):
        ok += check_docs(out, spec, total)
    return ok


ROOT_MARK = ".git"


def repo_root(start: pathlib.Path) -> pathlib.Path:
    """リポジトリの根を探す。**パスを実行場所に依存させない** ──
    依存させると、どこから呼んだかで結果が変わり、冪等でなくなる。"""
    for d in [start, *start.parents]:
        if (d / ROOT_MARK).exists():
            return d
    return start


def load(folder: pathlib.Path) -> dict:
    """記録のフォルダから spec を読む。図は隣のファイルから読み、JSON へ埋め込まない。"""
    spec = json.loads((folder / "acdr.json").read_text(encoding="utf-8"))
    # **パスはリポジトリの根から解決する** ── 実行場所に依存させると、冪等でなくなる
    root = repo_root(folder)
    validate(spec, folder=folder, root=root)
    fig = spec.get("figure")
    if fig:
        spec["_図"] = (folder / fig).read_text(encoding="utf-8")
    for d in spec.get("docs", []):
        d["file"] = str(root / d["file"])
    return spec


def validate(spec: dict, *, folder: pathlib.Path | None = None,
             root: pathlib.Path | None = None) -> None:
    """入力を検査する。**1件でも検出したら、HTML を1バイトも出さない。**

    検査そのものは `validate_input` が保持する ── 組み立てと検査を同じ場所に置くと、
    不合格の判明が生成物のあとになる。
    """
    bad = _vi.fields(spec) + _vi.shape(spec)
    for place, cell in _vi._cells(spec):
        bad += _vi.prose(place, cell) + _vi.level_mix(place, cell)
    if folder is not None:
        bad += _vi.refs(folder, spec, root)
    if bad:
        raise SystemExit("入力の検査が通っていない ── HTML は書き出さない:\n  "
                         + "\n  ".join("× " + e for e in bad))


def new(folder: pathlib.Path, title: str) -> None:
    """雛形から記録のフォルダを作る。**同じ名前が在れば作らない。**"""
    if folder.exists():
        raise SystemExit(f"既に在る: {folder}")
    tpl = REFERENCES / "spec-template.json"
    spec = json.loads(tpl.read_text(encoding="utf-8"))
    spec["title"] = title
    spec["no"] = f"ACDR {folder.name.split('-')[0]}"
    spec["date"] = _dt.date.today().isoformat()
    folder.mkdir(parents=True)
    (folder / "acdr.json").write_text(
        json.dumps(spec, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(f"作った: {folder / 'acdr.json'}")
    print("  図を置くなら flow.svg を隣に置く（描くのは design-svg である）")
    print(f"  組む: python3 scripts/cli.py render {folder}")


def seal(folder: pathlib.Path, spec: dict) -> dict[str, str]:
    """対象の文書の sha256 を取る。**承認済みの記録は、承認時点の姿を保持する。**"""
    out = {}
    for d in spec.get("docs", []):
        raw = pathlib.Path(d["file"]).read_bytes()
        out[d["key"]] = hashlib.sha256(raw).hexdigest()
    return out


def drifted(folder: pathlib.Path, raw: dict) -> list[str]:
    """封印のあとに、対象の文書が変化した面を返す。

    **封印の判定を、組み立てより前に置く** ── 承認済みの記録は過去の姿であり、
    対象が移動・消滅していることが在る。先に組もうとすると、そこで停止して
    「封印されている」という結論にすら到達しない。
    """
    sealed = raw.get("seal") or {}
    if not sealed:
        return []
    root = repo_root(folder)
    moved = []
    for d in raw.get("docs", []):
        path = root / d["file"]
        now = (hashlib.sha256(path.read_bytes()).hexdigest() if path.exists() else None)
        if sealed.get(d["key"]) != now:
            moved.append(d["key"])
    return moved


def build_record(folder: pathlib.Path, *, check_only: bool = False,
                 force: bool = False) -> int:
    """記録を1枚へ組む。**argv を解釈しない** ── 入口は `cli.py` の1つだけである。"""
    dest = folder / "index.html"
    raw = json.loads((folder / "acdr.json").read_text(encoding="utf-8"))
    sealed = raw.get("seal")
    moved = drifted(folder, raw)

    if moved and not force:
        if check_only:
            print("  承認時点の姿である  対象の文書が後に変化した: "
                  + " ・ ".join(moved), file=sys.stderr)
            return 0
        print("  組み直しを拒否する  承認済みの記録で、対象の文書が後に変化している: "
              + " ・ ".join(moved), file=sys.stderr)
        print("  組み直すと承認時点の姿が失われる。意図する場合は --force を渡す",
              file=sys.stderr)
        return 1

    spec = load(folder)
    out, total = build(spec)
    for name, good in check(out, spec, total):
        print(("  OK  " if good else "  NG  ") + name, file=sys.stderr)

    if check_only:
        same = dest.exists() and dest.read_text(encoding="utf-8") == out
        print(("  同一  " if same else "  差が在る  ") + str(dest), file=sys.stderr)
        return 0 if same else 1

    dest.write_text(out, encoding="utf-8")
    if raw.get("status") == "accepted" and not sealed:
        raw["seal"] = seal(folder, spec)
        (folder / "acdr.json").write_text(
            json.dumps(raw, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
        print("  封印した  対象の文書の sha256 を記録へ保存した", file=sys.stderr)
    print(f"  印 {total} 件 / {len(spec.get('docs', []))} 面 / {len(out)} 字", file=sys.stderr)
    return 0
