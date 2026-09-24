# -*- coding: utf-8 -*-
"""ゲート1の振る舞いを、事例で検証する。

**0件にできる検査だけを置いている。**
だから検証するのは、出ることと、**出てはいけない場所で出ないこと**の両方である。

  python3 test_gate.py
"""
import os
import tempfile
import unittest

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from lib import gate  # noqa: E402


def doc(text):
    d = tempfile.mkdtemp()
    p = os.path.join(d, "a.md")
    with open(p, "w", encoding="utf-8") as f:
        f.write(text)
    return p


def names(text, check=None):
    fs = gate.inspect(doc(text))
    return [x for x in fs if check is None or x.check == check]


class TestHeadingLevels(unittest.TestCase):
    """TestHeadingLevels。"""
    C = "見出しの階層が飛んでいる"

    def test_skipped_level_is_reported(self):
        """飛んだら出る。"""
        self.assertTrue(names("# あ\n\n#### い\n", self.C))

    def test_stepwise_deepening_is_not_reported(self):
        """順に深くなれば出ない。"""
        self.assertFalse(names("# あ\n\n## い\n\n### う\n", self.C))

    def test_going_shallower_is_not_reported(self):
        """浅くなるのは出ない。"""
        self.assertFalse(names("# あ\n\n## い\n\n### う\n\n## え\n", self.C))


class TestMixedStyle(unittest.TestCase):
    """TestMixedStyle。"""
    C = "文体が混ざっている"

    def test_mixed_style_is_reported(self):
        """混ざれば出る。"""
        self.assertTrue(names("本文である。\n\nこれを書きます。\n", self.C))

    def test_plain_style_only_is_not_reported(self):
        """常体だけなら出ない。"""
        self.assertFalse(names("本文である。\n\nこれを書く。\n", self.C))

    def test_polite_style_only_is_not_reported(self):
        """敬体だけなら出ない。"""
        self.assertFalse(names("本文です。\n\nこれを書きます。\n", self.C))

    def test_noun_ending_is_not_counted(self):
        """体言止めは、常体とも敬体とも決められない。"""
        self.assertFalse(names("本文です。\n\n調査結果の記録。\n", self.C))


class TestListItemEndings(unittest.TestCase):
    """TestListItemEndings。"""
    C = "並んだ項目の語尾が統一されていない"

    def test_polite_and_plain_mixed_is_reported(self):
        """敬体と非敬体が混ざれば出る。"""
        self.assertTrue(names("- 項目を書く\n- 項目を書きます\n", self.C))

    def test_period_presence_is_not_reported(self):
        """箇条書きは句点を省く。省いたことを揺れと数えない。"""
        self.assertFalse(names("- 項目を書く\n- 項目を書く。\n", self.C))

    def test_noun_and_plain_together_is_not_reported(self):
        """品詞を判定しないと分けられないものを、分けたことにしない。"""
        self.assertFalse(names("- 調査結果の記録\n- 記録を残す\n", self.C))

    def test_single_item_is_not_reported(self):
        """項目が1つなら出ない。"""
        self.assertFalse(names("- 項目を書きます\n", self.C))

    def test_different_indent_forms_another_group(self):
        """字下げが違えば別の群として見る。"""
        self.assertFalse(names("- 親を書く\n  - 子を書きます\n", self.C))


class TestAsciiDiagrams(unittest.TestCase):
    """TestAsciiDiagrams。"""
    C = "文字で図や表を描いている"

    def test_corner_characters_are_reported(self):
        """角があれば出る。"""
        self.assertTrue(names("```\n┌──┐\n```\n", self.C))

    def test_em_dash_is_not_reported(self):
        """二倍ダッシュは出ない。"""
        self.assertFalse(names("名前と理由を残す ── 消すと気づけない。\n", self.C))

    def test_tree_characters_are_not_reported(self):
        """木構造は出ない。"""
        self.assertFalse(names("```\nsrc/\n├── a.py\n└── b.py\n```\n", self.C))

    def test_four_or_more_rules_are_reported(self):
        """横罫が4つ以上続けば出る。"""
        self.assertTrue(names("区切り ──── である。\n", self.C))


class TestSynonyms(unittest.TestCase):
    """TestSynonyms。"""
    C = "同じ意味の語が2つある"

    def test_given_pair_is_reported(self):
        """対を与えれば出る。"""
        gate.SYNONYM_PAIRS = [("性質", "特徴")]
        try:
            self.assertTrue(names("性質を確認する。特徴を確認する。\n", self.C))
        finally:
            gate.SYNONYM_PAIRS = []

    def test_without_pairs_nothing_is_reported(self):
        """対を与えなければ出ない。"""
        self.assertFalse(names("性質を確認する。特徴を確認する。\n", self.C))


class TestUnrenderedEmphasis(unittest.TestCase):
    """TestUnrenderedEmphasis。"""
    C = "強調が描画されない"

    def test_reported_in_list_items(self):
        """箇条書きでも出る。"""
        self.assertTrue(names("- **壊れた強調。**続き\n", self.C))

    def test_reported_in_headings(self):
        """見出しでも出る。"""
        self.assertTrue(names("# **壊れた強調。**続き\n", self.C))

    def test_not_reported_inside_code(self):
        """コードの中では出ない。"""
        self.assertFalse(names("```\n**壊れた強調。**続き\n```\n", self.C))

    def test_period_outside_is_not_reported(self):
        """句点を外へ出せば出ない。"""
        self.assertFalse(names("- **壊れていない強調**。続き\n", self.C))


class TestQuotedStyle(unittest.TestCase):
    """TestQuotedStyle。"""
    C = "文体が混ざっている"

    def test_inside_brackets_is_not_counted(self):
        """鉤括弧の中は数えない。"""
        self.assertFalse(names("# あ\n\n常体で書く。常体で続ける。\n\n「これは引用です。」\n", self.C))

    def test_brackets_spanning_sentences_are_not_counted(self):
        """鉤括弧が文をまたいでも数えない。"""
        self.assertFalse(names(
            "# あ\n\n常体で書く。常体で続ける。\n\n"
            "「引用の一文目です。二文目もです。三文目もです」\n", self.C))

    def test_outside_brackets_is_counted(self):
        """鉤括弧の外は数える。"""
        self.assertTrue(names("# あ\n\n常体で書く。常体で続ける。\n\nこれは敬体です。\n", self.C))

    def test_polite_style_in_quotes_is_not_counted(self):
        """引用の敬体は数えない。"""
        self.assertFalse(names("# あ\n\n常体で書く。常体で続ける。\n\n> 引用です。これも引用です。\n", self.C))


class TestCheckDiscipline(unittest.TestCase):
    """TestCheckDiscipline。"""
    def test_every_check_declares_its_ground(self):
        """すべての検査が拠って立つものを持つ。"""
        for c in gate.all_checks():
            self.assertTrue(c.basis, f"{c.name} に拠って立つものが無い")

    def test_check_without_a_concept_says_so(self):
        """概念に拠らない検査は、そう名乗る。"""
        others = [c for c in gate.all_checks() if not c.basis.startswith("概念")]
        self.assertEqual([c.basis for c in others], ["媒体の決め"])

    def test_checks_can_be_listed(self):
        """**数ではなく名前で照合する。** 数で照合すると、検査を足した時点で落ちる ──
        実際に、廃語の検査を足した時点から落ちたまま気づかなかった。"""
        self.assertEqual([c.name for c in gate.all_checks()], [
            "見出しの階層が飛んでいる",
            "文体が混ざっている",
            "並んだ項目の語尾が統一されていない",
            "文字で図や表を描いている",
            "同じ意味の語が2つある",
            "述部が和語である",
            "廃語を使用している",
            "強調が描画されない",
        ])

    def test_marked_document_is_skipped(self):
        """印のある文書は検査しない。"""
        self.assertEqual(names("<!-- doc-writing-skills: exempt -->\n\n# あ\n\n#### い\n"), [])



class TestWordPairsAndQuotes(unittest.TestCase):
    """TestWordPairsAndQuotes。"""
    C = "同じ意味の語が2つある"

    def setUp(self):
        gate.SYNONYM_PAIRS[:] = [("本測定", "main")]

    def tearDown(self):
        gate.SYNONYM_PAIRS[:] = []

    def test_used_in_prose_is_reported(self):
        """地の文で使えば出る。"""
        self.assertTrue(names("本測定を流す。main の側で決める。", self.C))

    def test_inline_code_is_verbatim_and_not_reported(self):
        """識別子と原文の引用は、こちらの言葉づかいではない。"""
        self.assertFalse(
            names("本測定を流す。原文は `the main agent has finished` である。", self.C))


if __name__ == "__main__":
    unittest.main(verbosity=2)
