# 取得した原文の取り直し方

**原文の実体は置かない。** 置くのは `*.meta.json` だけで、url ・ 取得日 ・ `sha256` ・
バイト数 ・ 行数が入っている。取り直して `sha256` が一致すれば、照合した当時と同じ中身である。

```
python3 <source-fidelity>/scripts/source.py fetch <url> --dir .
```

PDF は本文を起こしてから照合する。ブレストボードの根拠が指す行番号は、起こした `.txt` の行番号である。

```
pdftotext -layout <名前>.pdf <名前>.txt
```
