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


class TestCatalogMatchesRegistry:
    """目録は台帳と一致する。"""
    def test_every_registered_component_is_listed(self, cat):
        """台帳の部品が全部載っている。"""
        assert sorted(cat["parts"]) == known_kinds()

    def test_every_component_has_an_example(self):
        """台帳の全部品に見本がある。"""
        # 足りないと、その部品は一度も描かれないまま公開される
        assert sorted(EXAMPLES) == known_kinds()

    def test_unknown_component_is_refused_by_name(self):
        """台帳に無い部品を引いたら名前を挙げて断る。"""
        with pytest.raises(KeyError) as e:
            props_of("知らない部品")
        assert "box" in str(e.value)


class TestExamplesStayWithinCatalog:
    """見本だけは手で書くので、ここがずれの入口になる。"""

    @pytest.mark.parametrize("kind", known_kinds())
    def test_examples_pass_no_key_outside_catalog(self, kind, cat):
        """見本が目録に無いキーを渡していない。"""
        entry = cat["parts"][kind]
        declared = set(entry["props"])
        if str(entry.get("forwards_to", "")).startswith("props:"):
            # 渡し先が値で決まる部品は、先が定まらないのでキーも定まらない
            pytest.skip(f"{kind} は渡し先が値で決まる")
        # label はどの部品にも渡せる共通のキーで、読まない部品もある
        given = set(EXAMPLES[kind]) - {"label"}
        assert given <= declared, f"{kind}: 目録に無いキー {sorted(given - declared)}"

    @pytest.mark.parametrize("kind", known_kinds())
    def test_examples_carry_every_required_key(self, kind, cat):
        """必須のキーが見本にそろっている。"""
        need = {k for k, v in cat["parts"][kind]["props"].items() if v["required"]}
        assert need <= set(EXAMPLES[kind]), f"{kind}: 見本に足りない {sorted(need - set(EXAMPLES[kind]))}"


class TestCatalogIsUsable:
    """目録は使える形で出る。"""
    def test_serializes_to_json(self, cat):
        """JSONにできる。"""
        # 利用側は言語を問わないので、文字列へ落とせなければ公開できていない
        json.loads(json.dumps(cat, ensure_ascii=False, default=str))

    def test_declaration_required_keys_are_listed(self, cat):
        """宣言の必須のキーが載っている。"""
        d = cat["declaration"]
        assert d["nodes"]["id"]["required"]
        assert d["edges"]["from"]["required"] and d["edges"]["to"]["required"]
        # 囲みの members は合成では読まれず、群を集約する側で読まれる。
        # 入口の関数だけを走査すると欠落するキーなので、名指しで縛る
        assert d["groups"]["members"]["required"]

    def test_pass_through_component_declares_its_target(self, cat):
        """素通しする部品は渡し先を公開している。"""
        # 自分では読まないキーを受け取れる部品は、渡し先を書かないと使えない
        assert cat["parts"]["pie"]["forwards_to"] == "donut"
        assert cat["parts"]["titled"]["forwards_to"] == "props:of"
        assert "centre" in cat["parts"]["pie"]["props"]
        assert "centre" not in cat["parts"]["pie"]["reads_itself"]

    def test_only_two_placement_kinds_exist(self, cat):
        """置き方は2系統しかない。"""
        kinds = {p["placement"] for p in cat["parts"].values()}
        assert kinds <= {"own-origin", "absolute"}

    def test_bounded_token_publishes_its_bounds(self, cat):
        """範囲を持つトークンには範囲が載っている。"""
        assert cat["tokens"]["font.size"]["range"] == [8.0, 40.0]

    def test_roles_can_be_listed(self, cat):
        """役割の一覧が引ける。"""
        assert cat["roles"]["focus"]["color.box-stroke"] == "color.accent"


class TestCatalogHasNoClaimVocabulary:
    """「何を表せるか」は利用側の持ち物。目録が答えるのは「何を受け取れるか」だけ。"""

    def test_no_claim_vocabulary_leaks_in(self, cat):
        """主張の語彙が混ざっていない。"""
        text = json.dumps(cat, ensure_ascii=False, default=str)
        for word in ("主張", "asserts", "Document", "Schema", "読み方"):
            assert word not in text, f"目録に利用側の語彙が混ざっている: {word}"
