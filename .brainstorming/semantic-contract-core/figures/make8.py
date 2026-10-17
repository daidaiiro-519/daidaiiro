# 第18回の図。HTTP は基盤が待ち受けず、Concrete が CLI を包んで受ける（platform-advisor の案C）。
# Q3 の Rust の構成と Q4 の protocol の図から、基盤のインターフェースとしての HTTP を外す。
def sub(p, a, b):
    s = open(p).read(); assert a in s, (p, a); open(p, 'w').write(s.replace(a, b, 1))
sub('q3-rust.src.svg', '>CLI ・ MCP ・ HTTP ・ WASM Component<', '>CLI ・ MCP ・ WASM Component（HTTP は Concrete が包む）<')
sub('q4-protocol.src.svg', '<rect class="box warn" x="392" y="20" width="172" height="62"/><text class="title" x="404" y="46">HTTP</text><text class="note" x="404" y="68">足す</text>',
    '<rect class="boundary" x="392" y="20" width="172" height="62"/><text class="title" x="404" y="46">HTTP</text><text class="note" x="404" y="68">Concrete が CLI を包む</text>')
# 第9回の「標準は JSON Schema だけ」と食い違っていた標準の部品の例も直す。
sub('q3-rust.src.svg', '>jsonschema ・ json-ld など。feature で能力を選ぶ<', '>jsonschema。feature で能力を選ぶ<')
# 第20回：HTTP は Concrete のコマンドの一覧を GET ・ POST で載せる。
sub('q4-protocol.src.svg', '<text class="note" x="404" y="68">Concrete が CLI を包む</text>', '<text class="note" x="404" y="64">Concrete のコマンド</text><text class="note" x="404" y="78">を GET ・ POST で</text>')
