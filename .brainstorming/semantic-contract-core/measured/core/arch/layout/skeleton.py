"""tree.py から空の Rust のワークスペースを組み、cargo check と architecture-lint で構成を実測する。
使い方：python3 skeleton.py <出力するフォルダ>"""
import os, sys, json, shutil
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from tree import TREE, EDGES, OUTBOUND, P

out = sys.argv[1]
shutil.rmtree(out, ignore_errors=True)

def walk(node, base):
    n, d, kids, o, k = node
    path = os.path.join(base, n.rstrip('/'))
    if kids is None:
        if n in ('…', '*.json'): return
        os.makedirs(base, exist_ok=True); open(path, 'a').close()
    else:
        os.makedirs(path, exist_ok=True)
        for c in kids: walk(c, path)
walk(TREE, out)
root = os.path.join(out, 'semantic-agent-schema')
snake = lambda n: n.replace('-', '_')
camel = lambda n: ''.join(w.capitalize() for w in n.split('_'))

crates = {}
for layer in ('domain', 'outbound', 'inbound'):
    base = os.path.join(root, 'crates', layer)
    for c in os.listdir(base):
        if os.path.isdir(os.path.join(base, c)): crates[c] = os.path.join(base, c)
crates['application'] = os.path.join(root, 'crates', 'application')

def deps_of(c):
    if c in EDGES: return EDGES[c]
    if c == 'application': return list(EDGES)
    if c == 'agents-contract': return ['application', 'ai-cli']
    if c in OUTBOUND: return ['application']
    if c == 'cli': return ['application'] + OUTBOUND
    return []

def doc(path, text):
    open(path, 'w').write(text)

for c, path in crates.items():
    rel = lambda d: os.path.relpath(crates[d], path)
    dep = ''.join(f'{P}{d} = {{ path = "{rel(d)}" }}\n' for d in deps_of(c))
    extra = ''
    if c == 'cli':
        extra = ('\n[[bin]]\nname = "semantic-agent-schema"\npath = "src/main.rs"\n'
                 '\n[[bin]]\nname = "semantic-agent-schema-mcp"\npath = "src/bin/mcp.rs"\n'
                 '\n[[test]]\nname = "system"\npath = "tests/system/main.rs"\nharness = false\n')
    doc(os.path.join(path, 'Cargo.toml'),
        f'[package]\nname = "{P}{c}"\nversion.workspace = true\nedition.workspace = true\nrust-version.workspace = true\n\n[lints]\nworkspace = true\n\n[dependencies]\n{dep}{extra}')
    src = os.path.join(path, 'src')
    aggs_here = sorted(f[:-3] for f in os.listdir(os.path.join(src, 'aggregates')) if f.endswith('.rs')) if os.path.isdir(os.path.join(src, 'aggregates')) else []
    def gen(dirpath, modpath):
        names = []
        entries = sorted(os.listdir(dirpath))
        for f in entries:
            full = os.path.join(dirpath, f)
            if os.path.isdir(full):
                if f == 'bin': continue
                kids = gen(full, modpath + [f])
                decl = ''.join(f'pub mod {k};\n' for k in kids)
                if os.path.exists(full + '.rs') and modpath[-1:] == ['aggregates']:
                    # 集約のフォルダ：集約のファイルに子のモジュールを宣言する
                    m = f; ty = camel(m)
                    doc(full + '.rs', f'//! aggregates/{m}（この集約の境界）\n' + decl + f'/// {m}\npub struct {ty} {{ pub(in crate::aggregates::{m}) inner: String }}\nimpl {ty} {{\n    /// 読む\n    pub fn inner(&self) -> &str {{ &self.inner }}\n}}\n')
                else:
                    doc(full + '.rs', '//! ' + '/'.join(modpath + [f]) + '\n' + decl)
                names.append(f)
            elif f.endswith('.rs') and f not in ('lib.rs', 'main.rs') and not os.path.isdir(full[:-3]):
                m = f[:-3]; ty = camel(m); where = modpath[-1] if modpath else ''
                if where == 'aggregates':
                    body = f'/// {m}\npub struct {ty} {{ inner: String }}\nimpl {ty} {{\n    /// 作る\n    pub fn create() -> Self {{ {ty} {{ inner: String::new() }} }}\n    /// 読む\n    pub fn inner(&self) -> &str {{ &self.inner }}\n}}\n'
                elif where == 'entities':
                    scope = 'crate::' + '::'.join(modpath[:-1])
                    body = f'/// {m}\npub struct {ty} {{ pub(in {scope}) inner: String }}\nimpl {ty} {{\n    /// 作る（{scope} の中にだけ見せる）\n    #[allow(dead_code)]\n    pub(in {scope}) fn new() -> Self {{ {ty} {{ inner: String::new() }} }}\n    /// 読む\n    pub fn inner(&self) -> &str {{ &self.inner }}\n}}\n'
                elif where == 'factories':
                    tgt = modpath[-2]; tty = camel(tgt)
                    body = f'use crate::aggregates::{tgt}::{tty};\n/// 集約の欄で作る\npub fn create() -> {tty} {{ {tty} {{ inner: String::new() }} }}\n'
                elif where == 'values':
                    body = f'/// {m}\npub struct {ty}(String);\nimpl {ty} {{\n    /// 作る\n    pub fn new(s: &str) -> Self {{ {ty}(s.into()) }}\n    /// 読む\n    pub fn as_str(&self) -> &str {{ &self.0 }}\n}}\n'
                elif where == 'services':
                    body = f'/// {m}\npub fn run() {{}}\n'
                elif c == 'cli' and m == 'compose':
                    body = '/// outbound を application へ渡す\npub fn compose() {}\n'
                else:
                    body = ''
                doc(full, '//! ' + '/'.join(modpath + [m]) + '\n' + body)
                names.append(m)
        return names
    top = gen(src, [])
    mods = ''.join(f'pub mod {m};\n' for m in sorted(set(top)))
    uses = ''.join(f'pub use {snake(P + x)};\n' for x in deps_of(c))
    doc(os.path.join(src, 'lib.rs'), f'//! {P}{c}\n' + mods + uses)
    if c == 'cli':
        doc(os.path.join(src, 'main.rs'), '//! 実行ファイル semantic-agent-schema\nfn main() { semantic_agent_schema_cli::compose::compose(); }\n')
        doc(os.path.join(src, 'bin', 'mcp.rs'), '//! 実行ファイル semantic-agent-schema-mcp\nfn main() { semantic_agent_schema_cli::compose::compose(); }\n')
    if c == 'declaration':
        doc(os.path.join(path, 'build.rs'), '//! schemas/v1/ を取り込む\nfn main() {}\n')

for dp, dn, fn in os.walk(root):
    for f in fn:
        p = os.path.join(dp, f)
        if f == 'main.rs' and '/tests/' in p:
            doc(p, '//! システムテスト\nmod trace;\nfn main() {}\n' if p.endswith('system/main.rs') else '//! 結合テスト\n')
        if f == 'trace.rs' and '/tests/' in p:
            doc(p, '//! 追跡の検査\n')

doc(os.path.join(root, 'Cargo.toml'),
    '[workspace]\nmembers = ["crates/domain/*", "crates/application", "crates/outbound/*", "crates/inbound/*"]\nresolver = "3"\n\n'
    '[workspace.package]\nversion = "0.1.0"\nedition = "2024"\nrust-version = "1.97"\n\n[workspace.lints.rust]\nmissing_docs = "warn"\n')
os.makedirs(os.path.join(root, '.cargo'), exist_ok=True)
doc(os.path.join(root, '.cargo', 'config.toml'), '[build]\nwarnings = "deny"\n')
comp = [{"name": c, "nodes": [P + c]} for c in crates]
allow = [{"from": c, "to": deps_of(c)} for c in crates if deps_of(c)]
json.dump({"version": 1, "description": "tree.py から作った規則", "components": comp, "allow": allow},
          open(os.path.join(root, 'architecture.json'), 'w'), ensure_ascii=False, indent=1)
print(root)
