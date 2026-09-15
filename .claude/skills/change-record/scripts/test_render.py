# -*- coding: utf-8 -*-
"""render.py の振る舞いを、事例で確かめる。

**この道具の壊れ方は、出来上がった面を開くまで見えない。**
だから確かめるのは、印が付くことだけではない ──
**印が、別の印の属性の中へ入らないこと**を、同じ重さで確かめる。

  python3 test_render.py
"""
import io
import re
import sys
import unittest
from contextlib import redirect_stderr

import render


def mark(h, marks):
    """標準エラーへの報告を捨てて、印を付けた HTML を返す。"""
    with redirect_stderr(io.StringIO()):
        return render.mark(h, marks)


def log(h, marks):
    buf = io.StringIO()
    with redirect_stderr(buf):
        render.mark(h, marks)
    return buf.getvalue()


def attrs(h):
    """付いた印の属性を並べる。"""
    return re.findall(r'<mark class="chg"[^>]*data-b="([^"]*)"[^>]*data-w="([^"]*)"', h)


class 印を付ける(unittest.TestCase):
    def test_見つけた語に付く(self):
        out = mark("<p>あいうえお</p>", [{"find": "いう", "before": "旧", "why": "理由"}])
        self.assertIn('data-b="旧"', out)
        self.assertIn(">いう</mark>", out)

    def test_本文は変わらない(self):
        out = mark("<p>あいうえお</p>", [{"find": "いう", "before": "旧", "why": "理由"}])
        self.assertEqual(re.sub(r"<[^>]+>", "", out), "あいうえお")

    def test_当たらない語は報告する(self):
        self.assertIn("当たらず", log("<p>あ</p>", [{"find": "無い語", "before": "x", "why": "y"}]))

    def test_コードの中にしかない語には付かない(self):
        h = "<pre>いう</pre><p>あお</p>"
        out = mark(h, [{"find": "いう", "before": "旧", "why": "理由"}])
        self.assertNotIn("<mark", out)


class 印が_別の印の中へ入らない(unittest.TestCase):
    """**今日、実際に壊れた形である。**"""

    def test_理由文に次の語が含まれていても_属性の中に入らない(self):
        h = "<p>先の箇所と、後の箇所がある。</p>"
        ms = [
            {"find": "先の箇所", "before": "旧1", "why": "ここに 後の箇所 という語が入っている"},
            {"find": "後の箇所", "before": "旧2", "why": "理由2"},
        ]
        out = mark(h, ms)
        self.assertEqual(len(attrs(out)), 2)
        # 属性の中に <mark が入っていないこと
        for b, w in attrs(out):
            self.assertNotIn("<mark", b)
            self.assertNotIn("<mark", w)

    def test_変更前の文に次の語が含まれていても_属性の中に入らない(self):
        h = "<p>甲と乙がある。</p>"
        ms = [
            {"find": "甲", "before": "むかしは 乙 と書いていた", "why": "理由1"},
            {"find": "乙", "before": "旧2", "why": "理由2"},
        ]
        out = mark(h, ms)
        self.assertEqual(len(attrs(out)), 2)
        self.assertNotIn('<mark class="chg" tabindex="0" role="button" aria-expanded="false" data-b="むかしは <mark', out)

    def test_付いた数が報告と合う(self):
        h = "<p>甲と乙がある。</p>"
        ms = [
            {"find": "甲", "before": "乙", "why": "理由1"},
            {"find": "乙", "before": "旧2", "why": "理由2"},
        ]
        out = mark(h, ms)
        self.assertIn("2/2 件に印を付けた", log(h, ms))
        self.assertEqual(out.count('<mark class="chg"'), 2)


class 位置が重なる印(unittest.TestCase):
    def test_同じ語を2度指したら_報告して落とす(self):
        h = "<p>あいうえお</p>"
        ms = [
            {"find": "いう", "before": "旧1", "why": "理由1"},
            {"find": "いう", "before": "旧2", "why": "理由2"},
        ]
        out = mark(h, ms)
        self.assertEqual(out.count('<mark class="chg"'), 1)
        self.assertIn("重なる", log(h, ms))

    def test_一方が他方を含んでいたら_報告して落とす(self):
        h = "<p>あいうえお</p>"
        ms = [
            {"find": "あいうえ", "before": "旧1", "why": "理由1"},
            {"find": "いう", "before": "旧2", "why": "理由2"},
        ]
        out = mark(h, ms)
        self.assertEqual(out.count('<mark class="chg"'), 1)
        self.assertIn("重なる", log(h, ms))


class 順序に依らない(unittest.TestCase):
    def test_渡す順を変えても同じ結果になる(self):
        h = "<p>甲と乙がある。</p>"
        a = {"find": "甲", "before": "旧1", "why": "理由1"}
        b = {"find": "乙", "before": "旧2", "why": "理由2"}
        self.assertEqual(mark(h, [a, b]), mark(h, [b, a]))


if __name__ == "__main__":
    unittest.main(verbosity=2)
