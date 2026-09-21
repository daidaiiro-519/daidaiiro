# -*- coding: utf-8 -*-
"""ゲート1を適用する。**0件にできるものだけを検査する。**

  python3 scripts/cli.py check <ファイル...>
  python3 scripts/cli.py checks
  python3 scripts/cli.py check --synonyms 用語.tsv <ファイル...>
  python3 cli.py check --retired <廃語の一覧.json> <ファイル...>

**この道具は3つの型でできている。**

  単位        文書を切ったもの。種別（本文・見出し・箇条書き・表のセル・引用・コード）と行番号を持つ
  検査        名前・拠って立つ概念・適用する単位の種別を持つ
  見つけたもの  検査の名前・概念・位置・抜粋

**検査は、概念から導けるものだけを置く。**
0件にできないもの（1文の長さ・段落の行数など）は、書き手が判定として適用する。

**1つだけ、概念から導けない検査がある。**
「書いた強調が描画されない」は媒体の決めであり、CommonMark の記法に拠る。
概念ではなく媒体に拠ることを、名簿に書いてある。
"""
from __future__ import annotations

import re
import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

KINDS = frozenset({"本文", "見出し", "箇条書き", "表のセル", "引用", "コード"})
RENDERED = KINDS - {"コード"}
PROSE = frozenset({"本文", "箇条書き", "引用"})
EXEMPT_MARK = "<!-- doc-writing-skills: exempt -->"


@dataclass
class Unit:
    kind: str
    line: int
    raw: str
    text: str
    level: int = 0      # 見出しの深さ。見出し以外は0
    indent: int = 0     # 箇条書きの字下げ


@dataclass
class Finding:
    check: str
    basis: str          # 拠って立つもの。概念の番号、または「媒体の決め」
    line: int
    excerpt: str


@dataclass
class Check:
    name: str
    basis: str
    fn: Callable
    kinds: frozenset[str] | None = None
    note: str = ""


# ── 文書を単位へ切る ────────────────────────────────────────
_HEADING = re.compile(r"^(\s*)(#{1,6})(\s|$)")
_LIST = re.compile(r"^(\s*)([-*+]|\d+[.)])(\s|$)")
_QUOTE = re.compile(r"^\s*>")
_FENCE = re.compile(r"^\s*(`{3,}|~{3,})")
_TABLE = re.compile(r"^\s*\|")
_TABLE_SEP = re.compile(r"^\s*\|?[\s:|-]+\|[\s:|-]*\|?\s*$")
_INLINE = re.compile(r"\*\*|`|\[|\]\([^)]*\)|<br\s*/?>")


def _plain(s: str) -> str:
    return _INLINE.sub("", s).strip()


def split_units(text: str) -> list[Unit]:
    """文書を、位置を保ったまま単位へ切る。"""
    units: list[Unit] = []
    fence: tuple[str, int] | None = None
    in_front = False
    for i, raw in enumerate(text.split("\n"), 1):
        if i == 1 and raw.strip() == "---":
            in_front = True
            continue
        if in_front:
            if raw.strip() == "---":
                in_front = False
            continue
        m = _FENCE.match(raw)
        if m:
            run = m.group(1)
            if fence is None:
                fence = (run[0], len(run))
            elif run[0] == fence[0] and len(run) >= fence[1]:
                fence = None
            continue
        if fence is not None:
            units.append(Unit("コード", i, raw, raw.strip()))
            continue
        if not raw.strip():
            continue
        h = _HEADING.match(raw)
        if h:
            units.append(Unit("見出し", i, raw,
                              _plain(re.sub(r"^\s*#{1,6}\s*", "", raw)), level=len(h.group(2))))
            continue
        s = raw.lstrip()
        if _QUOTE.match(s):
            units.append(Unit("引用", i, raw, _plain(re.sub(r"^\s*>\s*", "", s))))
        elif _TABLE.match(s):
            if _TABLE_SEP.match(s):
                continue
            for cell in s.strip().strip("|").split("|"):
                if cell.strip():
                    units.append(Unit("表のセル", i, raw, _plain(cell)))
        elif _LIST.match(s):
            units.append(Unit("箇条書き", i, raw, _plain(_LIST.sub("", s, count=1)),
                              indent=len(raw) - len(s)))
        else:
            units.append(Unit("本文", i, raw, _plain(raw)))
    return units


# ── 文末の型 ────────────────────────────────────────────────
KEITAI = re.compile(r"(です|ます|ません|でした|ましょう|でしょう|ください)。?$")
JOTAI = re.compile(r"(である|だ|た|ない|る|い|う|く|す|つ|ぬ|ぶ|む)。$")


def ending(text: str) -> str:
    """文末を、敬体か、そうでないかに分ける。

    **体言止めと常体は、品詞を判定しないと分けられない。**
    「扱い」は体言で「短い」は常体だが、字面は同じ形をしている。
    分けられないものを分けたことにせず、**敬体かどうかだけ**を判定する。
    """
    t = text.strip().rstrip("。")
    return "敬体" if KEITAI.search(t + "。") or KEITAI.search(t) else "非敬体"


# ── 検査 ────────────────────────────────────────────────────
def _heading_skip(units: list[Unit]) -> list[Finding]:
    """概念4。見出しの階層を飛ばすと、何がどこにあるかが掴めない。"""
    out, prev = [], 0
    for u in units:
        if u.kind != "見出し":
            continue
        if prev and u.level > prev + 1:
            out.append(Finding("見出しの階層が飛んでいる", "概念4", u.line,
                               f"見出し{prev} の次に 見出し{u.level}：{u.text[:24]}"))
        prev = u.level
    return out


def _mixed_style(units: list[Unit]) -> list[Finding]:
    """概念6。文体が混ざると、読み手は書き分けに意味があると読む。

    **引用は数えない。**引用は書き手の文体ではなく、他人の文である。
    統一しようとすれば原文を書き換えることになり、引用でなくなる。

    引用の印は2つ ── 引用として置かれた単位か、鉤括弧の中か。
    表のセルに置いた他人の言葉は前者にならないので、後者で検出する。
    鉤括弧は文をまたぐので、深さを持って追う。
    """
    kinds = Counter()
    where: dict[str, tuple[int, str]] = {}
    for u in units:
        if u.kind not in PROSE or u.kind == "引用":
            continue
        depth = 0  # 鉤括弧の深さ。引用は文をまたぐので、1文ずつでは判定できない
        for s in re.split(r"(?<=。)", u.text):
            s = s.strip()
            inside = depth > 0 or s.startswith("「")
            depth += s.count("「") - s.count("」")
            if not s.endswith("。"):
                continue
            if inside:
                continue  # 鉤括弧の中は、他人の言葉である
            e = ending(s)
            if e == "非敬体" and not JOTAI.search(s):
                continue          # 体言止めは、常体とも敬体とも決められない
            kinds[e] += 1
            where.setdefault(e, (u.line, s[:26]))
    if len(kinds) > 1:
        a, b = where["敬体"], where["非敬体"]
        return [Finding("文体が混ざっている", "概念6", min(a[0], b[0]),
                        f"敬体 {kinds['敬体']} 文（{a[0]}行「{a[1]}」）と "
                        f"常体 {kinds['非敬体']} 文（{b[0]}行「{b[1]}」）")]
    return []


def _unparallel_items(units: list[Unit]) -> list[Finding]:
    """概念6。並んだ項目の形が統一されていないと、対応が読めない。"""
    out: list[Finding] = []
    group: list[Unit] = []

    def flush(g: list[Unit]) -> None:
        if len(g) < 2:
            return
        seen = {}
        for u in g:
            seen.setdefault(ending(u.text), u)
        if len(seen) > 1:
            names = " と ".join(f"{k}（{v.line}行）" for k, v in seen.items())
            out.append(Finding("並んだ項目の語尾が統一されていない", "概念6", g[0].line,
                               f"{len(g)} 項目に {names} が混ざる"))

    prev_line = -9
    for u in units:
        if u.kind == "箇条書き" and (not group or u.line - prev_line <= 1) \
           and (not group or u.indent == group[0].indent):
            group.append(u)
        else:
            flush(group)
            group = [u] if u.kind == "箇条書き" else []
        if u.kind == "箇条書き":
            prev_line = u.line
    flush(group)
    return out


BOX_CORNER = re.compile(r"[┌┐┘┏┓┛╭╮╯┬┴┼┤]")
BOX_RULE = re.compile(r"[─━]{4,}")


def _drawn_figure(u: Unit) -> list[str]:
    """概念2。図と表は、文字で描かず、図と表の形で置く。"""
    if BOX_CORNER.search(u.raw) or BOX_RULE.search(u.raw):
        return [u.raw.strip()[:28]]
    return []


SYNONYM_PAIRS: list[tuple[str, str]] = []

# 廃語 ── 一度破棄した語と、その言い換え先。**この一覧は、このSkillの外にある。**
# 語をいつ破棄したかはプロジェクトごとに相違する。ここへ書くと、書いたプロジェクトでしか
# 使えない検査になる。既定では対象のファイルから上へたどって
# `.doc-writing/retired-words.json` を探し、無ければこの検査は走らない。
RETIRED: list[tuple[str, str, str]] = []


def _synonym(units: list[Unit]) -> list[Finding]:
    """概念7。同じ文脈では、1つの意味に1つの語だけを対応づける。

    **インラインコードの中は検査しない。**そこに在るのは識別子と原文の引用であり、
    書き手の言葉づかいではない。**言い換えれば、その鍵で参照するものが壊れる。**
    """
    body = "\n".join(_without_code(u.raw) for u in units if u.kind != "コード")
    return [Finding("同じ意味の語が2つある", "概念7", 0, f"「{a}」と「{b}」")
            for a, b in SYNONYM_PAIRS if a in body and b in body]


def _retired_word(u: Unit) -> list[str]:
    """廃語 ── 一度破棄した語を、また使用していないか。

    **和語の述部と違い、この検査は語の一覧を持たない。**廃語も、いつ破棄したかも
    プロジェクトごとに相違する。一覧が渡されなければ、何も出ない。
    """
    hits = []
    for word, to, whence in RETIRED:
        i = u.text.find(word)
        if i >= 0:
            hits.append(f"…{u.text[max(0, i - 10):i]}<{word}>"
                        f"{u.text[i + len(word):i + len(word) + 8]}… → {to}（{whence}）")
    return hits


def _without_code(text: str) -> str:
    """インラインコードを外す。**閉じの無いものは、そのまま残す。**"""
    return re.sub(r"`[^`\n]+`", " ", text)


JA_PUNCT = "。、）」・：；！？"


def _broken_emphasis(u: Unit) -> list[str]:
    """媒体の決め。閉じの ** が約物の直後にあると、CommonMark では閉じ記号にならない。"""
    parts, idx = [], 0
    while True:
        j = u.raw.find("**", idx)
        if j < 0:
            parts.append(u.raw[idx:])
            break
        parts.append(u.raw[idx:j])
        parts.append("\x00")
        idx = j + 2
    depth, hits = 0, []
    for k, v in enumerate(parts):
        if v != "\x00":
            continue
        depth ^= 1
        if depth:
            continue
        prev = parts[k - 1] if k else ""
        nxt = parts[k + 1] if k + 1 < len(parts) else ""
        if prev and prev[-1] in JA_PUNCT and nxt and nxt[0] not in JA_PUNCT and not nxt[0].isspace():
            hits.append(f"…{prev[-14:]}**{nxt[:10]}…")
    return hits


# 和語の述部と、その言い換え先。
#
# **原典は禁止していない。これはこのリポジトリの決定である。**
# 公用文作成の考え方（建議）Ⅲ－４ には、ウ と エ が対称に置かれている。
#   ウ 重厚さや正確さを高めるには、述部に漢語を用いる
#     「訓読みの動詞（和語の動詞）を漢語にすると、効果が得られることがある」
#     例）決める → 決定（する）  消える → 消失（する）
#     「ただし、分かりやすさ、親しみやすさを妨げるおそれがあることに留意する」
#   エ 分かりやすさや親しみやすさを高めるには、述部に訓読みの動詞を用いる
#     「ただし、訓読みの動詞は意味の範囲が広いため、厳密に意味を特定しなければ
#      ならないときには不向きなこともあることに留意する」
#
# 原典の強度は「効果が得られることがある」── **推奨である**。
# この検査は、それを技術文書に対して**必須**へ上げた決定である。
# 根拠は エ のただし書き ── 技術文書は厳密に意味を特定しなければならない文書である。
# 決定の記録は .acdr/0003-... が保持する。
# 原文は sources/www.bunka.go.jp_..._93651301_01.txt ── ウ は :2292、エ は :2302、
# エ のただし書きは :2307 である（取得日 2026-09-12、sha256 47ad41d2b9ed6892…）。
WAGO: list[tuple[str, str]] = [
    # 対象にしないのは、言い換える先を持たない語だけである ── 引き返す ・ 落とし穴 ・
    # 推測が当たる ・ 上回る ・ 迂回する ・ 見出し。
    # **「ふつうの日本語だから」は、対象から外す理由にしない。**
    # 意味を特定できる漢語が在るなら、そちらを書く。
    (r"揃え[るたてよ]|揃っ[てた]|揃わ",                        "統一する／一致させる"),
    (r"畳[むみめん]",                                          "折り畳む／集約する"),
    (r"捨て[るたてよ]",                                        "破棄する／除外する"),
    (r"埋め[るたてよ]|埋ま[るっ]",                              "充填する／補完する"),
    (r"見直[すしせさ](?!し)",                                  "精査する／再確認する"),
    (r"担[うっい](?!当)",                                      "分担する／担当する"),
    (r"割り当て",                                             "配分する／対応づける"),
    (r"引き受け",                                             "負担する／受託する"),
    (r"見分け",                                               "識別する"),
    (r"抱え[るたて]",                                          "内包する"),
    (r"後回し",                                               "延期"),
    (r"塞[ぐぎげ]",                                            "閉塞する／解消する"),
    (r"積み上[がげ]",                                          "蓄積する"),
    (r"見落と[すしせさ]",                                      "看過する"),
    (r"曲げ[るたて]",                                          "湾曲させる／逸脱する"),
    (r"倒れ[るた]",                                            "帰着する"),
    (r"漂[うっい]",                                            "乖離する"),
    (r"刺さ[るった]",                                          "該当する"),
    (r"(?<!意)(?<!発)(?<!助)言え[るば]|[がは]言う(?!葉)",       "判定する"),
    (r"確かめ",                                                "確認する／検証する"),
    (r"(?<!見)(?<!意)守[るりれろ](?!備)|守っ[てた]",            "遵守する／適合する"),
    (r"[をで]見[るれた](?!目|込|通|当|本)|見れば",              "確認する／判定する"),
    (r"眺め[るてた]",                                         "精査する"),
    (r"叩[くきけ]",                                    "実行する"),
    (r"手が伸び",                                      "書き換える（文ごと直す）"),
    (r"効[くきけ](?!果)|効い[てた]",                    "適用される"),
    (r"(?<!後)(?<!迂)回[すし](?![算能])|(?<!出)(?<!上)(?<!下)回っ[てた]", "委譲する"),
    (r"(?<!書き)(?<!引き)写[すしせ](?!真)|(?<!青)写し(?!真)", "複製する"),
    (r"拾[うっい]",                                    "再開する"),
    (r"(?<!み)積[むみ](?!上|み上)",                     "蓄積する"),
    (r"持ち込[むみめん]",                               "取り込む"),
    (r"持ち出[すしせ]",                                 "転用する"),
    (r"崩[すし](?!れ)|崩れ",                            "反証する／成立しなくなる"),
    (r"[をで]通[すし](?!番)",                           "実行する"),
    (r"引き直[すしせ]",                                 "定義し直す"),
    (r"[をで]引[くき](?!受|返|ず|継)",                   "参照する"),
    (r"(?<!見)落と[すし](?!穴)|(?<!着)落ち[るたてな]",    "変換する／取得する／除外する"),
    (r"(?<!割り)[をに]当て[るたよ]|判定を当",            "適用する"),
    # 2026-09-21 tails.py で洗い出した分。**語彙表に無いものは通過していた**
    (r"[をが](?:知らな|知りに行)",                        "認知しない／参照する"),
    (r"[をに](?:書き出|書きだ)[すした]",                  "出力する"),
    (r"[をは]持[たっ](?:ず|ない|ている)",                  "保持する"),
    (r"[でをは]扱[うっ]",                                "処理する"),
    (r"[をに]読み込ま?な[いい]",                          "参照しない"),
    (r"空にする",                                       "空欄とする"),
    (r"[はが]変わ[るら]な[いい]",                         "変化しない"),
    (r"[をに](?:全文|それ)を置く|場所に印を置く",           "配置する"),
    (r"[にへ]残す(?!ため)",                              "保存する"),
    (r"[がは]終わ[るっ]て?い?[るた]",                     "終了する"),
    (r"[にへ]寄せな[いい]",                              "偏向させない"),
    (r"[をに]混ぜな[いい]",                              "混在させない"),
    (r"[にへ]絞る",                                     "限定する"),
    (r"[がは]止ま[るっ]",                                "停止する"),
    (r"からやり直す",                                   "再実行する"),
    (r"[もは]補えな[いい]",                              "補完できない"),
    (r"ために使う",                                     "使用する"),
    (r"で走る",                                        "実行する"),
    (r"[をに]消さな[いい]",                              "削除しない"),
    # 「を当てる」だけを見ていたので、「当たる先」が通過していた
    (r"当た[るりら]",                                   "適用する／該当する"),
    (r"(?<!割り)当て[るたてよ]",                         "適用する"),
]
_WAGO = [(re.compile(p), t) for p, t in WAGO]


def _wago_predicate(u: Unit) -> list[str]:
    """訓読みの動詞は意味の範囲が広い（公用文 Ⅲ－４ エ のただし書き）。

    技術文書は厳密に意味を特定しなければならない文書なので、ウ を必須として適用する。
    **原典は禁止していない** ── 強度を上げたのは、このリポジトリの決定である。
    """
    hits = []
    for rx, to in _WAGO:
        m = rx.search(u.text)
        if m:
            i = m.start()
            hits.append(f"…{u.text[max(0, i - 12):i]}<{m.group(0)}>"
                        f"{u.text[m.end():m.end() + 8]}… → {to}")
    return hits


CHECKS: list[Check] = [
    Check("見出しの階層が飛んでいる", "概念4", _heading_skip, None,
          "見出し2の次に見出し4が来ると、間に何が在るはずだったかが分からない"),
    Check("文体が混ざっている", "概念6", _mixed_style, None,
          "敬体と常体を、同じ文書の中で混ぜない。体言止めは数えない"),
    Check("並んだ項目の語尾が統一されていない", "概念6", _unparallel_items, None,
          "同じ立場のものは、同じ形で書く。敬体かどうかだけを判定する"),
    Check("文字で図や表を描いている", "概念2", _drawn_figure, KINDS,
          "箱の角と、横罫の連なりを検出する。二倍ダッシュと木構造は図ではない"),
    Check("同じ意味の語が2つある", "概念7", _synonym, None,
          "--synonyms で対を与える。対を渡さなければ何も出ない"),
    Check("述部が和語である", "概念8", _wago_predicate,
          frozenset({"本文", "見出し", "箇条書き", "表のセル"}),
          "公用文 Ⅲ－４ ウ を必須へ上げたリポジトリの決定（.acdr/0003-...）。"
          "引用は検査しない ── 原文を書き換えてはならない"),
    Check("廃語を使用している", "概念7", _retired_word,
          frozenset({"本文", "見出し", "箇条書き", "表のセル"}),
          "一覧は .doc-writing/retired-words.json が持つ。渡されなければ何も出ない"),
    Check("強調が描画されない", "媒体の決め", _broken_emphasis, RENDERED,
          "概念からは導けない。CommonMark の記法に拠る"),
]


def all_checks() -> list[Check]:
    return CHECKS


def inspect(path: str | Path) -> list[Finding]:
    raw = Path(path).read_text(encoding="utf-8")
    if EXEMPT_MARK in raw[:400]:
        return []
    units = split_units(raw)
    out: list[Finding] = []
    for c in CHECKS:
        if c.kinds is None:
            out += c.fn(units)
            continue
        for u in units:
            if u.kind in c.kinds:
                out += [Finding(c.name, c.basis, u.line, ex) for ex in c.fn(u)]
    seen, uniq = set(), []
    for f in out:
        key = (f.check, f.line, f.excerpt)
        if key not in seen:
            seen.add(key)
            uniq.append(f)
    return uniq


def print_checks() -> int:
    print(f"{'検査':30}{'拠って立つもの':16}{'適用する単位'}")
    for c in CHECKS:
        kinds = "文書全体" if c.kinds is None else " ・ ".join(sorted(c.kinds))
        print(f"{c.name:30}{c.basis:16}{kinds}")
        print(f"{'':46}{c.note}")
    return 0


def _find_retired(start: Path) -> Path | None:
    """対象のファイルから上へたどり、`.doc-writing/retired-words.json` を探す。

    **見つからないことを、失敗として扱わない。**一覧を持たないプロジェクトでも
    この道具はそのまま動く ── 持ち出した先で必ず止まる作りにしない。
    """
    here = start.resolve()
    for d in [here] + list(here.parents):
        c = d / ".doc-writing" / "retired-words.json"
        if c.exists():
            return c
    return None


def _load_retired(path: Path) -> list[tuple[str, str, str]]:
    """一覧を読む。形が違えば、そのまま例外で止まる ── 黙って空にしない。"""
    import json
    d = json.loads(path.read_text(encoding="utf-8"))
    return [(x["word"], x["use_instead"], x.get("source", "")) for x in d["retired"]]


def main(argv: list[str]) -> int:
    global SYNONYM_PAIRS, RETIRED
    args = list(argv)
    retired_path: Path | None = None
    if "--retired" in args:
        i = args.index("--retired")
        retired_path = Path(args[i + 1])
        del args[i:i + 2]
    if "--synonyms" in args:
        i = args.index("--synonyms")
        p = Path(args[i + 1])
        SYNONYM_PAIRS = [tuple(x.split("\t")[:2]) for x in p.read_text(encoding="utf-8").splitlines()
                         if x.strip() and not x.startswith("#") and "\t" in x]
        del args[i:i + 2]
    if "--list" in args:
        return print_checks()
    if retired_path is None and args:
        retired_path = _find_retired(Path(args[0]))
    if retired_path and retired_path.exists():
        RETIRED = _load_retired(retired_path)
    if not args:
        print(__doc__)
        return 2
    lines, n = gate_lines(args)
    for line in lines:
        print(line)
    return 1 if n else 0


def gate_lines(paths: list[str], retired: str | None = None) -> tuple[list[str], int]:
    """報告の行と、指摘の件数。**印字する側と、機械へ返す側が、同じ文字列を使う。**

    **廃語の一覧は、ここで読む** ── 入口ごとに読む形にすると、片方の入口から
    呼んだときだけ検査が走らない（実際に、そうなっていた）。
    """
    global RETIRED
    path = Path(retired) if retired else (_find_retired(Path(paths[0])) if paths else None)
    RETIRED = _load_retired(path) if path and path.exists() else []
    out, n = [], 0
    for a in paths:
        for f in inspect(a):
            out.append(f"× {Path(a).name}:{f.line} [{f.basis}] {f.check}：{f.excerpt}")
            n += 1
    out.append(f"\nゲート1　{'通った' if n == 0 else f'通っていない（{n} 件）'}"
               f"　／　対象 {len(paths)} ファイル")
    if n:
        out.append("**0件にできるものだけを検査している。直してから出す。**")
    return out, n
