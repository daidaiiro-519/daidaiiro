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
    f.box(30, 40, 290, 56, ['オーケストレーター：', '目的のセッションを立てて任せる'])
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
    f.box(30, 450, 290, 64, ['記録：誰が何で動いたかと', '書いた変更・検証'], 'focus')
    f.box(355, 450, 290, 64, ['契約：手順を返す・差分', '（Git は読むだけ）・検査・描画'], 'focus')
    f.box(680, 450, 290, 64, ['意味としての目的：rule.why・', 'reading.when・description・意図'], 'kind-1')
    f.area(10, 580, 980, 110, '一般の製品')
    f.path('M420,514 V556 H260 V618', dashed=True); f.note(268, 574, '読むだけ')
    f.path('M580,514 V556 H825 V618', dashed=True); f.note(833, 574, '渡す')
    f.box(30, 620, 290, 56, ['製品：Git', '（コミット済みの版を読むだけ）'])
    f.box(355, 620, 290, 56, ['製品：コーディングエージェント', '（sas を打つ。SAS は起動しない）'])
    f.box(680, 620, 290, 56, ['製品：2020-12 のバリデーター・', 'テンプレートの処理系'])
    return f.write('overview')

def layers():
    f = Fig(1000, 520, '事業領域・業務領域・区切られた文脈')
    f.area(10, 10, 980, 196, '問題の側：業務領域（見つける）')
    f.box(250, 32, 500, 56, ['事業領域：AI のエージェントが扱う構造化データを、', 'スキーマに従って書き、検証すること'], 'kind-1')
    xs = [20, 180, 340, 500, 660, 820]
    for x in [175, 415, 575, 735, 895]: f.path(f'M500,88 V108 H{x} V128', arrow=True)
    f.note(512, 104, '分解')
    f.box(20, 130, 310, 56, ['書いて検証する（中核）', 'ドメインモデル'], 'focus')
    f.box(340, 130, 150, 56, ['検索（補完）', 'スクリプト'])
    f.box(500, 130, 150, 56, ['記録（補完）', 'スクリプト'])
    f.box(660, 130, 150, 56, ['描画（補完）', 'スクリプト'])
    f.box(820, 130, 150, 56, ['一般：版の管理・', '検証・描画の部品'], dashed=True)
    f.area(10, 250, 980, 260, '解決の側（設計する）', tx=136)
    f.bound(24, 280, 952, 124, '文脈「スキーマとインスタンス」（1つ）', tx=136)
    labs = [['writing/', '書くのモジュール'], ['validation/', '検証のモジュール'], ['search/', '検索のモジュール'], ['recording/', '記録のモジュール'], ['rendering/', '描画のモジュール'], ['published-language/', '公開された言葉']]
    for x, l in zip(xs, labs):
        if x < 800: f.path(f'M{x+75},186 V280', dashed=True, arrow=False)
        f.path(f'M{x+75},306 V326', dashed=True)
        f.box(x, 328, 150, 56, l, 'focus' if x < 300 else '')
    f.note(108, 230, 'モジュール')
    f.path('M895,384 V438', dashed=True); f.note(885, 420, 'アダプターとして渡す', 'end')
    f.bound(24, 440, 952, 56, '外の文脈：製品（Git・バリデーター・テンプレートの処理系）と、sas を打つコーディングエージェント')
    return f.write('layers')

def model():
    f = Fig(1000, 470, '集約・値・サービス（突き合わせで確かめたもの）')
    f.area(10, 10, 480, 450, 'writing-domain（書くのモジュール）')
    f.box(30, 44, 200, 64, ['集約：Concrete', '版は閉じた一覧・公開した版'], 'focus')
    f.box(270, 44, 200, 64, ['集約：インスタンス', '導出値・人の欄・ハッシュ'], 'focus')
    f.path('M270,76 H232', dashed=True); f.note(251, 132, 'URN と版で指す', 'middle')
    f.box(30, 160, 440, 70, ['値：ハッシュ・差分（欄ごとの前後）・', '書く条件の外れ'])
    f.box(30, 270, 440, 66, ['サービスは持たない', '（参照を見分けず、文書をまたぐ計算が無い）'], dashed=True)
    f.box(30, 370, 440, 66, ['use_cases に残すもの：取り込みの前の値・', '衝突を返す・従うインスタンスがあれば止める'], dashed=True)
    f.area(510, 10, 480, 450, 'validation-domain（検証のモジュール）')
    f.box(530, 44, 440, 64, ['集約は持たない', '← 書くモジュールの集約を読んで rule に照らす'], dashed=True)
    f.box(530, 160, 440, 56, ['値：判定の5つの区分・rule の結果・誰が判定したか（by）'])
    f.box(530, 270, 440, 66, ['サービス：当たる rule・', 'Concrete に当たる Abstract の rule'], 'kind-1')
    f.box(530, 370, 440, 66, ['意味（x-prompt の rule）は同じ検証の中。', '意味を読ませるポートは application（ステップ6）'], dashed=True)
    f.path('M530,76 H472')
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
        ('sessions（打ったセッション。誰がどのツールで動いたか）', '操作の記録の行（集約にしない。追記だけ）', ''),
        ('changes・field_changes', '操作の記録の行（書いた変更。差分は値で読む）', ''),
        ('validations・rule_results', '操作の記録の行（検証。判定の結果は値で読む）', '')])

def gone():
    return rows_map('gone', '外したものと引き継ぐ先', '外す（版15 の schemas/v1）', '引き継ぐ先', [
        ('judgment', 'rule_results・rule-result', 'focus'),
        ('change', 'changes・field_changes・diff', 'focus'),
        ('change-summary', '使う側（まとめ・承認・コミット）', 'muted'),
        ('call', 'procedure（値を渡す command を持つ）', 'focus'),
        ('output-judging・output-working', 'agent-answer', 'focus'),
        ('vocab-meta', 'vocabulary（改名・x-view を足す）', 'focus')])

def bnd():
    f = Fig(1000, 370, 'スキーマの境界')
    f.box(40, 30, 260, 56, ['Concrete を書く人・AI'])
    f.box(370, 30, 260, 56, ['呼び出し元（人・AI・CI）'])
    f.box(700, 30, 260, 56, ['使う側のファイル', '（設定・デザインシステム）'])
    f.box(370, 170, 260, 56, ['SAS（CLI）'], 'focus')
    f.path('M500,86 V168'); f.note(508, 112, 'CLI の引数 ・ 値 ・ 判定'); f.note(508, 130, '⇄ command-result ・ procedure')
    f.path('M170,86 V198 H368'); f.note(178, 132, 'Concrete を書く型')
    f.path('M830,86 V198 H632'); f.note(822, 152, '受け取りの型', 'end')
    f.box(40, 290, 260, 50, ['手元の DB（5表・公開しない）'], dashed=True)
    f.path('M440,226 V315 H302', dashed=True); f.note(310, 306, 'スキーマにしない')
    return f.write('bnd')

def hexf():
    f = Fig(1300, 780, 'ポートとアダプター：内側に業務領域を作るモジュール、縁にポート、外にアダプター')
    f.o.append('<path class="area" d="M300,390 L440,60 L860,60 L1000,390 L860,720 L440,720 Z"/>')
    f.note(650, 84, '文脈「スキーマとインスタンス」', 'middle', True)
    f.box(30, 350, 200, 82, ['inbound/cli（sas コマンド）', '入口・main.rs が', 'コンポジションルート'], 'focus')
    apps = [(100, 'writing', 'application', 'focus'), (180, 'validation', 'application', 'focus'), (470, 'recording', 'application（スクリプト）', ''), (550, 'rendering', 'application（ステップ5）', ''), (630, 'search', 'application（ステップ8）', '')]
    for y, a, b, r in apps: f.box(440, y, 230, 56, [a, b], r)
    f.box(470, 262, 190, 50, ['validation-domain', '判定の値・サービス'], 'focus')
    f.box(470, 360, 190, 56, ['writing-domain', '集約・値'], 'focus')
    f.path('M555,156 V178'); f.path('M555,236 V260'); f.path('M555,312 V358')
    f.path('M670,128 H690 V388 H662')
    f.path('M440,208 H420 V498 H438', dashed=True); f.note(412, 470, '記録', 'end')
    f.path('M230,391 H320 V128 H438'); f.path('M320,208 H438'); f.path('M320,391 V658 H438'); f.path('M320,498 H438'); f.path('M320,578 H438')
    PS = [(102, 46, ['ポート（書く）', 'commit_reader']), (170, 66, ['ポート（検証）', 'document_store', 'schema_validator']), (462, 90, ['ポート（記録）', 'identify_caller', 'operation_log', 'operation_reader']), (562, 46, ['ポート（描画）', 'template_renderer'])]
    for y, h, l in PS: f.box(704, y, 166, h, l)
    for y in [100, 180, 470, 550]: f.path(f'M670,{y+28} H702')
    AD = [(150, 'outbound/filesystem'), (220, 'outbound/git（読むだけ）'), (290, 'outbound/jsonschema'), (470, 'outbound/sqlite'), (560, 'outbound/templates（5）')]
    for y, l in AD: f.box(1080, y, 200, 44, [l])
    f.note(1180, 60, 'アダプター（outbound）', 'middle', True)
    C = [(172, 1045, 216), (242, 1030, 125), (312, 1015, 230), (492, 1000, 505), (582, 985, 579)]
    for ay, lx, ty in C: f.path(f'M1078,{ay} H{lx} V{ty} H872')
    f.note(1180, 690, 'アダプターがポートを実装する（外 → 内）', 'middle')
    f.note(1180, 708, 'writing-domain は何にも依存しない', 'middle')
    return f.write('hex', 1100)

def cratesf():
    f = Fig(1240, 740, 'crate の層と、依存してよい向き')
    f.area(10, 10, 300, 720, 'adapters')
    f.area(350, 10, 520, 720, 'application')
    f.area(910, 10, 320, 720, 'domain')
    f.box(30, 46, 260, 56, ['inbound-cli', '（bin・コンポジションルート）'], 'focus')
    outs = ['filesystem', 'git', 'jsonschema', 'sqlite', 'templates', 'javascript']
    for i, n in enumerate(outs): f.box(30, 136 + i * 92, 260, 56, ['outbound-' + n], dashed=n.endswith('）'))
    f.path('M30,86 H20 V624', arrow=False)
    for i in range(6): f.path(f'M20,{164 + i * 92} H28')
    apps = [(60, 'writing-application', 'focus', False), (170, 'validation-application', 'focus', False), (400, 'recording-application', '', False), (500, 'rendering-application', '', False), (600, 'search-application', '', False)]
    for y, n, r, d in apps: f.box(390, y, 290, 56, [n], r, dashed=d)
    f.box(940, 60, 270, 56, ['writing-domain'], 'focus')
    f.box(940, 170, 270, 56, ['validation-domain'], 'focus')
    f.path('M680,88 H938'); f.path('M680,198 H938'); f.path('M1075,170 V118')
    f.path('M535,116 V168'); f.path('M535,226 V398')
    f.path('M680,528 H720 V210 H682'); f.path('M680,628 H720 V528', arrow=False)
    f.path('M290,164 H330', arrow=False)
    for i in range(1, 6): f.path(f'M290,{164 + i * 92} H330', arrow=False)
    f.path('M330,164 V624', arrow=False)
    for y, _, _, _ in apps: f.path(f'M330,{y + 28} H388')
    f.note(1075, 280, 'writing-domain は何にも依存しない', 'middle', True)
    f.note(1075, 300, 'validation-domain は集約を読む', 'middle')
    f.note(160, 700, 'cli だけが outbound に依存', 'middle')
    f.note(610, 676, '線は依存してよい向き：書く → 検証 → 記録、検証の domain → 書くの domain', 'middle')
    f.note(610, 694, 'いま15。描画 ・ 検索 → 検証', 'middle')
    f.note(610, 712, 'outbound は application のポートを実装する', 'middle')
    return f.write('crates', 1000)

def stages():
    f = Fig(1000, 440, '探索の4つの段階と、引き上げの表')
    ST = [['1 境界', '区切られた文脈と業務領域'], ['2 ユースケース', 'アクター・事前状態・保証'], ['3 トランザクションスクリプト', '手続きとルールをそのまま書く'], ['4 引き上げ', '何になるかを判別する']]
    for i, l in enumerate(ST):
        f.box(20 + i * 245, 30, 220, 64, l, 'focus' if i == 3 else '')
        if i < 3: f.path(f'M{240 + i * 245},62 H{263 + i * 245}')
    f.area(20, 120, 960, 310, '引き上げの表（段階4）')
    RW = [('いつも同時に守らないといけない条件', '集約', 'focus'), ('意味を持つ値と、その値の規則', '値オブジェクト', 'focus'), ('いくつかのものにまたがる判定や計算', '業務サービス', 'kind-1'), ('データの出し入れだけで、ルールが無い', 'スクリプトのまま', ''), ('見た目は同じだが、意味や変わる理由が違う', '別のものとして分ける', '')]
    for i, (a, b, r) in enumerate(RW):
        y = 156 + i * 52
        f.box(50, y, 500, 40, [a]); f.box(640, y, 300, 40, [b], r)
        f.path(f'M550,{y+20} H638')
    return f.write('stages', 760)
