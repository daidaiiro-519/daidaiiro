# 第6回の図：Rust の構成（workspace ・ crate ・ module ・ trait ・ feature）と、外への契約
def e(s): return s.replace('&','&amp;').replace('<','&lt;')
o=['<svg viewBox="0 0 780 470" style="min-width:660px" role="img" aria-label="Rust の構成と、外への契約">']
o.append('<rect class="area" x="10" y="10" width="560" height="450"/><text class="title" x="24" y="34">workspace（版と Cargo.lock は1か所）</text>')
o.append('<rect class="box" x="26" y="48" width="528" height="170"/><text class="title" x="40" y="72">crate：core（入出力に触らない）</text>')
for i,(m,n) in enumerate([("module domain","宣言 ・ 結果の5つの語（enum）・ 決まりの判定"),("module application","作成 ・ 更新 ・ 検査 ・ 承認 ・ 描画の手順"),("module ports","trait：Files ・ Programs ・ Agents（AI を起動する）")]):
    y=86+i*42
    o.append(f'<rect class="box" x="42" y="{y}" width="496" height="34"/><text class="label" x="54" y="{y+22}">{e(m)}</text><text class="note" x="200" y="{y+22}">{e(n)}</text>')
o.append('<path class="flow" d="M290,262 V222"/><text class="note" x="300" y="246">adapters → core の一方向だけ（ビルドが守る）</text>')
o.append('<rect class="box" x="26" y="266" width="528" height="180"/><text class="title" x="40" y="290">crate：adapters（trait を実物で満たす）</text>')
for i,(m,n) in enumerate([("インターフェース","CLI ・ MCP ・ HTTP ・ WASM Component"),("出ていく側","AI の CLI を起動する（Agents 契約）"),("標準の部品","jsonschema ・ json-ld など。feature で能力を選ぶ")]):
    y=304+i*44
    o.append(f'<rect class="box" x="42" y="{y}" width="496" height="36"/><text class="label" x="54" y="{y+23}">{e(m)}</text><text class="note" x="170" y="{y+23}">{e(n)}</text>')
o.append('<rect class="box focus" x="590" y="48" width="180" height="170"/><text class="title focus" x="604" y="72">外への契約</text>')
for i,t in enumerate(["protocol","（JSON Schema）","crate や trait は","外へ出さない"]):
    o.append(f'<text class="label" x="604" y="{100+i*22}">{e(t)}</text>')
o.append('<rect class="boundary" x="590" y="266" width="180" height="180"/><text class="title" x="604" y="290">ほかの言語</text>')
for i,t in enumerate(["TypeScript ・ Python","・ Go の担い手は","protocol でつなぐ","（WASM でも使える）"]):
    o.append(f'<text class="label" x="604" y="{318+i*22}">{e(t)}</text>')
o.append('<path class="link" d="M680,220 V264"/>')
o.append('</svg>')
open('q3-rust.src.svg','w').write('\n'.join(o))
