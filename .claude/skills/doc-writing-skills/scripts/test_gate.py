# -*- coding: utf-8 -*-
"""ゲート1の振る舞いを、事例で検証する。

**0件にできる検査だけを置いている。**
だから検証するのは、出ることと、**出てはいけない場所で出ないこと**の両方である。

  python3 test_gate.py
"""
import os
import tempfile
import unittest

import gate


def doc(text):
    d = tempfile.mkdtemp()
    p = os.path.join(d, "a.md")
    with open(p, "w", encoding="utf-8") as f:
        f.write(text)
    return p


def names(text, check=None):
    fs = gate.inspect(doc(text))
    return [x for x in fs if check is None or x.check == check]


class 見出しの階層(unittest.TestCase):
    C = "見出しの階層が飛んでいる"

    def test_飛んだら出る(self):
        self.assertTrue(names("# あ\n\n#### い\n", self.C))

    def test_順に深くなれば出ない(self):
        self.assertFalse(names("# あ\n\n## い\n\n### う\n", self.C))

    def test_浅くなるのは出ない(self):
        self.assertFalse(names("# あ\n\n## い\n\n### う\n\n## え\n", self.C))


class 文体の混在(unittest.TestCase):
    C = "文体が混ざっている"

    def test_混ざれば出る(self):
        self.assertTrue(names("本文である。\n\nこれを書きます。\n", self.C))

    def test_常体だけなら出ない(self):
        self.assertFalse(names("本文である。\n\nこれを書く。\n", self.C))

    def test_敬体だけなら出ない(self):
        self.assertFalse(names("本文です。\n\nこれを書きます。\n", self.C))

    def test_体言止めは数えない(self):
        """体言止めは、常体とも敬体とも決められない。"""
        self.assertFalse(names("本文です。\n\n調査結果の記録。\n", self.C))


class 並んだ項目の語尾(unittest.TestCase):
    C = "並んだ項目の語尾が統一されていない"

    def test_敬体と非敬体が混ざれば出る(self):
        self.assertTrue(names("- 項目を書く\n- 項目を書きます\n", self.C))

    def test_句点の有無では出ない(self):
        """箇条書きは句点を省く。省いたことを揺れと数えない。"""
        self.assertFalse(names("- 項目を書く\n- 項目を書く。\n", self.C))

    def test_体言と常体が並んでも出ない(self):
        """品詞を判定しないと分けられないものを、分けたことにしない。"""
        self.assertFalse(names("- 調査結果の記録\n- 記録を残す\n", self.C))

    def test_項目が1つなら出ない(self):
        self.assertFalse(names("- 項目を書きます\n", self.C))

    def test_字下げが違えば別の群として見る(self):
        self.assertFalse(names("- 親を書く\n  - 子を書きます\n", self.C))


class 文字で描いた図(unittest.TestCase):
    C = "文字で図や表を描いている"

    def test_角があれば出る(self):
        self.assertTrue(names("```\n┌──┐\n```\n", self.C))

    def test_二倍ダッシュは出ない(self):
        self.assertFalse(names("名前と理由を残す ── 消すと気づけない。\n", self.C))

    def test_木構造は出ない(self):
        self.assertFalse(names("```\nsrc/\n├── a.py\n└── b.py\n```\n", self.C))

    def test_横罫が4つ以上続けば出る(self):
        self.assertTrue(names("区切り ──── である。\n", self.C))


class 同じ意味の語(unittest.TestCase):
    C = "同じ意味の語が2つある"

    def test_対を与えれば出る(self):
        gate.SYNONYM_PAIRS = [("性質", "特徴")]
        try:
            self.assertTrue(names("性質を確認する。特徴を確認する。\n", self.C))
        finally:
            gate.SYNONYM_PAIRS = []

    def test_対を与えなければ出ない(self):
        self.assertFalse(names("性質を確認する。特徴を確認する。\n", self.C))


class 描画されない強調(unittest.TestCase):
    C = "強調が描画されない"

    def test_箇条書きでも出る(self):
        self.assertTrue(names("- **壊れた強調。**続き\n", self.C))

    def test_見出しでも出る(self):
        self.assertTrue(names("# **壊れた強調。**続き\n", self.C))

    def test_コードの中では出ない(self):
        self.assertFalse(names("```\n**壊れた強調。**続き\n```\n", self.C))

    def test_句点を外へ出せば出ない(self):
        self.assertFalse(names("- **壊れていない強調**。続き\n", self.C))


class 引用の文体(unittest.TestCase):
    C = "文体が混ざっている"

    def test_鉤括弧の中は数えない(self):
        self.assertFalse(names("# あ\n\n常体で書く。常体で続ける。\n\n「これは引用です。」\n", self.C))

    def test_鉤括弧が文をまたいでも数えない(self):
        self.assertFalse(names(
            "# あ\n\n常体で書く。常体で続ける。\n\n"
            "「引用の一文目です。二文目もです。三文目もです」\n", self.C))

    def test_鉤括弧の外は数える(self):
        self.assertTrue(names("# あ\n\n常体で書く。常体で続ける。\n\nこれは敬体です。\n", self.C))

    def test_引用の敬体は数えない(self):
        self.assertFalse(names("# あ\n\n常体で書く。常体で続ける。\n\n> 引用です。これも引用です。\n", self.C))


class 検査そのものの規律(unittest.TestCase):
    def test_すべての検査が拠って立つものを持つ(self):
        for c in gate.all_checks():
            self.assertTrue(c.basis, f"{c.name} に拠って立つものが無い")

    def test_概念に拠らない検査は_そう名乗る(self):
        others = [c for c in gate.all_checks() if not c.basis.startswith("概念")]
        self.assertEqual([c.basis for c in others], ["媒体の決め"])

    def test_検査を列挙できる(self):
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

    def test_印のある文書は検査しない(self):
        self.assertEqual(names("<!-- doc-writing-skills: exempt -->\n\n# あ\n\n#### い\n"), [])



class 語の対と原文の引用(unittest.TestCase):
    C = "同じ意味の語が2つある"

    def setUp(self):
        gate.SYNONYM_PAIRS[:] = [("本測定", "main")]

    def tearDown(self):
        gate.SYNONYM_PAIRS[:] = []

    def test_地の文で使えば出る(self):
        self.assertTrue(names("本測定を流す。main の側で決める。", self.C))

    def test_インラインコードの中は原文なので出ない(self):
        """識別子と原文の引用は、こちらの言葉づかいではない。"""
        self.assertFalse(
            names("本測定を流す。原文は `the main agent has finished` である。", self.C))


if __name__ == "__main__":
    unittest.main(verbosity=2)
