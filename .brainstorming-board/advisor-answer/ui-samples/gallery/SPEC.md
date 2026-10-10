# advisor の完成イメージ一覧（UI 案）── パネルを書く担当への指示

## 何を作るか
利用者は、各 advisor が設計相談・判断相談で返す「完成イメージ」の種類ごとに、実際の頁でどう見えるか（UI）を見たい。
1つの頁（サイドバーで1パネルずつ切り替える）に、種類1つにつきパネル1枚を並べる。枠と CSS と JS は用意済みで、あなたが書くのは**パネルの HTML の断片だけ**である。

- 置き場所：`/tmp/claude-1000/-home-daidaiiro-workspace-daidaiiro/4a179eab-201c-42f3-b3d6-fa325e16ded0/scratchpad/ux-sim/gallery/`
- 書くファイル：`frag-<advisor>.html`（advisor は ux ・ ddd ・ usecase ・ qa ・ platform ・ meta-thinking）
- 生成：`python3 assemble.py` で `gallery.html` ができる（全員の断片をまとめる。自分の分だけでも動く）
- 見た目の手本（先に開いて読む）：`../ux-answer-sim.html`（ux の設計相談）・`../ddd-answer-sim.html`（ddd の判断相談）。CSS のクラスはこの2枚にあるものを使う。新しい CSS は書かない（`style` 属性の最小限の幅・色指定だけ可。HTML の部分の色は必ず `var(--…)`。図の色は design-svg の `theme.json` が持つ）

## パネル1枚の形（この形を崩さない）
```html
<section class="panel" id="p-ddd-1" data-p="ddd-1" data-kind="図" data-label="文脈の地図" hidden>
<div class="smp"><span class="kd">図</span><span>ddd-advisor ・ 完成イメージ 1 / 9</span></div><h2>文脈の地図</h2>
<p class="note">出す相談：システムをどう分けるか（設計相談）</p>
… 中身（下の「中身の書き方」）…
<p class="gr">拠る判断基準：context-map ・ bounded-context ・ conformist</p>
</section>
```
- `data-p` は `<advisor>-<番号>`、`id` は `p-` を前に付ける。`data-kind` は 図 ・ 画面 ・ 文書 ・ 表 ・ ファイル のどれか（サイドバーと頭のしるしになる）。`data-label` はサイドバーに出る短い名前（**全角10字くらいまで**。折り返さない長さ）
- 1つの advisor の中で、扱う相談の例を1つの話にそろえると読みやすい（例：ddd は「ネットショップの注文と在庫」）。中身は想定の例でよいが、**それらしい本物の中身**で埋める。「〇〇」「ダミー」「lorem」は書かない

## 中身の書き方
- **図**は design-svg で作る。手で値を書いた SVG や、頁の CSS 変数（`var(--accent)` など）を使う SVG は置かない
  1. 図の種類に近いテンプレートを選ぶ：`/home/daidaiiro/workspace/daidaiiro/.claude/skills/design-svg/bin/design-svg get --kind exemplars`。既にある図の class の版は `/home/daidaiiro/workspace/daidaiiro/.claude/skills/design-svg/tool/business_logic/tests/gallery/class/<data-p>.svg` にある
  2. 座標だけを書き、スタイルは class で指定する（`box` ・ `boundary` ・ `area` ・ `badge` ・ `flow` ・ `link` ・ `title` ・ `label` ・ `note` ・ `focus` ・ `warn` ・ `kind-1`〜`3` ・ `small` など）。色 ・ 文字の大きさ ・ 線の幅 ・ `marker` ・ `style` 要素は書かない。値は `references/theme.json` が持ち、頁の CSS 変数からは取らない
  3. `design-svg resolve <class の SVG> <出力>` で値へ解決する。矢じりとダークモードの `style` は resolve が付け、id には図ごとに別の接頭辞が付く。検出が0件になるまで class か `min-width` を直す
  4. 解決した SVG をそのまま `<figure class="fig">…</figure>` に入れる。同じ図を同じ頁に2回置かない（id が重なり、隠れたパネルの矢じりを参照してしまう）
- **表**は `<div class="tw"><table class="t"><thead>…</thead><tbody>…</tbody></table></div>`。1列目は `<td class="lead">`。縦の見出し表は `class="t kvt"` と `<th scope="row">`
- **直す前と直したあと**は `<div class="grid2"><figure class="fig"><figcaption class="ph before">直す前</figcaption>…</figure><figure class="fig"><figcaption class="ph after">直したあと</figcaption>…</figure></div>`
- **文書**（本文 ・ 計画書 ・ ユースケース本文など）は、見出し・番号付きの手順・表を組み合わせる。白い面の中に置く（`<div class="tw" style="padding:14px 16px">` など）。背景に直接長い文を置かない
- **コード**は `<pre class="code"><code>…</code></pre>`（ddd-answer-sim / ux-answer-sim にある見た目）
- **背景に直接文字を書かない**。説明は `<p class="note">` 1〜2行まで。中身は必ず白い面（表・図・カード）の中

## 言葉の決まり
- ふだん使う言葉で書く。比喩・作った語を使わない。原典の訳語・技術用語はそのまま
- 次の語は使わない：`/home/daidaiiro/workspace/daidaiiro/.doc-writing/retired-words.json` の `word`（先に読む）。とくに「組む」「組み立て」「道具」「要る」「器」
- 「川上と川下」を完成イメージの名前・見出しに使わない（利用者が分かりにくいと言った）。判断基準の id（upstream-and-downstream など）を `拠る判断基準` に書くのはよい
- 判断基準の id は、その advisor の `/home/daidaiiro/workspace/daidaiiro/.claude/skills/<advisor>-advisor/references/criteria.json` に**実在するもの**だけ書く（platform は grades.json の id も可）

## 確かめてから返す
1. `python3 assemble.py`
2. 自分のパネルを全部、ヘッドレスブラウザで 1200 幅と 390 幅で撮って目で見る：
   `C=$(find ~/.cache/ms-playwright/chromium_headless_shell-1234 -name chrome-headless-shell -type f|head -1)`
   `$C --disable-gpu --window-size=1200,1400 --screenshot=shot/<id>.png "file://<gallery の絶対パス>/gallery.html#<data-p>"`
   はみ出し・重なり・語の途中の折れ・空白だらけの図・読めない小ささが無いことを確かめ、あれば直す
   ダークモードは `--blink-settings=preferredColorScheme=0` を足して同じように撮り、矢じりが出ていることと、色が薄くて読めない所が無いことを確かめる
3. ゲート1：`/home/daidaiiro/workspace/daidaiiro/.claude/skills/doc-writing-skills/bin/doc-writing-skills check gallery.html` を実行し、**自分の断片から出た指摘を0件にする**
4. git は使わない。scratchpad の外のファイルを書き換えない
