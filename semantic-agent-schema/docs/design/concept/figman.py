"""手で座標を置く図（class で書き、design-svg resolve で検査する）。版11 で崩れていた図を置き換える。"""
import os, subprocess
from html import escape as e
H = os.path.dirname(os.path.abspath(__file__))
DS = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', '..', '..', '.claude', 'skills', 'design-svg', 'bin', 'design-svg'))
REPORT = []

class Fig:
    def __init__(s, W, Hh, label): s.W, s.H, s.label, s.o = W, Hh, label, []
    def area(s, x, y, w, h, title=None, tx=None):
        s.o.append(f'<rect class="area" x="{x}" y="{y}" width="{w}" height="{h}"/>')
        if title: s.o.append(f'<text class="note" x="{tx if tx is not None else x+12}" y="{y+20}" font-weight="700">{e(title)}</text>')
    def bound(s, x, y, w, h, title=None, tx=None):
        s.o.append(f'<rect class="boundary" x="{x}" y="{y}" width="{w}" height="{h}"/>')
        if title: s.o.append(f'<text class="label" x="{tx if tx is not None else x+12}" y="{y+20}" font-weight="700">{e(title)}</text>')
    def box(s, x, y, w, h, lines, role='', dashed=False):
        cls = 'boundary' if dashed else ('box ' + role).strip()
        s.o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{w}" height="{h}"/>')
        n = len(lines); y0 = y + h / 2 - (n - 1) * 9 + 4
        for i, t in enumerate(lines):
            s.o.append(f'<text class="{("label " + role).strip()}" x="{x + w / 2}" y="{y0 + i * 18}" text-anchor="middle">{e(t)}</text>')
    def path(s, d, dashed=False, arrow=True):
        s.o.append(f'<path class="{"flow" if arrow else "link"}{" async" if dashed else ""}" d="{d}"/>')
    def note(s, x, y, t, anchor='start', bold=False):
        s.o.append(f'<text class="note" x="{x}" y="{y}" text-anchor="{anchor}"{" font-weight=\"700\"" if bold else ""}>{e(t)}</text>')
    def write(s, name, minw=760):
        src = os.path.join(H, 'fig', name + '.source.svg'); out = os.path.join(H, 'fig', name + '.svg')
        svg = f'<svg viewBox="0 0 {s.W} {s.H}" width="100%" style="min-width:{minw}px;max-width:{s.W}px;display:block;margin:0 auto" role="img" aria-label="{e(s.label)}">' + ''.join(s.o) + '</svg>'
        open(src, 'w').write(svg)
        p = subprocess.run([DS, 'resolve', src, '--out', out], capture_output=True, text=True)
        res = [l for l in (p.stdout + p.stderr).splitlines() if '検査' in l]
        REPORT.append((name, res))
        return open(out).read()

def overview():
    f = Fig(1000, 700, 'SAS と使う側と一般の製品')
    f.area(10, 10, 980, 190, '使う側（SAS の外）')
    f.box(30, 40, 290, 56, ['オーケストレーター：', '目的のセッション・親子・子への委譲'])
    f.box(30, 124, 290, 56, ['進め方としての目的', '（目的ごとに進める営み）'], dashed=True)
    f.box(355, 40, 290, 56, ['変更の運用：', '承認・まとめ・コミット・PR'])
    f.box(680, 40, 290, 56, ['Concrete を持つ Skill・ガイド'])
    f.path('M175,96 V122', dashed=True)
    f.area(10, 240, 980, 290, 'Semantic Agent Schema')
    f.path('M500,96 V268'); f.note(510, 226, '使う（公開する形だけ）')
    f.box(340, 270, 320, 56, ['公開する形：Abstract の版・', 'CLI の出力・描画の契約'], 'focus')
    f.path('M500,326 V358')
    f.box(340, 360, 320, 56, ['能力：作成・更新・削除・', '検証・描画・検索'], 'focus')
    f.path('M500,416 V448'); f.path('M340,388 H175 V448'); f.path('M660,388 H825 V448', dashed=True)
    f.box(30, 450, 290, 64, ['記録：セッションと操作', '（動いているセッションを回す）'], 'focus')
    f.box(355, 450, 290, 64, ['契約：セッションの4つの型・差分', '（Git は読むだけ）・検査・描画'], 'focus')
    f.box(680, 450, 290, 64, ['意味としての目的：rule.why・', 'reading.when・description・意図'], 'kind-1')
    f.area(10, 580, 980, 110, '一般の製品（差し込む）')
    f.path('M500,514 V618', dashed=True); f.note(508, 566, 'アダプター')
    f.path('M420,514 V556 H260 V618', dashed=True); f.note(268, 574, '読むだけ')
    f.path('M580,514 V556 H825 V618', dashed=True); f.note(833, 574, '差し込む')
    f.box(30, 620, 290, 56, ['製品：Git', '（コミット済みの版を読むだけ）'])
    f.box(355, 620, 290, 56, ['製品：コーディングエージェント', '（どのツールでも）'])
    f.box(680, 620, 290, 56, ['製品：2020-12 の検査器・', 'テンプレートの処理系'])
    return f.write('overview')

def layers():
    f = Fig(1000, 520, '事業領域・業務領域・区切られた文脈')
    f.area(10, 10, 980, 196, '問題の側：業務領域（見つける）')
    f.box(330, 36, 340, 50, ['事業領域：Semantic Agent Schema（プロダクト全体）'], 'kind-1')
    xs = [36, 224, 412, 600, 788]
    for x in xs: f.path(f'M500,86 V108 H{x+88} V128', arrow=True)
    f.note(512, 102, '分解')
    f.box(36, 130, 176, 56, ['意味の操作（中核）', 'ドメインモデル'], 'focus')
    f.box(224, 130, 176, 56, ['契約への適合（中核）', 'ドメインモデル'], 'focus')
    f.box(412, 130, 176, 56, ['操作の記録（補完）', 'トランザクションスクリプト'])
    f.box(600, 130, 176, 56, ['描画（補完）', 'トランザクションスクリプト'])
    f.box(788, 130, 176, 56, ['一般：版の管理・検査・', 'エージェント・描画の部品'], dashed=True)
    f.area(10, 250, 980, 260, '解決の側（設計する）', tx=136)
    f.bound(24, 280, 952, 124, '文脈「契約とインスタンス」（1つ）', tx=136)
    labs = [['意味の操作のモジュール'], ['契約への適合のモジュール'], ['操作の記録のモジュール'], ['描画のモジュール'], ['契約（ポートと SAS の型）']]
    for x, l in zip(xs, labs):
        f.path(f'M{x+88},186 V326', dashed=True)
        f.box(x, 328, 176, 56, l, 'focus' if x < 400 else '')
    f.note(132, 230, '対応')
    f.path('M876,384 V438', dashed=True); f.note(868, 420, 'アダプターで差し込む', 'end')
    f.bound(24, 440, 952, 56, '外の文脈：製品（Git・コーディングエージェント・検査器・テンプレートの処理系）')
    return f.write('layers')

def model():
    f = Fig(1000, 470, '集約・値・サービス（突き合わせで確かめたもの）')
    f.area(10, 10, 480, 450, 'conformance-domain（契約への適合）')
    f.box(30, 44, 200, 64, ['集約：Concrete', '版は閉じた一覧・名前を足さない'], 'focus')
    f.box(270, 44, 200, 64, ['集約：インスタンス', 'id・導出値・人の欄・指紋'], 'focus')
    f.path('M270,76 H232', dashed=True); f.note(251, 132, 'URN と版で指す', 'middle')
    f.box(30, 160, 440, 70, ['値：指紋・差分・URN と版・インスタンスの id・', '判定の結果（5つの値と数え方）'])
    f.box(30, 270, 440, 56, ['サービス：x-relationship（指す先の実在・参照元を探す）'], 'kind-1')
    f.box(30, 370, 440, 66, ['use_cases に残すもの：検索・参照元を消して回る・', '版の移し方・$schema を決める・衝突を返す'], dashed=True)
    f.area(510, 10, 480, 450, 'semantic-operations-domain（意味の操作）')
    f.box(530, 44, 440, 64, ['集約：SAS が立てたセッション（回は内側）', '型ごとの不変条件・閉じたら足さない'], 'focus')
    f.box(530, 160, 440, 56, ['値：セッションの型（寿命・文脈）・能力の宣言', '（左の値を使う）'])
    f.box(530, 270, 440, 56, ['サービス：再開の決まり（5段）・やり直しの決まり'], 'kind-1')
    f.box(530, 370, 440, 66, ['use_cases に残すもの：手順のプロンプトを組む・', 'エージェントを呼ぶ'], dashed=True)
    f.path('M530,188 H472', dashed=True)
    return f.write('model')

def rows_map(name, label, lh, rh, rows, W=1000):
    n = len(rows); Hh = 70 + n * 66 + 10
    f = Fig(W, Hh, label)
    f.area(10, 10, 400, Hh - 20, lh); f.area(590, 10, 400, Hh - 20, rh)
    for i, (l, r, rrole) in enumerate(rows):
        y = 44 + i * 66
        f.box(30, y, 360, 50, [l])
        f.box(610, y, 360, 50, [r], rrole if rrole != 'muted' else '', dashed=(rrole == 'muted'))
        f.path(f'M390,{y+25} H608')
    return f.write(name)

def datamap():
    return rows_map('datamap', 'データとドメインモデルの対応', '手元の DB と Git', 'ドメインモデル・行', [
        ('Git のファイル：Concrete', '集約：Concrete', 'focus'),
        ('Git のファイル：インスタンス', '集約：インスタンス', 'focus'),
        ('sessions（kind = sas）＋ session_turns', '集約：SAS が立てたセッション（回は内側）', 'focus'),
        ('sessions（kind = caller）', '操作の記録の行（集約にしない）', ''),
        ('operations・field_changes・rule_results', '操作の記録の行（追記だけ。差分と判定の結果は値で読む）', '')])

def gone():
    return rows_map('gone', '外したものと引き継ぐ先', '外す（いまの schemas/v1）', '引き継ぐ先', [
        ('judgment', 'rule_results・rule-result', 'focus'),
        ('change', 'operations・field_changes・diff', 'focus'),
        ('change-summary', '使う側（まとめ・承認・コミット）', 'muted'),
        ('call', 'procedure-prompt・session-request', 'focus'),
        ('output-judging・output-working', 'agent-answer', 'focus'),
        ('vocab-meta', 'vocabulary（改名・x-view を足す）', 'focus')])

def bnd():
    f = Fig(1000, 490, 'スキーマの境界')
    f.box(40, 30, 260, 56, ['Concrete を書く人・AI'])
    f.box(370, 30, 260, 56, ['呼び出し元（人・AI・CI）'])
    f.box(700, 30, 260, 56, ['使う側のファイル', '（設定・接続の定義・デザインシステム）'])
    f.box(370, 170, 260, 56, ['SAS（CLI）'], 'focus')
    f.path('M500,86 V168'); f.note(508, 118, 'operation-request ⇄ operation-result')
    f.path('M170,86 V198 H368'); f.note(178, 132, 'Concrete を書く型')
    f.path('M830,86 V198 H632'); f.note(822, 152, '差し込む物の契約', 'end')
    f.box(40, 290, 260, 50, ['手元の DB（5表・公開しない）'], dashed=True)
    f.path('M440,226 V315 H302', dashed=True); f.note(310, 306, 'スキーマにしない')
    f.box(560, 290, 200, 50, ['アダプター'])
    f.path('M600,226 V288'); f.note(608, 262, 'session-request ⇄ reply')
    f.box(560, 410, 200, 56, ['SAS が立てたセッションの', 'エージェント'])
    f.path('M660,340 V408'); f.note(668, 380, 'procedure-prompt ⇄ agent-answer')
    return f.write('bnd')

def hexf():
    f = Fig(1300, 760, '1つのヘキサゴナル：内側に domain と4つの application、縁にポート、外にアダプター')
    f.o.append('<path class="area" d="M300,380 L440,60 L860,60 L1000,380 L860,700 L440,700 Z"/>')
    f.note(650, 84, '文脈「契約とインスタンス」（1つのヘキサゴナル）', 'middle', True)
    f.box(30, 350, 200, 70, ['inbound/cli（sas コマンド）', '入口・main.rs で組み立てる'], 'focus')
    apps = [(110, 'semantic-operations', 'application'), (200, 'conformance', 'application'), (520, 'operation-recording', 'application（スクリプト）'), (610, 'rendering', 'application（スクリプト）')]
    for y, a, b in apps: f.box(440, y, 240, 56, [a, b])
    f.box(470, 300, 210, 110, ['domain（2つの crate）', 'conformance-domain', 'semantic-operations-domain', '集約・値・サービス'], 'focus')
    f.path('M680,150 H700 V340 H682'); f.path('M600,256 V298')
    f.path('M560,520 V412', dashed=True); f.note(572, 470, '値だけ')
    f.path('M230,385 H320 V138 H438'); f.path('M320,228 H438'); f.path('M320,385 V638 H438'); f.path('M320,548 H438')
    PS = [(100, 90, ['ポート（意味の操作）', 'agent_runtime', 'keep_prompt', 'agent_session_store']), (210, 90, ['ポート（契約への適合）', 'document_store', 'version_reader', 'schema_checker']), (490, 70, ['ポート（操作の記録）', 'identify_caller', 'operation_log']), (600, 46, ['ポート（描画）', 'template_renderer'])]
    for y, h, l in PS: f.box(712, y, 166, h, l)
    for (y, _, _), (py, ph, _) in zip(apps, PS): f.path(f'M680,{y+28} H710')
    AD = [(78, 'outbound/agents'), (168, 'outbound/files'), (258, 'outbound/sqlite'), (348, 'outbound/git（読むだけ）'), (428, 'outbound/jsonschema'), (598, 'outbound/templates')]
    for y, l in AD: f.box(1080, y, 200, 44, [l])
    f.note(1180, 60, 'アダプター（outbound）', 'middle', True)
    C = [(100, 1060, 125), (100, 1048, 515), (190, 1036, 145), (190, 1024, 235), (280, 1012, 165), (280, 1000, 530), (370, 988, 255), (450, 976, 275), (620, 964, 623)]
    for ay, lx, ty in C: f.path(f'M1078,{ay} H{lx} V{ty} H880')
    f.note(1180, 690, 'アダプターがポートを実装する（外 → 内）', 'middle')
    f.note(1180, 708, 'domain は何にも依存しない', 'middle')
    return f.write('hex', 1100)

def cratesf():
    f = Fig(1240, 720, 'crate の層と、依存してよい向き')
    f.area(10, 10, 300, 700, 'adapters')
    f.area(350, 10, 520, 700, 'application')
    f.area(910, 10, 320, 700, 'domain')
    f.box(30, 46, 260, 56, ['adapters-inbound-cli', '（bin・組み立て）'], 'focus')
    outs = ['agents', 'files', 'git', 'jsonschema', 'sqlite', 'templates']
    for i, n in enumerate(outs): f.box(30, 136 + i * 92, 260, 56, ['adapters-outbound-' + n])
    f.path('M30,86 H20 V624', arrow=False)
    for i in range(6): f.path(f'M20,{164 + i * 92} H28')
    apps = [(60, 'semantic-operations-application'), (220, 'conformance-application'), (400, 'operation-recording-application'), (560, 'rendering-application')]
    for y, n in apps: f.box(390, y, 290, 56, [n], 'focus' if y < 300 else '')
    f.box(940, 60, 270, 56, ['semantic-operations-domain'], 'focus')
    f.box(940, 220, 270, 56, ['conformance-domain'], 'focus')
    f.path('M680,88 H938'); f.path('M680,248 H938'); f.path('M1075,116 V218')
    f.path('M535,116 V218'); f.path('M535,276 V398'); f.path('M680,588 H720 V262 H682', dashed=True)
    f.path('M680,428 H760 V264 H938', dashed=True); f.note(768, 360, '値だけ')
    f.path('M290,164 H330', arrow=False)
    for i in range(1, 6): f.path(f'M290,{164 + i * 92} H330', arrow=False)
    f.path('M330,164 V624', arrow=False)
    for y, _ in apps: f.path(f'M330,{y + 28} H388')
    f.note(1075, 330, 'conformance-domain は何にも依存しない', 'middle', True)
    f.note(160, 690, 'cli だけが outbound に依存', 'middle')
    f.note(610, 670, '線は依存してよい向き（意味の操作 → 契約への適合 → 操作の記録）。', 'middle')
    f.note(610, 688, 'outbound は application のポートを実装し、domain の型は再公開から使う', 'middle')
    return f.write('crates', 1000)


def stages():
    f = Fig(1000, 440, '探索の4つの段階と、引き上げの表')
    ST = [['1 境界', '区切られた文脈と業務領域'], ['2 ユースケース', 'アクター・事前状態・保証'], ['3 トランザクションスクリプト', '手続きとルールをそのまま書く'], ['4 引き上げ', '何になるかを見分ける']]
    for i, l in enumerate(ST):
        f.box(20 + i * 245, 30, 220, 64, l, 'focus' if i == 3 else '')
        if i < 3: f.path(f'M{240 + i * 245},62 H{263 + i * 245}')
    f.area(20, 120, 960, 310, '引き上げの表（段階4）')
    RW = [('いつも同時に守らないといけない条件', '集約', 'focus'), ('意味を持つ値と、その値の規則', '値オブジェクト', 'focus'), ('いくつかのものにまたがる判定や計算', 'ドメインサービス', 'kind-1'), ('データの出し入れだけで、ルールが無い', 'スクリプトのまま', ''), ('見た目は同じだが、意味や変わる理由が違う', '別のものとして分ける', '')]
    for i, (a, b, r) in enumerate(RW):
        y = 156 + i * 52
        f.box(50, y, 500, 40, [a]); f.box(640, y, 300, 40, [b], r)
        f.path(f'M550,{y+20} H638')
    return f.write('stages', 760)
