# -*- coding: utf-8 -*-
"""source.py の振る舞いを、事例で確かめる。

**この道具の売りは「静かに壊れないこと」である。**
だから確かめるのは、当たることだけではない ──
**当ててはいけないものに当たらないこと**と、
**読めなかったことが結果に出ること**を、同じ重さで確かめる。

  python3 test_source.py
"""
import os
import shutil
import tempfile
import unittest

import source


def doc(text):
    """1つの原文を持つ一時フォルダを作り、その道を返す。"""
    d = tempfile.mkdtemp()
    with open(os.path.join(d, "a.md"), "w", encoding="utf-8") as f:
        f.write(text)
    return d


class 識別子として当てる(unittest.TestCase):
    """語の文字 ＝ 英数と _ ＋ 当てる語自身が含む区切り文字。"""

    def 当たるか(self, needle, text):
        s = source.scan(doc(text), [needle], how="identifier")
        return bool(s.results[0].hits)

    def test_同じ名前は当たる(self):
        self.assertTrue(self.当たるか("tool_use_id", "field: tool_use_id"))

    def test_切り詰めた名前は当たらない(self):
        self.assertFalse(self.当たるか("tool_use", "field: tool_use_id"))

    def test_接頭辞の付いた名前は当たらない(self):
        self.assertFalse(self.当たるか("PostToolUse", "pre_PostToolUse"))

    def test_日本語の文の中でも当たる(self):
        self.assertTrue(self.当たるか("compact_summary", "項目名 compact_summary は圧縮の要約である"))

    def test_英文の文末でも当たる(self):
        self.assertTrue(self.当たるか("compact_summary", "the hook returns compact_summary."))

    def test_ドット区切りの鍵は丸ごとなら当たる(self):
        self.assertTrue(self.当たるか("github.copilot.chat.otel.enabled",
                                      "鍵は github.copilot.chat.otel.enabled である"))

    def test_ドット区切りの切れ端は当たらない(self):
        self.assertFalse(self.当たるか("otel.enabled", "github.copilot.chat.otel.enabled"))

    def test_ハイフンの名前は当たる(self):
        self.assertTrue(self.当たるか("user-agent", "the user-agent header"))

    def test_ハイフンの名前の切れ端は当たらない(self):
        self.assertFalse(self.当たるか("agent", "the user-agent header"))

    def test_大文字小文字が違えば当たらない(self):
        self.assertFalse(self.当たるか("compact_Summary", "項目名 compact_summary"))

    def test_記号に囲まれていても当たる(self):
        self.assertTrue(self.当たるか("compact_summary", "`compact_summary` を引く"))


class 引用として当てる(unittest.TestCase):
    """原文と1文字も違わず、続けて在ることを見る。空白と改行だけ揃える。"""

    def 当たるか(self, needle, text):
        s = source.scan(doc(text), [needle], how="quote")
        return bool(s.results[0].hits)

    def test_原文どおりの引用は当たる(self):
        self.assertTrue(self.当たるか("圧縮の起こし方", "| trigger | 圧縮の起こし方 |"))

    def test_語を落とした引用は当たらない(self):
        self.assertFalse(self.当たるか("その場合に出る", "その場合にのみ出る"))

    def test_語を足した引用は当たらない(self):
        self.assertFalse(self.当たるか("その場合にのみ出る", "その場合に出る"))

    def test_空白の数が違うだけなら当たる(self):
        self.assertTrue(self.当たるか("trigger は manual", "trigger　は    manual である"))

    def test_日本語が行で折り返されていても当たる(self):
        """**日本語は、行の折り返しに空白を持たない。**畳んだ先に空白を作らない。"""
        self.assertTrue(self.当たるか("圧縮の起こし方", "見出し\n圧縮の\n起こし方\n次の行"))

    def test_畳み方は原文と引用の両方に同じ規則で当たる(self):
        """引用側に空白が在っても無くても、同じに畳まれるので一致する。"""
        self.assertTrue(self.当たるか("圧縮の 起こし方", "見出し\n圧縮の\n起こし方\n次の行"))

    def test_英語が行で折り返されていれば空白1つになる(self):
        self.assertTrue(self.当たるか("the compact summary", "see\nthe compact\nsummary here"))

    def test_日本語の間の空白は無いものとして扱う(self):
        """原文に空白が在っても、日本語どうしの間なら畳んで無にする。"""
        self.assertTrue(self.当たるか("圧縮の起こし方", "…圧縮の 起こし方…"))

    def test_英語の語の間の空白は消さない(self):
        """畳むのは空白の連なりであって、空白そのものではない。"""
        self.assertFalse(self.当たるか("compact summary", "the compactsummary here"))

    def test_日本語と英数字の間の空白は残る(self):
        self.assertTrue(self.当たるか("圧縮の trigger", "…圧縮の\ntrigger…"))

    def test_切れ端でも_続けて在れば当たる(self):
        self.assertTrue(self.当たるか("起こし方", "圧縮の起こし方である"))


class 部分一致として当てる(unittest.TestCase):
    """探索のための種類。名前を付けて、明示して選ぶ。"""

    def test_切り詰めた名前でも当たる(self):
        s = source.scan(doc("field: tool_use_id"), ["tool_use"], how="text")
        self.assertTrue(s.results[0].hits)

    def test_どの種類で当てたかが結果に残る(self):
        s = source.scan(doc("field: tool_use_id"), ["tool_use"], how="text")
        self.assertEqual(s.results[0].how, "text")


class 種類を渡さなければ止まる(unittest.TestCase):
    def test_種類が無ければ例外(self):
        with self.assertRaises(ValueError):
            source.scan(doc("x"), ["x"], how="")

    def test_知らない種類なら例外(self):
        with self.assertRaises(ValueError):
            source.scan(doc("x"), ["x"], how="fuzzy")


class 位置を保つ(unittest.TestCase):
    """連結しない。どのファイルの何行目かを言えるようにする。"""

    def test_ファイルと行番号が出る(self):
        d = doc("1行目\n2行目\ncompact_summary\n4行目")
        s = source.scan(d, ["compact_summary"], how="identifier")
        hit = s.results[0].hits[0]
        self.assertEqual(hit.doc, "a.md")
        self.assertEqual(hit.line, 3)

    def test_同じ行に2回あれば2件出る(self):
        s = source.scan(doc("a_b と a_b"), ["a_b"], how="identifier")
        self.assertEqual(len(s.results[0].hits), 2)


class アンカーとの近さ(unittest.TestCase):
    """名前が在ることと、その名前がそこで使われることは別である。"""

    def 二つのファイル(self):
        d = tempfile.mkdtemp()
        with open(os.path.join(d, "a.md"), "w", encoding="utf-8") as f:
            f.write("PostCompact input\n")
        with open(os.path.join(d, "b.md"), "w", encoding="utf-8") as f:
            f.write("x\n" * 5 + "trigger\n")
        return d

    def test_同じファイルで近ければ当たる(self):
        d = doc("PostCompact input\ntrigger\n")
        s = source.scan(d, ["trigger"], how="identifier", near="PostCompact input", within=25)
        self.assertTrue(s.results[0].hits)

    def test_別のファイルのアンカーには近いと言わない(self):
        s = source.scan(self.二つのファイル(), ["trigger"], how="identifier",
                        near="PostCompact input", within=25)
        self.assertFalse(s.results[0].hits)

    def test_同じファイルでも離れていれば当たらない(self):
        d = doc("PostCompact input\n" + "x\n" * 50 + "trigger\n")
        s = source.scan(d, ["trigger"], how="identifier", near="PostCompact input", within=10)
        self.assertFalse(s.results[0].hits)


class 読めなかったものを黙って落とさない(unittest.TestCase):
    """0件は「無い」ではなく「読めた範囲には無い」である。"""

    def test_読めないファイルは読めなかったとして出る(self):
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

    def test_全部読めたときだけ_無いと結論できる(self):
        s = source.scan(doc("ほかの内容"), ["compact_summary"], how="identifier")
        self.assertTrue(s.can_conclude_absent)

    def test_大きすぎるファイルも読めなかったとして出る(self):
        d = tempfile.mkdtemp()
        with open(os.path.join(d, "big.md"), "w", encoding="utf-8") as f:
            f.write("x" * 200)
        s = source.scan(d, ["x"], how="text", max_bytes=100)
        self.assertEqual(len(s.unreadable), 1)
        self.assertFalse(s.can_conclude_absent)


class 落とすときの判定(unittest.TestCase):
    """HTML しか返らない頁も、原文として残す。"""

    def test_マークダウンは受け取る(self):
        self.assertTrue(source.acceptable("200", "text/markdown", 120))

    def test_HTMLも受け取る(self):
        self.assertTrue(source.acceptable("200", "text/html; charset=utf-8", 120))

    def test_404は受け取らない(self):
        self.assertFalse(source.acceptable("404", "text/html", 120))

    def test_中身が空なら受け取らない(self):
        self.assertFalse(source.acceptable("200", "text/markdown", 0))


if __name__ == "__main__":
    unittest.main(verbosity=2)
