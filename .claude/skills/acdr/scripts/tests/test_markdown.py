# -*- coding: utf-8 -*-
"""面の描画（lib/markdown.py）の振る舞いを、事例で検証する。

**この道具の壊れ方は、出来上がった面を開くまで見えない。**
だから検証するのは、印が付くことだけではない ──
**印が、別の印の属性の中へ入らないこと**を、同じ重さで検証する。

  python3 tests/test_markdown.py
"""
import io
import re
import sys
import unittest
from contextlib import redirect_stderr

import pathlib

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from lib import markdown as render  # noqa: E402


def mark(h, marks):
    """標準エラーへの報告を破棄して、印を付けた HTML を返す。"""
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


class TestMarking(unittest.TestCase):
    """TestMarking。"""
    def test_marks_the_found_word(self):
        """見つけた語に付く。"""
        out = mark("<p>あいうえお</p>", [{"find": "いう", "before": "旧", "why": "理由"}])
        self.assertIn('data-b="旧"', out)
        self.assertIn(">いう</mark>", out)

    def test_body_text_is_unchanged(self):
        """本文は変わらない。"""
        out = mark("<p>あいうえお</p>", [{"find": "いう", "before": "旧", "why": "理由"}])
        self.assertEqual(re.sub(r"<[^>]+>", "", out), "あいうえお")

    def test_reports_unmatched_words(self):
        """一致しない語は報告する。"""
        self.assertIn("一致せず", log("<p>あ</p>", [{"find": "無い語", "before": "x", "why": "y"}]))

    def test_word_only_inside_code_is_not_marked(self):
        """コードの中にしかない語には付かない。"""
        h = "<pre>いう</pre><p>あお</p>"
        out = mark(h, [{"find": "いう", "before": "旧", "why": "理由"}])
        self.assertNotIn("<mark", out)


class TestMarksDoNotNest(unittest.TestCase):
    """**今日、実際に壊れた形である。**"""

    def test_reason_containing_next_word_stays_out_of_attributes(self):
        """理由文に次の語が含まれていても、属性の中に入らない。"""
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

    def test_before_text_containing_next_word_stays_out_of_attributes(self):
        """変更前の文に次の語が含まれていても、属性の中に入らない。"""
        h = "<p>甲と乙がある。</p>"
        ms = [
            {"find": "甲", "before": "むかしは 乙 と書いていた", "why": "理由1"},
            {"find": "乙", "before": "旧2", "why": "理由2"},
        ]
        out = mark(h, ms)
        self.assertEqual(len(attrs(out)), 2)
        self.assertNotIn('<mark class="chg" tabindex="0" role="button" aria-expanded="false" data-b="むかしは <mark', out)

    def test_mark_count_matches_report(self):
        """付いた数が報告と合う。"""
        h = "<p>甲と乙がある。</p>"
        ms = [
            {"find": "甲", "before": "乙", "why": "理由1"},
            {"find": "乙", "before": "旧2", "why": "理由2"},
        ]
        out = mark(h, ms)
        self.assertIn("2/2 件に印を付けた", log(h, ms))
        self.assertEqual(out.count('<mark class="chg"'), 2)


class TestOverlappingMarks(unittest.TestCase):
    """TestOverlappingMarks。"""
    def test_same_word_twice_is_reported_and_dropped(self):
        """同じ語を2度指したら、報告して除外する。"""
        h = "<p>あいうえお</p>"
        ms = [
            {"find": "いう", "before": "旧1", "why": "理由1"},
            {"find": "いう", "before": "旧2", "why": "理由2"},
        ]
        out = mark(h, ms)
        self.assertEqual(out.count('<mark class="chg"'), 1)
        self.assertIn("重なる", log(h, ms))

    def test_containing_mark_is_reported_and_dropped(self):
        """一方が他方を含んでいたら、報告して除外する。"""
        h = "<p>あいうえお</p>"
        ms = [
            {"find": "あいうえ", "before": "旧1", "why": "理由1"},
            {"find": "いう", "before": "旧2", "why": "理由2"},
        ]
        out = mark(h, ms)
        self.assertEqual(out.count('<mark class="chg"'), 1)
        self.assertIn("重なる", log(h, ms))


class TestOrderIndependence(unittest.TestCase):
    """TestOrderIndependence。"""
    def test_input_order_does_not_change_result(self):
        """渡す順を変えても同じ結果になる。"""
        h = "<p>甲と乙がある。</p>"
        a = {"find": "甲", "before": "旧1", "why": "理由1"}
        b = {"find": "乙", "before": "旧2", "why": "理由2"}
        self.assertEqual(mark(h, [a, b]), mark(h, [b, a]))


if __name__ == "__main__":
    unittest.main(verbosity=2)
