# 描画して確かめる

**公開する前に、必ず画像にして目で見る**。HTMLを書いただけで出さない。

---

## 1. 描画の道具を用意する

```bash
CH=$HOME/.cache/ms-playwright/chromium_headless_shell-*/chrome-headless-shell-linux64/chrome-headless-shell
```

無ければ `npx playwright install chromium-headless-shell` で入る。
共有ライブラリが足りないときは `libasound2t64` を展開して `LD_LIBRARY_PATH` に渡す。

---

## 2. 1枚を画像にする

```bash
"$CH" --no-sandbox --disable-gpu --hide-scrollbars \
  --window-size=1280,720 --screenshot=out.png \
  --virtual-time-budget=6000 "file:///絶対パス/deck.html#12"
```

**URLの `#12` が枚番である**。書き換えたら必ずその枚を描画して見る。

---

## 3. はみ出しを測る

**ここを最初に見る。**
はみ出しと切れは最も多い欠陥で、**見た人に必ず見える**（`knowledge/visual-design-for-slide-decks.md` の出所）。


**`.body` を測ってはいけない**。`flex:1` で下端まで伸びる器なので、
中身が溢れていても常に枠内に収まって見える。

**葉の要素の下端を測る。**

```js
setTimeout(()=>{
  const out=[];
  document.querySelectorAll('.slide').forEach((sl,i)=>{
    const prev=sl.style.display; sl.style.display='flex';
    let bottom=0;
    sl.querySelectorAll('*').forEach(el=>{
      if(el.classList.contains('body')) return;
      if(el.children.length && !el.classList.contains('paper') && !el.classList.contains('pcard')) return;
      const r=el.getBoundingClientRect();
      if(r.height>0) bottom=Math.max(bottom, r.bottom);
    });
    const st=document.getElementById('stage').getBoundingClientRect();
    out.push((i+1)+':'+Math.round(bottom-st.top));
    sl.style.display=prev;
  });
  document.title='BOTTOM '+out.join(' ');
},2500);
```

この断片をデッキの複製に追記し、`--dump-dom` で `<title>` を読む。

```
上限   642   （720 − 下余白78）
目安   620を超えたら、その枚は中身を減らす
```

---

## 4. 画面サイズへの追従を壊さない

**固定ステージ 1280×720 を、JSで拡大縮小する。**

```
.deck    position:absolute; inset:0; overflow:hidden
.stage   position:absolute; left:50%; top:50%; width:1280px; height:720px
JS       stage.style.transform='translate(-50%,-50%) scale(k)'
         k = Math.min(deck幅/1280, deck高/720)
```

**踏んだ落とし穴が3つある。**

```
画面幅を見るメディアクエリを残す   端末が狭いと、1280pxの中で2段組みが1段に潰れて縦に溢れる
グリッドで中央寄せする             トラックが1280pxのまま残り、ステージが右へずれる
position:fixed を使う              iOS の iframe で当てにならない
```

**`.chrome` と `.progress` は `.stage` の中に置く**。外に出ると縮尺から外れる。
`</div>` の数を数えて確かめること。

---

## 4.5 新しいクラス名は、既存と衝突していないか確かめる

**枚に足したクラス名が、デッキの共通CSSに既にあると、意図しない指定を拾う。**
`display` や `align-items` を上書きされ、要素が別の場所へ飛ぶ。

```bash
grep -o '^\s*\.[a-z0-9-]*' deck.html | tr -d ' .' | sort -u | grep -x '<新しいクラス名>'
```

**枚に固有のクラスには、その枚だけの接頭辞を付ける。**

```
危ない   .two ／ .gap ／ .card ／ .row     一般名詞は既に使われている
安全     .czrow ／ .mirbox                 枚の名前を頭に付ける
```

---

## 5. 入りきらない枚を自動で縮める

環境によってフォントの読み込みが違い、行が増えることがある。**保険を入れる。**

```js
const shrink=(sl)=>{
  const b=sl.querySelector('.body'); if(!b) return;
  b.style.transform=''; b.style.width='';
  const avail=b.clientHeight, need=b.scrollHeight;
  if(need>avail+1){
    const k=Math.max(.72, avail/need);
    b.style.transformOrigin='top left';
    b.style.transform='scale('+k+')';
    b.style.width=(100/k)+'%';
  }
};
```

枚を表示するたびと、画面サイズが変わるたびに呼ぶ。

**これは保険であって、設計ではない**。縮んだ枚があるなら、中身を減らす。

---

## 5.5 出たものから、中身を取り出して読む

**書いたものを読み返しても、入れ忘れは見つからない**（概念7）。
**描画されたテキストを抜き出して、それを読む。**

```
入れ忘れ   置くはずのものが無いか
順序       枚の並びが意図どおりか
置き忘れ   仮の文字が残っていないか
```

**置き忘れは機械で探せる。**

```
grep -iEo "lorem|ipsum|TODO|\[insert|xxx+|ダミー|仮置き" deck.html
```

**出たら、直してから公開する。**

---

## 5.6 元から在った壊れと、自分が作った壊れを分ける

**直す前に1度検査して、件数を控える**（概念8）。
**増えたぶんだけが、自分が作った壊れである。**

**土台にした素材が、もともと壊れていることがある。**
分けずに数えると、自分が作っていない欠陥を自分のものとして直すことになる。

---

## 6. 公開する前の確認

```
1  変更した枚を描画して、目で見た
2  全枚の下端を測って、620を超える枚を把握した
3  縦長の画面（393×852）と横長（1440×900）で、はみ出しが無いことを見た
4  枚数・ラベル数・通し番号の3つが一致している
```
