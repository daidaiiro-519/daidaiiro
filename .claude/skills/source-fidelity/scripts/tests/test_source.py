# -*- coding: utf-8 -*-
"""source.py の振る舞いを、事例で検証する。

**この道具の売りは「静かに壊れないこと」である。**
だから検証するのは、当たることだけではない ──
**当ててはいけないものに当たらないこと**と、
**読めなかったことが結果に出ること**を、同じ重さで検証する。

  python3 test_source.py
"""
import os
import shutil
import tempfile
import unittest

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from lib import source  # noqa: E402


def doc(text):
    """1つの原文を持つ一時フォルダを作り、その道を返す。"""
    d = tempfile.mkdtemp()
    with open(os.path.join(d, "a.md"), "w", encoding="utf-8") as f:
        f.write(text)
    return d


class TestIdentifierMatching(unittest.TestCase):
    """語の文字 ＝ 英数と _ ＋ 照合する語自身が含む区切り文字。"""

    def hits(self, needle, text):
        """hits。"""
        s = source.scan(doc(text), [needle], how="identifier")
        return bool(s.results[0].hits)

    def test_same_name_hits(self):
        """同じ名前は当たる。"""
        self.assertTrue(self.hits("tool_use_id", "field: tool_use_id"))

    def test_truncated_name_does_not_hit(self):
        """切り詰めた名前は当たらない。"""
        self.assertFalse(self.hits("tool_use", "field: tool_use_id"))

    def test_prefixed_name_does_not_hit(self):
        """接頭辞の付いた名前は当たらない。"""
        self.assertFalse(self.hits("PostToolUse", "pre_PostToolUse"))

    def test_hits_inside_japanese_sentence(self):
        """日本語の文の中でも当たる。"""
        self.assertTrue(self.hits("compact_summary", "項目名 compact_summary は圧縮の要約である"))

    def test_hits_at_end_of_english_sentence(self):
        """英文の文末でも当たる。"""
        self.assertTrue(self.hits("compact_summary", "the hook returns compact_summary."))

    def test_dotted_key_hits_when_whole(self):
        """ドット区切りの鍵は丸ごとなら当たる。"""
        self.assertTrue(self.hits("github.copilot.chat.otel.enabled",
                                      "鍵は github.copilot.chat.otel.enabled である"))

    def test_dotted_fragment_does_not_hit(self):
        """ドット区切りの切れ端は当たらない。"""
        self.assertFalse(self.hits("otel.enabled", "github.copilot.chat.otel.enabled"))

    def test_hyphenated_name_hits(self):
        """ハイフンの名前は当たる。"""
        self.assertTrue(self.hits("user-agent", "the user-agent header"))

    def test_hyphenated_fragment_does_not_hit(self):
        """ハイフンの名前の切れ端は当たらない。"""
        self.assertFalse(self.hits("agent", "the user-agent header"))

    def test_different_case_does_not_hit(self):
        """大文字小文字が違えば当たらない。"""
        self.assertFalse(self.hits("compact_Summary", "項目名 compact_summary"))

    def test_hits_when_surrounded_by_symbols(self):
        """記号に囲まれていても当たる。"""
        self.assertTrue(self.hits("compact_summary", "`compact_summary` を参照する"))


class TestQuoteMatching(unittest.TestCase):
    """原文と1文字も違わず、続けて在ることを検査する。空白と改行だけ統一する。"""

    def hits(self, needle, text):
        """hits。"""
        s = source.scan(doc(text), [needle], how="quote")
        return bool(s.results[0].hits)

    def test_verbatim_quote_hits(self):
        """原文どおりの引用は当たる。"""
        self.assertTrue(self.hits("圧縮の起こし方", "| trigger | 圧縮の起こし方 |"))

    def test_quote_with_a_word_removed_does_not_hit(self):
        """語を除いた引用は当たらない。"""
        self.assertFalse(self.hits("その場合に出る", "その場合にのみ出る"))

    def test_quote_with_a_word_added_does_not_hit(self):
        """語を足した引用は当たらない。"""
        self.assertFalse(self.hits("その場合にのみ出る", "その場合に出る"))

    def test_differing_space_count_still_hits(self):
        """空白の数が違うだけなら当たる。"""
        self.assertTrue(self.hits("trigger は manual", "trigger　は    manual である"))

    def test_japanese_wrapped_across_lines_hits(self):
        """**日本語は、行の折り返しに空白を持たない。**圧縮した先に空白を作らない。"""
        self.assertTrue(self.hits("圧縮の起こし方", "見出し\n圧縮の\n起こし方\n次の行"))

    def test_same_space_rule_applies_to_source_and_quote(self):
        """引用側に空白が在っても無くても、同じに圧縮されるので一致する。"""
        self.assertTrue(self.hits("圧縮の 起こし方", "見出し\n圧縮の\n起こし方\n次の行"))

    def test_wrapped_english_becomes_one_space(self):
        """英語が行で折り返されていれば空白1つになる。"""
        self.assertTrue(self.hits("the compact summary", "see\nthe compact\nsummary here"))

    def test_space_between_japanese_is_ignored(self):
        """原文に空白が在っても、日本語どうしの間なら圧縮して無にする。"""
        self.assertTrue(self.hits("圧縮の起こし方", "…圧縮の 起こし方…"))

    def test_space_between_english_words_is_kept(self):
        """圧縮するのは空白の連なりであって、空白そのものではない。"""
        self.assertFalse(self.hits("compact summary", "the compactsummary here"))

    def test_space_between_japanese_and_ascii_is_kept(self):
        """日本語と英数字の間の空白は残る。"""
        self.assertTrue(self.hits("圧縮の trigger", "…圧縮の\ntrigger…"))

    def test_fragment_hits_when_contiguous(self):
        """切れ端でも、続けて在れば当たる。"""
        self.assertTrue(self.hits("起こし方", "圧縮の起こし方である"))


class TestTextMatching(unittest.TestCase):
    """探索のための種類。名前を付けて、明示して選ぶ。"""

    def test_truncated_name_hits(self):
        """切り詰めた名前でも当たる。"""
        s = source.scan(doc("field: tool_use_id"), ["tool_use"], how="text")
        self.assertTrue(s.results[0].hits)

    def test_match_kind_is_recorded(self):
        """どの種類で照合したかが結果に残る。"""
        s = source.scan(doc("field: tool_use_id"), ["tool_use"], how="text")
        self.assertEqual(s.results[0].how, "text")


class TestKindIsRequired(unittest.TestCase):
    """TestKindIsRequired。"""
    def test_missing_kind_raises(self):
        """種類が無ければ例外。"""
        with self.assertRaises(ValueError):
            source.scan(doc("x"), ["x"], how="")

    def test_unknown_kind_raises(self):
        """知らない種類なら例外。"""
        with self.assertRaises(ValueError):
            source.scan(doc("x"), ["x"], how="fuzzy")


class TestPositions(unittest.TestCase):
    """連結しない。どのファイルの何行目かを提示できるようにする。"""

    def test_file_and_line_are_reported(self):
        """ファイルと行番号が出る。"""
        d = doc("1行目\n2行目\ncompact_summary\n4行目")
        s = source.scan(d, ["compact_summary"], how="identifier")
        hit = s.results[0].hits[0]
        self.assertEqual(hit.doc, "a.md")
        self.assertEqual(hit.line, 3)

    def test_twice_on_one_line_gives_two_hits(self):
        """同じ行に2回あれば2件出る。"""
        s = source.scan(doc("a_b と a_b"), ["a_b"], how="identifier")
        self.assertEqual(len(s.results[0].hits), 2)


class TestAnchorProximity(unittest.TestCase):
    """名前が在ることと、その名前がそこで使われることは別である。"""

    def two_files(self):
        """two_files。"""
        d = tempfile.mkdtemp()
        with open(os.path.join(d, "a.md"), "w", encoding="utf-8") as f:
            f.write("PostCompact input\n")
        with open(os.path.join(d, "b.md"), "w", encoding="utf-8") as f:
            f.write("x\n" * 5 + "trigger\n")
        return d

    def test_near_in_same_file_hits(self):
        """同じファイルで近ければ当たる。"""
        d = doc("PostCompact input\ntrigger\n")
        s = source.scan(d, ["trigger"], how="identifier", near="PostCompact input", within=25)
        self.assertTrue(s.results[0].hits)

    def test_anchor_in_another_file_is_not_near(self):
        """別のファイルのアンカーには近いと言わない。"""
        s = source.scan(self.two_files(), ["trigger"], how="identifier",
                        near="PostCompact input", within=25)
        self.assertFalse(s.results[0].hits)

    def test_far_in_same_file_does_not_hit(self):
        """同じファイルでも離れていれば当たらない。"""
        d = doc("PostCompact input\n" + "x\n" * 50 + "trigger\n")
        s = source.scan(d, ["trigger"], how="identifier", near="PostCompact input", within=10)
        self.assertFalse(s.results[0].hits)


class TestUnreadableIsReported(unittest.TestCase):
    """0件は「無い」ではなく「読めた範囲には無い」である。"""

    def test_unreadable_file_is_reported_as_unread(self):
        """読めないファイルは読めなかったとして出る。"""
        d = tempfile.mkdtemp()
        p = os.path.join(d, "c.md")
        with open(p, "w", encoding="utf-8") as f:
            f.write("compact_summary\n")
        os.chmod(p, 0)
        try:
            s = source.scan(d, ["compact_summary"], how="identifier")
            self.assertEqual(len(s.unreadable), 1)
            self.assertIn("c.md", s.unreadable[0][0])
            self.assertFalse(s.results[0].hits)
            self.assertFalse(s.can_conclude_absent,
                             "読めなかった範囲が在るなら、無いと結論してはいけない")
        finally:
            os.chmod(p, 0o644)
            shutil.rmtree(d, ignore_errors=True)

    def test_absence_concluded_only_when_all_read(self):
        """全部読めたときだけ、無いと結論できる。"""
        s = source.scan(doc("ほかの内容"), ["compact_summary"], how="identifier")
        self.assertTrue(s.can_conclude_absent)

    def test_oversized_file_is_reported_as_unread(self):
        """大きすぎるファイルも読めなかったとして出る。"""
        d = tempfile.mkdtemp()
        with open(os.path.join(d, "big.md"), "w", encoding="utf-8") as f:
            f.write("x" * 200)
        s = source.scan(d, ["x"], how="text", max_bytes=100)
        self.assertEqual(len(s.unreadable), 1)
        self.assertFalse(s.can_conclude_absent)


class TestFetchVerdict(unittest.TestCase):
    """HTML しか返らない頁も、原文として残す。"""

    def test_markdown_is_accepted(self):
        """マークダウンは受け取る。"""
        self.assertTrue(source.acceptable("200", "text/markdown", 120))

    def test_html_is_accepted(self):
        """HTMLも受け取る。"""
        self.assertTrue(source.acceptable("200", "text/html; charset=utf-8", 120))

    def test_404_is_rejected(self):
        """404は受け取らない。"""
        self.assertFalse(source.acceptable("404", "text/html", 120))

    def test_empty_body_is_rejected(self):
        """中身が空なら受け取らない。"""
        self.assertFalse(source.acceptable("200", "text/markdown", 0))


if __name__ == "__main__":
    unittest.main(verbosity=2)
