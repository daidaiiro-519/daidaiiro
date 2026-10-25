# 題材：ACDR

システムテストと受け入れの題材は ACDR である。題材は例であって基盤の中身ではないので、crates/ に題材の語を入れない（追跡の検査が見る）。

場面の状態は、基盤のコマンド（create → patch）で作る。Concrete は持ち主の名前のフォルダ（`acdr/decision-record.schema.json`）に、glossary は glossary の Abstract のファイルの x-instance-path（`.semantic-agent-schema/glossary.json`）に置く。ACDR の Concrete の中身は、ボードの `measured/abstract-draft/concrete.json` に `id` の欄を足したものである。

このフォルダに手で書いたファイルを置くのは、既存のファイルを取り込む場面（段6 の import）だけである。
