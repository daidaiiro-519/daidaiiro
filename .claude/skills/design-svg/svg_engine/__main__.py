"""目録をJSONで吐く入口 ── `python -m svg_engine > catalog.json`。

利用側（変換器を書く人）は、この出力だけを見れば済む。Python から使うなら
`svg_engine.catalog()` を直接呼べばよく、そちらと同じものが出る。

`python -m svg_engine.catalog` ではなくここに置くのは、パッケージの読み込みで
既に catalog が読まれているため、モジュールを直接実行すると二重読み込みの
警告が出るから。
"""
from .catalog import main

main()
