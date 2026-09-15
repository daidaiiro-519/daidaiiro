"""目録 ── 公開しているものが、実物とずれていないこと。

目録は利用側（変換器を書く人）が唯一見る面なので、ここがずれると、
外から見て正しいのに動かない、という一番たちの悪い壊れ方をする。
"""
from __future__ import annotations

import json

import pytest

from svg_engine.catalog import EXAMPLES, catalog, props_of
from svg_engine.registry import known_kinds


@pytest.fixture(scope="module")
def cat():
    return catalog()


class Test目録は台帳と一致する:
    def test_台帳の部品が全部載っている(self, cat):
        assert sorted(cat["parts"]) == known_kinds()

    def test_台帳の全部品に見本がある(self):
        # 足りないと、その部品は一度も描かれないまま公開される
        assert sorted(EXAMPLES) == known_kinds()

    def test_台帳に無い部品を引いたら名前を挙げて断る(self):
        with pytest.raises(KeyError) as e:
            props_of("知らない部品")
        assert "box" in str(e.value)


class Test見本は目録の範囲に収まる:
    """見本だけは手で書くので、ここがずれの入口になる。"""

    @pytest.mark.parametrize("kind", known_kinds())
    def test_見本が目録に無い鍵を渡していない(self, kind, cat):
        entry = cat["parts"][kind]
        declared = set(entry["props"])
        if str(entry.get("forwards_to", "")).startswith("props:"):
            # 渡し先が値で決まる部品は、先が定まらないので鍵も定まらない
            pytest.skip(f"{kind} は渡し先が値で決まる")
        # label はどの部品にも渡せる共通の鍵で、読まない部品もある
        given = set(EXAMPLES[kind]) - {"label"}
        assert given <= declared, f"{kind}: 目録に無い鍵 {sorted(given - declared)}"

    @pytest.mark.parametrize("kind", known_kinds())
    def test_必須の鍵が見本にそろっている(self, kind, cat):
        need = {k for k, v in cat["parts"][kind]["props"].items() if v["required"]}
        assert need <= set(EXAMPLES[kind]), f"{kind}: 見本に足りない {sorted(need - set(EXAMPLES[kind]))}"


class Test目録は使える形で出る:
    def test_JSONにできる(self, cat):
        # 利用側は言語を問わないので、文字列へ落とせなければ公開できていない
        json.loads(json.dumps(cat, ensure_ascii=False, default=str))

    def test_宣言の必須の鍵が載っている(self, cat):
        d = cat["declaration"]
        assert d["nodes"]["id"]["required"]
        assert d["edges"]["from"]["required"] and d["edges"]["to"]["required"]
        # 囲みの members は合成では読まれず、群を畳む側で読まれる。
        # 入口の関数だけを見ると落ちる鍵なので、名指しで縛る
        assert d["groups"]["members"]["required"]

    def test_素通しする部品は渡し先を公開している(self, cat):
        # 自分では読まない鍵を受け取れる部品は、渡し先を書かないと使えない
        assert cat["parts"]["pie"]["forwards_to"] == "donut"
        assert cat["parts"]["titled"]["forwards_to"] == "props:of"
        assert "centre" in cat["parts"]["pie"]["props"]
        assert "centre" not in cat["parts"]["pie"]["reads_itself"]

    def test_置き方は2系統しかない(self, cat):
        kinds = {p["placement"] for p in cat["parts"].values()}
        assert kinds <= {"own-origin", "absolute"}

    def test_範囲を持つトークンには範囲が載っている(self, cat):
        assert cat["tokens"]["font.size"]["range"] == [8.0, 40.0]

    def test_役割の一覧が引ける(self, cat):
        assert cat["roles"]["focus"]["color.box-stroke"] == "color.accent"


class Test目録は主張の語彙を持たない:
    """「何を表せるか」は利用側の持ち物。目録が答えるのは「何を受け取れるか」だけ。"""

    def test_主張の語彙が混ざっていない(self, cat):
        text = json.dumps(cat, ensure_ascii=False, default=str)
        for word in ("主張", "asserts", "Document", "Schema", "読み方"):
            assert word not in text, f"目録に利用側の語彙が混ざっている: {word}"
