# 取得した原文の取り直し方

**原文の実体は置かない。** 置くのは `*.meta.json` だけで、url ・ 取得日 ・ `sha256` ・
バイト数 ・ 行数が入っている。取り直して `sha256` が一致すれば、照合した当時と同じ中身である。

```
python3 <fact-check>/scripts/source.py fetch <url> --dir .
```

PDF は本文を起こしてから照合する。ブレストボードの根拠が指す行番号は、起こした `.txt` の行番号である。

```
pdftotext -layout <名前>.pdf <名前>.txt
```
# 取得した原文の取り直し方

**原文の実体は置かない。** 置くのは `*.meta.json` だけで、これに url ・ 取得日 ・ `sha256` ・
バイト数 ・ 行数が入っている。取り直して `sha256` が一致すれば、照合した当時と同じ中身である。

```
python3 ../../../.claude/skills/fact-check/scripts/source.py fetch <url> --dir .
```

`url` は各 `*.meta.json` の `url` が持つ。

**PDF は本文を起こしてから照合している。** ボードの根拠が指す行番号は、起こした
`.txt` の行番号である。同じ手順で起こす。

```
pdftotext -layout <名前>.pdf <名前>.txt
```

| 原典 | 何を支えているか |
|---|---|
| `www.rfc-editor.org_rfc_rfc9110.txt` | 論点4 ── インターフェースの原典が状態コードを規定する |
| `pubs.opengroup.org_..._V3_chap02.html` | 論点4 ── POSIX が終了コードの値を規定する |
| `folk.universitetetioslo.no_..._1979-12-MVC.pdf` | 論点6 ── MVC が設計論として原典を持ち、層も配置も規定しない |
| `folk.uio.no_..._1979-05-MVC.pdf` | 同上の前身。取得のみで、照合していない |
| `react.dev_reference_rules_rules-of-hooks.md` | 論点6 ── React の規則が構文の水準にある |
| `legacy.reactjs.org_docs_faq-structure.html` | 論点6 ── React が配置を規定しない |
| `nextjs.org_..._project-structure.md` | 論点6 ── 枠組みが配置を規定する |
