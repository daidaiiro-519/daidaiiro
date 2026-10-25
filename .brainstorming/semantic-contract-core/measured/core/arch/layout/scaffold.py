"""tree.py から semantic-agent-schema/ の雛形を作る（仮の型は置かない。モジュールの宣言と doc コメント、設定、規則、追跡の一覧）。
使い方：python3 scaffold.py <リポジトリの根>"""
import os, sys, json
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
sys.path.insert(0, H)
from tree import TREE, EDGES, OUTBOUND, P, APP, PORTS
repo = sys.argv[1]; root = os.path.join(repo, 'semantic-agent-schema')
m = json.load(open(os.path.join(C, 'model', 'model.json')))
SKIP_EXT = ('.json', '.sql', '.md', '.toml', '.lock')
def write(path, text, keep=False):
    if keep and os.path.exists(path): return
    os.makedirs(os.path.dirname(path), exist_ok=True); open(path, 'w').write(text)
desc = {}
def collect(node, base):
    n, d, kids, o, k = node
    path = os.path.join(base, n.rstrip('/'))
    desc[path] = d
    for c in kids or []: collect(c, path)
collect(TREE, repo)
crates = {}
for layer in ('domain', 'outbound', 'inbound'):
    for node in [x for x in TREE[2] if x[0] == 'crates/'][0][2]:
        pass
def find(node, path):
    n, d, kids, o, k = node
    p = os.path.join(path, n.rstrip('/'))
    yield p, node
    for c in kids or []: yield from find(c, p)
nodes = dict(find(TREE, repo))
for p, node in nodes.items():
    if os.path.basename(p) == 'Cargo.toml' and '/crates/' in p:
        crates[os.path.basename(os.path.dirname(p))] = os.path.dirname(p)
def deps_of(c):
    if c in EDGES: return EDGES[c]
    if c == 'application': return list(EDGES)
    if c == 'agents-contract': return ['application', 'ai-cli']
    if c in OUTBOUND: return ['application']
    if c == 'cli': return ['application'] + OUTBOUND
    return []
# 各 crate
for c, path in crates.items():
    rel = lambda d: os.path.relpath(crates[d], path)
    dep = ''.join(f'{P}{d} = {{ path = "{rel(d)}" }}\n' for d in deps_of(c))
    extra = ''
    if c == 'cli':
        extra = ('\n[[bin]]\nname = "semantic-agent-schema"\npath = "src/main.rs"\n'
                 '\n[[bin]]\nname = "semantic-agent-schema-mcp"\npath = "src/bin/mcp.rs"\n'
                 '\n[[test]]\nname = "system"\npath = "tests/system/main.rs"\nharness = false\n')
    write(os.path.join(path, 'Cargo.toml'),
          f'[package]\nname = "{P}{c}"\ndescription = "{desc.get(path, "")}"\nversion.workspace = true\nedition.workspace = true\nrust-version.workspace = true\nlicense.workspace = true\n\n[lints]\nworkspace = true\n\n[dependencies]\n{dep}{extra}')
    if c == 'declaration':
        write(os.path.join(path, 'build.rs'), '//! schemas/v1/ を取り込み、$id で引く（中身はこれから書く）\nfn main() {}\n')
    src = os.path.join(path, 'src')
    def gen(dirpath, modpath):
        """tree.py の子から、このフォルダのモジュールの名前を返し、ファイルを書く"""
        node = nodes[dirpath]; names = []
        for kid in node[2] or []:
            n = kid[0]; full = os.path.join(dirpath, n.rstrip('/'))
            if kid[2] is not None:
                if n == 'bin/': continue
                sub = gen(full, modpath + [n.rstrip('/')])
                mod = n.rstrip('/')
                decl = ''.join(f'pub mod {s};\n' for s in sub)
                if os.path.join(dirpath, mod + '.rs') in nodes:
                    continue  # 集約のフォルダ：集約のファイルが宣言する
                write(full + '.rs', f'//! {kid[1] or mod}\n' + decl)
                names.append(mod)
            elif n.endswith('.rs') and n not in ('lib.rs', 'main.rs'):
                mod = n[:-3]
                folder = os.path.join(dirpath, mod)
                decl = ''
                if folder in nodes:
                    decl = ''.join(f'pub mod {s};\n' for s in gen(folder, modpath + [mod]))
                write(full, f'//! {kid[1] or mod}\n' + decl)
                names.append(mod)
        return names
    top = gen(src, [])
    uses = ''.join(f'pub use {(P + x).replace("-", "_")};\n' for x in deps_of(c))
    write(os.path.join(src, 'lib.rs'), f'//! {P}{c}：{desc.get(path, "")}\n' + ''.join(f'pub mod {t};\n' for t in top) + uses)
    if c == 'cli':
        write(os.path.join(src, 'compose.rs'), '//! 起動時の結合（outbound を application へ渡す1か所）\n\n/// 起動する（中身はこれから書く）\npub fn run() {}\n')
        write(os.path.join(src, 'main.rs'), '//! 実行ファイル semantic-agent-schema（CLI）\nfn main() {\n    semantic_agent_schema_cli::compose::run();\n}\n')
        write(os.path.join(src, 'bin', 'mcp.rs'), '//! 実行ファイル semantic-agent-schema-mcp（MCP）\nfn main() {\n    semantic_agent_schema_cli::compose::run();\n}\n')
# 結合テスト
for p, node in nodes.items():
    if p.endswith('/tests/system/main.rs'): write(p, '//! システムテスト：test-suite の場面を1件ずつ走らせる（中身はこれから書く）\nmod trace;\nfn main() {}\n')
    elif p.endswith('/tests/system/trace.rs'): write(p, '//! 追跡の検査：trace.json ・ syn ・ grep（中身はこれから書く）\n')
    elif p.endswith('main.rs') and '/tests/' in p: write(p, '//! 結合テスト（中身はこれから書く）\n')
# 根の設定
write(os.path.join(root, 'Cargo.toml'), '''[workspace]
members = ["crates/domain/*", "crates/application", "crates/outbound/*", "crates/inbound/*"]
resolver = "3"

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.97"
license = "MIT"

[workspace.lints.rust]
missing_docs = "deny"

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
missing_errors_doc = "deny"
''')
write(os.path.join(root, 'rustfmt.toml'), 'style_edition = "2024"\n')
write(os.path.join(root, '.cargo', 'config.toml'), '[build]\nwarnings = "deny"\n')
tests_allow = 'allow-unwrap-in-tests = true\nallow-expect-in-tests = true\nallow-panic-in-tests = true\n'
write(os.path.join(root, 'clippy.toml'), tests_allow)
write(os.path.join(root, 'crates', 'domain', 'clippy.toml'), tests_allow + '''disallowed-methods = [
  { path = "std::fs::read", reason = "domain は入出力をしない" },
  { path = "std::fs::write", reason = "domain は入出力をしない" },
  { path = "std::fs::read_to_string", reason = "domain は入出力をしない" },
  { path = "std::time::SystemTime::now", reason = "時刻は inbound から値で渡す" },
]
disallowed-types = [
  { path = "std::fs::File", reason = "domain は入出力をしない" },
  { path = "std::net::TcpStream", reason = "domain は通信をしない" },
  { path = "std::process::Command", reason = "domain はプロセスを起動しない" },
]
''')
comp = [{"name": c, "nodes": [P + c]} for c in sorted(crates)]
allow = [{"from": c, "to": deps_of(c)} for c in sorted(crates) if deps_of(c)]
write(os.path.join(root, 'architecture.json'), json.dumps({"version": 1, "description": "semantic-agent-schema の crate の間で許す依存の辺（arch.html 4節）", "components": comp, "allow": allow}, ensure_ascii=False, indent=1) + '\n')
# 追跡の一覧
inv = [{"id": r[0], "name": r[2]} for r in m['invariants']]
ucs = [{"id": u[0], "name": u[2]} for u in m['use_cases']] + [{"id": "U28", "name": "Semantic Search"}]
trace = {"invariants": inv, "use_cases": ucs, "command_groups": [{"file": f, "name": d} for f, d in APP], "ports": [{"file": f, "name": d} for f, d in PORTS]}
write(os.path.join(root, 'test-suite', 'trace.json'), json.dumps(trace, ensure_ascii=False, indent=1) + '\n')
untested = [{"id": x["id"], "why": "雛形を作った時点で、まだ試験が無い", "move_when": "この行を確かめる試験を書いたら"} for x in inv] + \
           [{"id": f'{u["id"]}:{k}', "why": "雛形を作った時点で、まだ試験が無い", "move_when": "この場面の試験を書いたら"} for u in ucs for k in ('事後', '失敗時')]
write(os.path.join(root, 'test-suite', 'untested.json'), json.dumps(untested, ensure_ascii=False, indent=1) + '\n')
os.makedirs(os.path.join(root, 'test-suite', 'scenarios'), exist_ok=True)
write(os.path.join(root, 'test-suite', 'scenarios', '.gitkeep'), '')
write(os.path.join(root, 'README.md'), '# semantic-agent-schema\n\nAI が JSON Schema の契約のもとで成果物を書き、基盤が機械的に検証する CLI と MCP。\n\n規則は [CODING.md](CODING.md)、依存の向きは [architecture.json](architecture.json)。\n', keep=True)
print(root, len(crates), 'crates')
