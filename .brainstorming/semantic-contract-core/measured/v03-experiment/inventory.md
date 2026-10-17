| id | 出どころ | 決まり | いま表している所 | いまの検査 | v0.3 | いまの基盤で試した結果 | 同じ決まり | メモ |
|---|---|---|---|---|---|---|---|---|
| P1 | brainstorming/assets/board.schema.json:14 | `/` decisions-only：利用者に渡すのは決定だけにし、事実は調べて根拠に書く。（test：問いに、調べれば分かることが入っていないかを見る。） | x-prompt の文 | A | b |  |  | test は「調べれば分かることが入っていないか」。意味の判断 |
| P2 | brainstorming/assets/board.schema.json:20 | `/` refuted-first：案を複数出して反証し、残った1つだけを答えに書く。除外した案は rejected に残す。（test：rejected が1件以上あるかを見る。） | x-prompt の文 | M | a | 標準のキーワードで書けた |  | test は「rejected が1件以上か」。rejected に minItems が無く、どこも数えていない |
| P3 | brainstorming/assets/board.schema.json:50 | `/properties/$schema` this-name：brainstorming/board@v1 と書く。（test：値がこの文字列と同じかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の const が検査する |
| P4 | brainstorming/assets/board.schema.json:67 | `/properties/kind` const：board と書く。（test：値が同じかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の const が検査する |
| P5 | brainstorming/assets/board.schema.json:84 | `/properties/id` file-name：.brainstorming/<id>.json のファイル名と同じにする。（test：ファイル名と同じかを見る。） | x-prompt の文 | O | c |  |  | test は「ファイル名と同じか」。xref-trial の id-differs-file で、check も inspect も見ないことを確かめた |
| P6 | brainstorming/assets/board.schema.json:102 | `/properties/title` short：40字くらいまでの名詞句で書く。（test：40字を超えていないかを見る。） | x-prompt の文 | M | a | 標準のキーワードで書けた |  | test は「40字を超えていないか」。title は minLength 1 だけ |
| P7 | brainstorming/assets/board.schema.json:120 | `/properties/intro` one-sentence：何を決めるかを1文で書く。決まったことの一覧は書かない。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P8 | brainstorming/assets/board.schema.json:138 | `/properties/hold` by-user：利用者が画面の「保留にする」で書き、「保留を解く」で消す。AI は書かない。（test：date があるかを見る。） | x-prompt の文 | O | c |  |  | rule は「AI は書かない」。誰が書いたかはインスタンスに残らない。test の date の有無は required が見る |
| P9 | brainstorming/assets/board.schema.json:159 | `/properties/hold/properties/date` today：保留にした日を書く。（test：日付の形かを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の pattern が検査する |
| P10 | brainstorming/assets/board.schema.json:177 | `/properties/hold/properties/why` optional：利用者が書いたときだけ入れる。（test：空の文字でないかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の minLength が検査する |
| P11 | brainstorming/assets/board.schema.json:197 | `/properties/flag` rare：ほかの作業を止めているときだけ true にする。（test：true のボードが少数かを見る。） | x-prompt の文 | A | b |  |  | test は「true のボードが少数か」。少数の線が決まっておらず、ほかの作業を止めているかは意味の判断 |
| P12 | brainstorming/assets/board.schema.json:214 | `/properties/waits_for` by-id：ほかのボードの id を書く。（test：指す先のボードがあるかを check で見る。） | x-prompt の文 | R | c |  |  | x-ref（to: board）を check がディレクトリの全部のボードで検査する。v0.3 の要求の例はインスタンスを1件しか渡さない |
| P13 | brainstorming/assets/board.schema.json:239 | `/properties/round` by-advance：advance が書く。手で書き換えない。（test：rounds/ の最後の番号と同じかを見る。） | x-prompt の文 | W | c |  |  | advance が書く（advance.rs:142）。test の「rounds/ の最後の番号と同じか」は誰も見ない。rounds/ はインスタンスの外 |
| P14 | brainstorming/assets/board.schema.json:257 | `/properties/opened` by-advance：advance が書く。（test：日付の形かを見る。） | x-prompt の文 | S | a |  |  | test は「日付の形か」で pattern が検査する。書くのは advance |
| P15 | brainstorming/assets/board.schema.json:275 | `/properties/queue` ready-only：前提が片付いた論点だけを並べ、なぜいま開いたかを1文で書く。（test：並べた論点の depends がすべて決着しているかを見る。） | x-prompt の文 | I | a | x-ref で書けた |  | inspect.rs:55 と :74 が見る。xref-trial で x-ref の accept と when でも書けた |
| P16 | brainstorming/assets/board.schema.json:306 | `/properties/queue/items/properties/topic` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P17 | brainstorming/assets/board.schema.json:324 | `/properties/queue/items/properties/why` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P18 | brainstorming/assets/board.schema.json:343 | `/properties/changes` by-advance：advance が、前の回と比べて書く。理由には差し戻しの言葉をそのまま入れる。（test：前の回の記録と比べて、変わった欄だけが並んでいるかを見る。） | x-prompt の文 | W | c |  |  | advance が前の回と比べて書く（advance.rs:162-205）。前の回はインスタンスの外 |
| P19 | brainstorming/assets/board.schema.json:375 | `/properties/changes/items/properties/topic` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P20 | brainstorming/assets/board.schema.json:403 | `/properties/changes/items/properties/part` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P21 | brainstorming/assets/board.schema.json:421 | `/properties/changes/items/properties/summary` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P22 | brainstorming/assets/board.schema.json:439 | `/properties/changes/items/properties/reason` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P23 | brainstorming/assets/board.schema.json:458 | `/properties/log` append：新しいものを先頭に足す。（test：日付が新しい順かを見る。） | x-prompt の文 | M | a |  |  | test は「日付が新しい順か」。どこも見ていない |
| P24 | brainstorming/assets/board.schema.json:483 | `/properties/log/items/properties/date` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P25 | brainstorming/assets/board.schema.json:500 | `/properties/log/items/properties/text` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P26 | brainstorming/assets/board.schema.json:519 | `/properties/topics` three-to-five：3〜5件にし、依存の順に並べる。（test：件数を見る。） | x-prompt の文 | M | a | 標準のキーワードで書けた |  | test は「件数」。minItems 1 だけで、実物のボード2枚とも6件（measure_unchecked.out.json） |
| P27 | brainstorming/assets/board.schema.json:544 | `/$defs/topic` one-decision：1つの論点で決めることを1つにする。（test：答えの1文が1つのことを決めているかを見る。） | x-prompt の文 | A | b |  |  | test は「答えの1文が1つのことを決めているか」。意味の判断 |
| P28 | brainstorming/assets/board.schema.json:569 | `/$defs/topic/properties/id` unique：ボードの中で重ならない id を書く。（test：重なりが無いかを見る。） | x-prompt の文 | M | a |  |  | test は「重なりが無いか」。xref-trial の dup-topic-id で、重なった id は参照されたときにだけ「1つに決まらない」で出た |
| P29 | brainstorming/assets/board.schema.json:587 | `/$defs/topic/properties/name` short：10字くらいの名前を書く。（test：長すぎないかを見る。） | x-prompt の文 | M | a |  |  | test は「長すぎないか」。線が「10字くらい」で決まっていない |
| P30 | brainstorming/assets/board.schema.json:608 | `/$defs/topic/properties/status` by-flow：前提が片付いていなければ waiting、AI が開いて利用者の回答を待つなら open、承認されて更新したら settled にする。（test：queue と合っているかを見る。） | x-prompt の文 | I | a |  |  | test は「queue と合っているか」。inspect.rs:50 ・ :55 ・ :65 が見る |
| P31 | brainstorming/assets/board.schema.json:626 | `/$defs/topic/properties/question` decidable：答えが1つに決まる問いを書く。（test：答えの1文が問いに答えているかを見る。） | x-prompt の文 | A | b |  |  | test は「答えの1文が問いに答えているか」。意味の判断 |
| P32 | brainstorming/assets/board.schema.json:652 | `/$defs/topic/properties/answer` one-sentence-first：承認してほしいことを1文で text に書き、詳しい決まりを details に並べる。（test：text が120字までかを見る。） | x-prompt の文 | I | b |  |  | test は「text が120字までか」。inspect.rs:105 が見るのは欄ではなく1文の長さ（引用を除く） |
| P33 | brainstorming/assets/board.schema.json:671 | `/$defs/topic/properties/answer/properties/text` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P34 | brainstorming/assets/board.schema.json:686 | `/$defs/topic/properties/answer/properties/details` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P35 | brainstorming/assets/board.schema.json:701 | `/$defs/topic/properties/answer/properties/details/items` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P36 | brainstorming/assets/board.schema.json:740 | `/$defs/topic/properties/images` always：答えの完成イメージを1つ以上置く。（test：1つ以上あるかを見る。） | x-prompt の文 | I | a | 標準のキーワードで書けた |  | inspect.rs:123 が open の論点だけ見る。if/then で書けた |
| P37 | brainstorming/assets/board.schema.json:746 | `/$defs/topic/properties/images` kind-by-content：示すものから種類を選ぶ。比べ ・ 流れ ・ 構成は figure、変更前後は diff、触れるものは screen、コマンドは run、フォルダは tree、ファイルの本文は code にする。（test：code の完成イメージが、ファイルとして成り立つ本文（コード ・ 設定 ・ Markdown など）かを見る。） | x-prompt の文 | A | b |  |  | test は「code がファイルとして成り立つ本文か」。inspect.rs:113 は言語が text などかだけを見る |
| P38 | brainstorming/assets/board.schema.json:767 | `/$defs/topic/properties/assumptions` numbered：答えが前提にしたことを1件ずつ書く。（test：1件に1つのことかを見る。） | x-prompt の文 | A | b |  |  | test は「1件に1つのことか」。意味の判断 |
| P39 | brainstorming/assets/board.schema.json:782 | `/$defs/topic/properties/assumptions/items` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P40 | brainstorming/assets/board.schema.json:812 | `/$defs/topic/properties/checks` three：読み手に確かめてほしい箇所を3件まで書く。（test：3件以下かを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の maxItems 3 が検査する |
| P41 | brainstorming/assets/board.schema.json:830 | `/$defs/topic/properties/checks/items/properties/target` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P42 | brainstorming/assets/board.schema.json:846 | `/$defs/topic/properties/checks/items/properties/ask` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P43 | brainstorming/assets/board.schema.json:877 | `/$defs/topic/properties/grounds` three-parts：答えのどこを支えるか ・ もとにしたこと ・ 出どころの3つを書く。（test：3つがそろっているかを見る。） | x-prompt の文 | S | a |  |  | items の required が4つを検査する |
| P44 | brainstorming/assets/board.schema.json:895 | `/$defs/topic/properties/grounds/items/properties/supports` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P45 | brainstorming/assets/board.schema.json:918 | `/$defs/topic/properties/grounds/items/properties/basis` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P46 | brainstorming/assets/board.schema.json:936 | `/$defs/topic/properties/grounds/items/properties/tag` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P47 | brainstorming/assets/board.schema.json:952 | `/$defs/topic/properties/grounds/items/properties/source` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P48 | brainstorming/assets/board.schema.json:981 | `/$defs/topic/properties/rejected` keep：反証で外した案と、何が壊れるかを書く。（test：1件以上あるかを見る。） | x-prompt の文 | M | a | 標準のキーワードで書けた |  | test は「1件以上あるか」。rejected に minItems が無い |
| P49 | brainstorming/assets/board.schema.json:999 | `/$defs/topic/properties/rejected/items/properties/option` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P50 | brainstorming/assets/board.schema.json:1015 | `/$defs/topic/properties/rejected/items/properties/reason` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P51 | brainstorming/assets/board.schema.json:1042 | `/$defs/topic/properties/depends` declared：前提にした論点の id を書く。（test：指す論点があるかを見る。） | x-prompt の文 | R | a |  |  | x-ref（to: self）を check が、inspect.rs:72 も検査する |
| P52 | brainstorming/assets/board.schema.json:1077 | `/$defs/topic/properties/verification` not-shown：書き手の検証の記録を書く。描画しない。（test：画面に出ていないかを見る。） | x-prompt の文 | W | b |  |  | x-view の hidden に render が従う。test の「画面に出ていないか」は描いてみないと分からない |
| P53 | brainstorming/assets/board.schema.json:1094 | `/$defs/topic/properties/verification/properties/passed` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P54 | brainstorming/assets/board.schema.json:1110 | `/$defs/topic/properties/verification/properties/passed/items/properties/option` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P55 | brainstorming/assets/board.schema.json:1129 | `/$defs/topic/properties/verification/properties/passed/items/properties/cost` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P56 | brainstorming/assets/board.schema.json:1143 | `/$defs/topic/properties/verification/properties/assumptions` numbered：答えが前提にしたことを1件ずつ書く。（test：1件に1つのことかを見る。） | x-prompt の文 | A | b |  |  | test は「1件に1つのことか」。意味の判断 |
| P57 | brainstorming/assets/board.schema.json:1170 | `/$defs/topic/properties/verification/properties/assumptions/items` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P58 | brainstorming/assets/board.schema.json:1190 | `/$defs/topic/properties/verification/properties/out_of_scope` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P59 | brainstorming/assets/board.schema.json:1207 | `/$defs/topic/properties/verification/properties/out_of_scope/items/properties/item` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P60 | brainstorming/assets/board.schema.json:1223 | `/$defs/topic/properties/verification/properties/out_of_scope/items/properties/treatment` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P61 | brainstorming/assets/board.schema.json:1242 | `/$defs/topic/properties/verification/properties/out_of_scope/items/properties/note` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P62 | brainstorming/assets/board.schema.json:1256 | `/$defs/topic/properties/verification/properties/history` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P63 | brainstorming/assets/board.schema.json:1271 | `/$defs/topic/properties/verification/properties/history/items` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P64 | brainstorming/assets/board.schema.json:1291 | `/$defs/figure` one-thing：この完成イメージで示すことを1つにし、caption にそれを1文で書く。（test：caption が1文かを見る。） | x-prompt の文 | M | b |  |  | test は「caption が1文か」。文の切り方（text.rs の sentences）を持つ Interpreter を必要とする |
| P65 | brainstorming/assets/board.schema.json:1313 | `/$defs/figure/properties/kind` as-is：完成イメージの種類の名前をそのまま書く。（test：6つの名前のどれかかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の const が検査する |
| P66 | brainstorming/assets/board.schema.json:1331 | `/$defs/figure/properties/caption` one-sentence：何を示すかを1文で書く。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P67 | brainstorming/assets/board.schema.json:1350 | `/$defs/figure/properties/svg` checked：design-svg の検査を通した SVG を書く。（test：render が断らないかを見る。） | x-prompt の文 | W | b |  |  | contentMediaType image/svg+xml の入力を render が検査する（schema-driven SKILL.md:173） |
| P68 | brainstorming/assets/board.schema.json:1370 | `/$defs/screen` one-thing：この完成イメージで示すことを1つにし、caption にそれを1文で書く。（test：caption が1文かを見る。） | x-prompt の文 | M | b |  |  | test は「caption が1文か」。文の切り方（text.rs の sentences）を持つ Interpreter を必要とする |
| P69 | brainstorming/assets/board.schema.json:1392 | `/$defs/screen/properties/kind` as-is：完成イメージの種類の名前をそのまま書く。（test：6つの名前のどれかかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の const が検査する |
| P70 | brainstorming/assets/board.schema.json:1410 | `/$defs/screen/properties/caption` one-sentence：何を示すかを1文で書く。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P71 | brainstorming/assets/board.schema.json:1428 | `/$defs/screen/properties/html` standalone：外のファイルを読まない1枚の HTML を書く。（test：外の URL を読んでいないかを見る。） | x-prompt の文 | M | b |  |  | test は「外の URL を読んでいないか」。この実験で読んだ範囲では、見ているコードを見つけていない |
| P72 | brainstorming/assets/board.schema.json:1447 | `/$defs/screen/properties/height` fit：中身が収まる高さを書く。（test：描いて目で見る。） | x-prompt の文 | A | b |  |  | test は「描いて目で見る」。人の目の判断 |
| P73 | brainstorming/assets/board.schema.json:1467 | `/$defs/code` one-thing：この完成イメージで示すことを1つにし、caption にそれを1文で書く。（test：caption が1文かを見る。） | x-prompt の文 | M | b |  |  | test は「caption が1文か」。文の切り方（text.rs の sentences）を持つ Interpreter を必要とする |
| P74 | brainstorming/assets/board.schema.json:1490 | `/$defs/code/properties/kind` as-is：完成イメージの種類の名前をそのまま書く。（test：6つの名前のどれかかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の const が検査する |
| P75 | brainstorming/assets/board.schema.json:1508 | `/$defs/code/properties/caption` one-sentence：何を示すかを1文で書く。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P76 | brainstorming/assets/board.schema.json:1526 | `/$defs/code/properties/path` as-in-repo：リポジトリの根からの場所で書く。（test：その場所が在るか、作る予定かを見る。） | x-prompt の文 | O | c |  |  | test は「その場所が在るか、作る予定か」。リポジトリと書き手の予定を必要とする |
| P77 | brainstorming/assets/board.schema.json:1544 | `/$defs/code/properties/language` lowercase：言語の名前を小文字で書く。（test：知っている名前かを見る。） | x-prompt の文 | M | a |  |  | test は「知っている名前か」。言語の名前の一覧を持てば汎用の検査で足りる |
| P78 | brainstorming/assets/board.schema.json:1550 | `/$defs/code/properties/language` file-language：そのファイルの言語（rust ・ json ・ markdown ・ shell など）を書く。text は書かない。（test：inspect が、言語が text ・ plain ・ txt のコードを見つけないかを見る。） | x-prompt の文 | I | a | 標準のキーワードで書けた |  | inspect.rs:113 が見る。not enum で書けた（ただし大文字小文字を区別する） |
| P79 | brainstorming/assets/board.schema.json:1568 | `/$defs/code/properties/text` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P80 | brainstorming/assets/board.schema.json:1574 | `/$defs/code/properties/text` real-file：実在するか作る予定のファイルの本文だけを書く。場面の比べや一覧は図にする。（test：path に書いたファイルの本文として成り立つかを見る。） | x-prompt の文 | A | b |  |  | test は「path のファイルの本文として成り立つか」。言語ごとの読み手を必要とする |
| P81 | brainstorming/assets/board.schema.json:1596 | `/$defs/code/properties/highlight` few：読んでほしい行だけを書く。（test：全体の行の数より十分少ないかを見る。） | x-prompt の文 | A | b |  |  | test は「全体の行の数より十分少ないか」。十分の線が無い |
| P82 | brainstorming/assets/board.schema.json:1614 | `/$defs/code/properties/note` why-highlight：強調した行が何かを1文で書く。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P83 | brainstorming/assets/board.schema.json:1634 | `/$defs/diff` one-thing：この完成イメージで示すことを1つにし、caption にそれを1文で書く。（test：caption が1文かを見る。） | x-prompt の文 | M | b |  |  | test は「caption が1文か」。文の切り方（text.rs の sentences）を持つ Interpreter を必要とする |
| P84 | brainstorming/assets/board.schema.json:1658 | `/$defs/diff/properties/kind` as-is：完成イメージの種類の名前をそのまま書く。（test：6つの名前のどれかかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の const が検査する |
| P85 | brainstorming/assets/board.schema.json:1676 | `/$defs/diff/properties/caption` one-sentence：何を示すかを1文で書く。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P86 | brainstorming/assets/board.schema.json:1694 | `/$defs/diff/properties/path` as-in-repo：リポジトリの根からの場所で書く。（test：その場所が在るか、作る予定かを見る。） | x-prompt の文 | O | c |  |  | test は「その場所が在るか、作る予定か」。リポジトリと書き手の予定を必要とする |
| P87 | brainstorming/assets/board.schema.json:1712 | `/$defs/diff/properties/language` lowercase：言語の名前を小文字で書く。（test：知っている名前かを見る。） | x-prompt の文 | M | a |  |  | test は「知っている名前か」。言語の名前の一覧を持てば汎用の検査で足りる |
| P88 | brainstorming/assets/board.schema.json:1729 | `/$defs/diff/properties/before` verbatim：変更前の原文をそのまま書く。（test：原文と同じかを見る。） | x-prompt の文 | O | c |  |  | test は「原文と同じか」。リポジトリのある時点のファイルを必要とする |
| P89 | brainstorming/assets/board.schema.json:1746 | `/$defs/diff/properties/after` verbatim：変更後の原文をそのまま書く。（test：原文と同じかを見る。） | x-prompt の文 | O | c |  |  | test は「原文と同じか」。リポジトリのある時点のファイルを必要とする |
| P90 | brainstorming/assets/board.schema.json:1766 | `/$defs/diff/properties/layout` default-unified：並べたいときだけ split と書く。（test：値を見る。） | x-prompt の文 | S | a |  |  | 同じ節点の enum が検査する |
| P91 | brainstorming/assets/board.schema.json:1786 | `/$defs/tree` one-thing：この完成イメージで示すことを1つにし、caption にそれを1文で書く。（test：caption が1文かを見る。） | x-prompt の文 | M | b |  |  | test は「caption が1文か」。文の切り方（text.rs の sentences）を持つ Interpreter を必要とする |
| P92 | brainstorming/assets/board.schema.json:1808 | `/$defs/tree/properties/kind` as-is：完成イメージの種類の名前をそのまま書く。（test：6つの名前のどれかかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の const が検査する |
| P93 | brainstorming/assets/board.schema.json:1826 | `/$defs/tree/properties/caption` one-sentence：何を示すかを1文で書く。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P94 | brainstorming/assets/board.schema.json:1856 | `/$defs/tree/properties/rows` top-down：上から順に、フォルダの中身を字下げで並べる。（test：depth が親より1つ深いかを見る。） | x-prompt の文 | M | b |  |  | test は「depth が親より1つ深いか」。並びの前後を比べる専用の検査を必要とする |
| P95 | brainstorming/assets/board.schema.json:1875 | `/$defs/tree/properties/rows/items/properties/path` as-in-repo：リポジトリの根からの場所で書く。（test：その場所が在るか、作る予定かを見る。） | x-prompt の文 | O | c |  |  | test は「その場所が在るか、作る予定か」。リポジトリと書き手の予定を必要とする |
| P96 | brainstorming/assets/board.schema.json:1893 | `/$defs/tree/properties/rows/items/properties/depth` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P97 | brainstorming/assets/board.schema.json:1914 | `/$defs/tree/properties/rows/items/properties/note` why-highlight：強調した行が何かを1文で書く。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P98 | brainstorming/assets/board.schema.json:1930 | `/$defs/tree/properties/rows/items/properties/mark` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P99 | brainstorming/assets/board.schema.json:1950 | `/$defs/run` one-thing：この完成イメージで示すことを1つにし、caption にそれを1文で書く。（test：caption が1文かを見る。） | x-prompt の文 | M | b |  |  | test は「caption が1文か」。文の切り方（text.rs の sentences）を持つ Interpreter を必要とする |
| P100 | brainstorming/assets/board.schema.json:1973 | `/$defs/run/properties/kind` as-is：完成イメージの種類の名前をそのまま書く。（test：6つの名前のどれかかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の const が検査する |
| P101 | brainstorming/assets/board.schema.json:1991 | `/$defs/run/properties/caption` one-sentence：何を示すかを1文で書く。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P102 | brainstorming/assets/board.schema.json:2009 | `/$defs/run/properties/cwd` where：実行した場所を書く。（test：場所が在るかを見る。） | x-prompt の文 | O | c |  |  | test は「場所が在るか」 |
| P103 | brainstorming/assets/board.schema.json:2027 | `/$defs/run/properties/command` as-typed：打ったとおりに書く。（test：実行できるかを見る。） | x-prompt の文 | O | c |  |  | test は「実行できるか」 |
| P104 | brainstorming/assets/board.schema.json:2044 | `/$defs/run/properties/output` as-printed：出たとおりに書き、長ければ途中を省いたと書く。（test：実物の出力と比べる。） | x-prompt の文 | O | c |  |  | test は「実物の出力と比べる」 |
| P105 | brainstorming/assets/board.schema.json:2061 | `/$defs/run/properties/exit` as-returned：返った終了コードを書く。（test：整数かを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の type integer が検査する |
| I1 | domain/inspect.rs:40 | 回答待ち（open）の論点は5件まで | Rust | I | a | 標準のキーワードで書けた |  | contains と maxContains で書けた |
| I2 | domain/inspect.rs:50 | open の論点があれば queue は空でない | Rust | I | a | 標準のキーワードで書けた | x-prompt:608 | if/then で書けた |
| I3 | domain/inspect.rs:58 | queue の topic が指す論点がある | Rust | I | a |  | x-ref:/properties/queue/items/properties/topic | x-ref の check と同じことを見る |
| I4 | domain/inspect.rs:55 | queue の topic が指す論点は open | Rust | I | a | x-ref で書けた | x-prompt:275 | accept（at: status, in: [open]） |
| I5 | domain/inspect.rs:62 | ボードに status を書かない | Rust | I | a | 標準のキーワードで書けた | G8 | 根の additionalProperties:false がすでに断る（xref-trial/boards/board-status.json で確かめた） |
| I6 | domain/inspect.rs:65 | open が無いのに決着していない論点があれば誤り | Rust | I | a |  | x-prompt:608 | anyOf と contains で書けるはずだが試していない |
| I7 | domain/inspect.rs:72 | depends が指す論点がある | Rust | I | a |  | x-prompt:1042 | x-ref の check と同じことを見る |
| I8 | domain/inspect.rs:74 | open の論点の depends はすべて settled | Rust | I | a | x-ref で書けた | x-prompt:275 | when（status: [open]）と accept（at: status, in: [settled]） |
| I9 | domain/inspect.rs:92 | 答えの本文が指す論点（Q と数字）は depends か下流にある | Rust | I | b |  |  | 本文から id を拾う決まり（inspect.rs:12 の mentioned）を持つ Interpreter を必要とする |
| I10 | domain/inspect.rs:105 | 答え ・ 詳しい決まり ・ 外した理由の1文は120字まで（「」の中は数えない） | Rust | I | b |  | x-prompt:652 | 文の切り方は text.rs:7 ・ 上限は text.rs:4 |
| I11 | domain/inspect.rs:113 | code の完成イメージの言語は text ・ plain ・ txt ・ 空でない | Rust | I | a | 標準のキーワードで書けた | x-prompt:1550 | not enum で書けた |
| I12 | domain/inspect.rs:123 | open の論点には完成イメージがある | Rust | I | a | 標準のキーワードで書けた | x-prompt:740 | if/then で書けた |
| I13 | domain/inspect.rs:132 | open の論点は、下流の論点に使われるか、外の作業の行き先が out_of_scope にある | Rust | I | b |  |  | 逆向きの参照と文字列の中身の「または」。x-ref の inverse には「または」が無い |
| D1 | domain/status.rs:69 | 論点のステータス：settled→決着、open→回答待ち、ほか→前提待ち | Rust | W | c |  |  | いまの基盤の x-derive（rules ・ otherwise）なら書ける形。v0.3 の試案には導く決まりの置き場所が無い |
| D2 | domain/status.rs:79 | AI が動いていればボードは更新中 | Rust | W | c |  |  | ロックのファイルとプロセスの状態 |
| D3 | domain/status.rs:82 | hold があればボードは保留 | Rust | W | c |  |  | 導く決まり。x-derive の when は値の一致だけで、有無を見られない |
| D4 | domain/status.rs:86 | 論点がすべて settled（1件以上）ならボードは決着 | Rust | W | c |  |  | 導く決まり。全部についての条件 |
| D5 | domain/status.rs:89 | それ以外のボードは回答待ち | Rust | W | c |  |  | 導く決まり |
| L1 | domain/lock.rs:206 | ロックが新しいあいだは持ち主だけが書く | Rust | W | c |  |  | プロセスと時間 |
| L2 | domain/lock.rs:175 | serve のロックは60秒で古くなる | Rust | W | c |  |  | 時間 |
| L3 | domain/lock.rs:177 | ターミナルのロックは10分で古くなる | Rust | W | c |  |  | 時間 |
| L4 | domain/lock.rs:190 | ロックの中身は board ・ owner ・ what ・ since ・ beat | Rust | W | a |  |  | 形の決まりだがスキーマが無く、コードの中にだけある |
| G1 | brainstorming/SKILL.md:119 | 案を並べて選ばせない | SKILL.md の文 | A | b |  | x-prompt:20 |  |
| G2 | brainstorming/SKILL.md:120 | 事実を利用者に聞かない | SKILL.md の文 | A | b |  | x-prompt:14 |  |
| G3 | brainstorming/SKILL.md:121 | 外した案を黙って消さない | SKILL.md の文 | W | c |  |  | advance が数の増減を changes に書く（advance.rs:177）。前の回はインスタンスの外 |
| G4 | brainstorming/SKILL.md:122 | ボードの JSON を手で書き換えない | SKILL.md の文 | O | c |  |  | 誰がどう書いたか |
| G5 | brainstorming/SKILL.md:123 | rounds/ と answers/ を書き換えない | SKILL.md の文 | O | c |  |  | この実験で読んだ範囲では、見ているコードを見つけていない |
| G6 | brainstorming/SKILL.md:124 | ボードのフォルダの外を書き換えない | SKILL.md の文 | W | c |  |  | serve が作業の前後を比べて知らせる（scope.rs:14） |
| G7 | brainstorming/SKILL.md:125 | AI の「検査した」で合格としない | SKILL.md の文 | O | a |  |  | 決まりではなく手順。v0.3 では Interpreter の応答の status と providedCapabilities が持つ |
| G8 | brainstorming/SKILL.md:126 | ボードに status を書かない | SKILL.md の文 | S | a | 標準のキーワードで書けた | I5 | 根の additionalProperties:false |
| G9 | brainstorming/SKILL.md:127 | ロックを残したまま作業をやめない | SKILL.md の文 | W | c |  |  | 外し忘れは lock.rs の時間で外れる |
| G10 | brainstorming/SKILL.md:128 | 画面の AI に聞くで、Kiro は読むだけ | SKILL.md の文 | W | c |  |  | agent.rs:3 が起動のしかたで絞る |
| G11 | brainstorming/SKILL.md:129 | 扱わない範囲に扱いを付ける | SKILL.md の文 | S | a |  |  | out_of_scope の items の required と enum |
