# SPDX-License-Identifier: MIT
"""意思決定の記録を、節と差分を持つ1枚のHTMLへ組む。

    python3 build_acdr.py <spec.json> <出力.html>

**この道具は、変更後の中身を複製しない。** 節が持つのは決定であり、
変更そのものは対象の文書の上に印として出る（tabs.build が組む）。
対象が無い決定（新規）は、節だけの1枚になる ── 例外にせず、同じ器で空にする。
"""
from __future__ import annotations

import html as _h
import datetime as _dt
import json
import pathlib
import sys

from tabs import build as build_docs, check as check_docs

# 承認の状態。これ以外を書かせない ── 状態が自由文になると、
# 「承認されているか」を読む側が判定することになる
STATUS = {
    "proposed": ("提案", "まだ承認を得ていない。適用してはならない"),
    "accepted": ("承認", "承認を得た。適用してよい"),
    "superseded": ("差し替え済み", "後の記録が、この決定を置き換えた"),
}

# 節。**欠けたら止まる** ── 欠けた記録は、あとから誰にも補えない
SECTIONS = ["決定", "なぜ", "適用先"]

CSS = """
.acdr{border:1px solid var(--line);background:var(--panel);border-radius:.35rem;
padding:1.1rem 1.3rem 1.2rem;margin:0 0 1.8rem;max-width:68rem}
.acdr .hd{display:flex;gap:.7rem;align-items:baseline;flex-wrap:wrap;
border-bottom:1px solid var(--line);padding-bottom:.55rem;margin-bottom:1rem}
.acdr .no{font-size:.78rem;font-weight:700;letter-spacing:.1em;color:var(--move)}
.acdr h1.t{font-size:1.35rem;margin:0;letter-spacing:.02em;border:0;padding:0}
.acdr .when{margin-left:auto;font-size:.82rem;color:var(--muted)}
.acdr .st{font-size:.76rem;font-weight:700;padding:.08rem .55rem;border-radius:999px;
border:1px solid currentColor}
.acdr .st.proposed{color:var(--move)}
.acdr .st.accepted{color:var(--key)}
.acdr .st.superseded{color:var(--muted)}
.acdr .stnote{font-size:.82rem;color:var(--muted)}
.acdr .ask{font-size:.78rem;font-weight:700;letter-spacing:.1em;color:var(--move);
margin:0 0 .35rem}
.acdr .decide{font-size:1.12rem;background:var(--paper);border-left:4px solid var(--move);
padding:.65rem .95rem;margin:0 0 1.1rem}
.acdr .sec{margin:1.1rem 0 0}
.acdr .sec h4{font-size:.78rem;font-weight:700;letter-spacing:.1em;color:var(--move);
margin:0 0 .3rem;text-transform:none}
.acdr .sec p{margin:0}
.acdr .sec ul{margin:.1rem 0;padding-left:1.2rem}
.acdr .sec table{margin:.35rem 0 0}
.acdr .none{color:var(--muted)}
.acdr .lead{font-size:.86rem;color:var(--muted);margin:0 0 .3rem}
.acdr table.shift td:nth-child(3){font-weight:700}
.acdr .fig{margin:.4rem 0 0;padding:.6rem;background:var(--paper);border:1px solid var(--line);border-radius:.3rem;overflow-x:auto}
.acdr .fig svg{display:block;max-width:100%;height:auto}
.acdr .fig figcaption{font-size:.84rem;color:var(--muted);margin-top:.4rem}
.lede{display:none}
.bridge{max-width:68rem;font-size:.9rem;color:var(--muted);margin:0 0 .6rem}
"""


def _list(items: list[str]) -> str:
    if not items:
        return '<span class="none">無し</span>'
    return "<ul>" + "".join(f"<li>{v}</li>" for v in items) + "</ul>"


def _dropped(rows: list) -> str:
    """比較した案。**どれも反証を通過しなかったものである** ── 未解決の懸念ではない。"""
    if not rows:
        return '<span class="none">比較した案は無い</span>'
    body = "".join(
        f"<tr><td>{r['案']}</td><td>{r.get('採らなかった理由', r.get('何が壊れるか', ''))}</td></tr>"
        for r in rows)
    return ('<p class="lead">どれも反証を通過しなかった案である ── '
            '未解決のまま残っているものではない。</p>'
            "<table><thead><tr><th>案</th><th>採らなかった理由</th></tr></thead>"
            f"<tbody>{body}</tbody></table>")


def _shift(rows: list) -> str:
    """変更前と変更後を、抽象の側で並べる。

    **具体の差分は面が持つ。** ここが持つのは、何がどう変わるかの形である ──
    抽象の対比が無いと、下に並ぶ具体の差分が何のためかを読み手が復元することになる。
    """
    body = "".join(f"<tr><td>{r['何が']}</td><td>{r['いまの形']}</td>"
                   f"<td>{r['これからの形']}</td></tr>" for r in rows)
    return ("<table class=\"shift\"><thead><tr><th>何が</th><th>いまの形</th>"
            "<th>これからの形</th></tr></thead>"
            f"<tbody>{body}</tbody></table>")


def header(spec: dict) -> str:
    """節を組む。**欠けている節が在れば止まる。**

    題はここが持つ ── 面の見出しと二重に出さない。
    """
    missing = [k for k in SECTIONS if not str(spec.get(k, "")).strip()]
    if missing:
        raise ValueError(f"節が欠けている: {'・'.join(missing)} ── "
                         "欠けた記録は、あとから誰にも補えない")
    st = spec.get("状態", "proposed")
    if st not in STATUS:
        raise ValueError(f"状態が「{st}」。使えるのは {'／'.join(STATUS)} である")
    label, note = STATUS[st]
    num = _h.escape(str(spec.get("番号", "")))
    when = _h.escape(str(spec.get("日付", "")))

    secs = [("なぜ、いま決めるのか", f'<p>{spec["なぜ"]}</p>')]
    if spec.get("形の変化"):
        secs.append(("形の変化", _shift(spec["形の変化"])))
    if spec.get("_図"):
        secs.append(("図で確認する", f'<figure class="fig">{spec["_図"]}'
                                   f'<figcaption>{spec.get("図の説明", "")}</figcaption></figure>'))
    if spec.get("実現の形"):
        secs.append(("実現の形", f'<p>{spec["実現の形"]}</p>'))
    secs += [("適用先", f'<p>{spec["適用先"]}</p>'),
             ("比較した案", _dropped(spec.get("比較した案", spec.get("除外した案", [])))),
             ("承認後に実施すること", _list(spec.get("承認後に実施すること",
                                                spec.get("負担すること", []))))]
    if spec.get("置き換えた記録"):
        secs.append(("置き換えた記録", f'<p>{spec["置き換えた記録"]}</p>'))
    body = "".join(f'<div class="sec"><h4>{k}</h4>{v}</div>' for k, v in secs)

    chip = '<span class="no">' + num + '</span>' if num else ''
    return (f'<div class="acdr"><div class="hd">{chip}'
            f'<h1 class="t">{_h.escape(spec["題"])}</h1>'
            f'<span class="st {st}">{label}</span>'
            f'<span class="stnote">{note}</span>'
            f'<span class="when">{when}</span></div>'
            f'<div class="ask">決めること ── 承認か差し戻しを返すのは、この1件である</div>'
            f'<div class="decide">{spec["決定"]}</div>{body}</div>')


def build(spec: dict) -> tuple[str, int]:
    docs = spec.get("docs", [])
    head = header(spec)
    if not docs:
        # 新規の決定。差分が無いので、節だけの1枚になる
        from tabs import CSS as BASE
        return ('<meta charset="utf-8">'
                '<meta name="viewport" content="width=device-width,initial-scale=1">'
                f'<title>{_h.escape(spec["題"])}</title>'
                f"<style>{BASE}{CSS}</style>"
                f'<div class="wrap">{head}'
                f'<p class="none">対象の文書に変更が無い決定である ── 差分は出ない。</p>'
                f"</div>", 0)

    inner = dict(spec)
    inner["title"] = spec["題"]
    out, total = build_docs(inner)
    # tabs.build が組んだ1枚の、見出しの直後へ節を差し込む
    anchor = f'<h1>{_h.escape(spec["題"])}</h1>'
    if anchor not in out:
        raise ValueError("差し込む位置が見つからない ── tabs.build の出力が変わっている")
    out = out.replace("</style>\n<style id=\"markcss-live\">",
                      f"{CSS}</style>\n<style id=\"markcss-live\">", 1)
    # 題は節が持つ。面の見出しと二重に出さない。
    # 節と面のあいだに橋を1行置く ── 下に並ぶのは、この決定を採ったときの具体である
    bridge = ('<p class="bridge">下に並ぶのは、この決定を採ったときに'
              '<b>何がどう変わるか</b>である ── 上の決定の具体である。</p>')
    return out.replace(anchor, head + bridge, 1), total


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
    if "適用先" not in spec and "対象" in spec:   # 旧い鍵を受け付ける
        spec["適用先"] = spec.pop("対象")
    validate(spec)
    fig = spec.get("図ファイル")
    if fig:
        path = folder / fig
        if not path.exists():
            raise SystemExit(f"図が無い: {path} ── design-svg に組ませて置くか、"
                             '"図ファイル" の欄を削除する')
        spec["_図"] = path.read_text(encoding="utf-8")
    root = repo_root(folder)
    for d in spec.get("docs", []):
        d["file"] = str(root / d["file"])
    return spec


def validate(spec: dict) -> None:
    """形を検査する。**欠けていれば止まる** ── 欠けた記録は、あとから誰にも補えない。"""
    for k in ("番号", "題", "日付", "状態", *SECTIONS):
        if not str(spec.get(k, "")).strip():
            raise ValueError(f"必須の欄が欠けている: {k}")
    if spec["状態"] not in STATUS:
        raise ValueError(f"状態が「{spec['状態']}」。使えるのは {'／'.join(STATUS)} である")
    for i, r in enumerate(spec.get("形の変化", [])):
        for k in ("何が", "いまの形", "これからの形"):
            if k not in r:
                raise ValueError(f"形の変化[{i}] に {k} が無い")
    for i, r in enumerate(spec.get("比較した案", [])):
        for k in ("案", "採らなかった理由"):
            if k not in r:
                raise ValueError(f"比較した案[{i}] に {k} が無い")
    for d in spec.get("docs", []):
        for k in ("key", "tab", "file"):
            if k not in d:
                raise ValueError(f"docs の項目に {k} が無い")
        for i, m in enumerate(d.get("marks", [])):
            for k in ("find", "before", "why"):
                if not str(m.get(k, "")).strip():
                    raise ValueError(f"{d['key']} の変更[{i}] に {k} が無い ── "
                                     "3つ組が完備していない変更は、記録に含めない")


def new(folder: pathlib.Path, title: str) -> None:
    """雛形から記録のフォルダを作る。**同じ名前が在れば作らない。**"""
    if folder.exists():
        raise SystemExit(f"既に在る: {folder}")
    tpl = pathlib.Path(__file__).resolve().parent.parent / "references" / "spec-template.json"
    spec = json.loads(tpl.read_text(encoding="utf-8"))
    spec["題"] = title
    spec["番号"] = f"ACDR {folder.name.split('-')[0]}"
    spec["日付"] = _dt.date.today().isoformat()
    folder.mkdir(parents=True)
    (folder / "acdr.json").write_text(
        json.dumps(spec, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(f"作った: {folder / 'acdr.json'}")
    print("  図を置くなら flow.svg を隣に置く（描くのは design-svg である）")
    print(f"  組む: python3 {pathlib.Path(__file__).name} {folder}")


def main(argv: list[str]) -> int:
    if len(argv) >= 3 and argv[1] == "new":
        new(pathlib.Path(argv[2]), argv[3] if len(argv) > 3 else "題を記入する")
        return 0

    folder = pathlib.Path(argv[1] if len(argv) > 1 else ".")
    check_only = "--check" in argv
    spec = load(folder)
    out, total = build(spec)
    for name, good in check(out, spec, total):
        print(("  OK  " if good else "  NG  ") + name, file=sys.stderr)

    dest = folder / "index.html"
    if check_only:
        same = dest.exists() and dest.read_text(encoding="utf-8") == out
        print(("  同一  " if same else "  差が在る  ") + str(dest), file=sys.stderr)
        return 0 if same else 1
    dest.write_text(out, encoding="utf-8")
    print(f"  印 {total} 件 / {len(spec.get('docs', []))} 面 / {len(out)} 字", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
