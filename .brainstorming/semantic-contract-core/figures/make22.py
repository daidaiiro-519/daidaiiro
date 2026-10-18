# 第67回の図：使う場面と業務領域から導き直したドメインモデル（measured/domain-model-v2.json）。
import json, runpy
G = runpy.run_path('make18.py', run_name='lib')['grouped']
d = json.load(open('../measured/domain-model-v2.json'))

G('q1-boundary', '境界：何を誰が持ち、誰が変えるか', ['', 'もの', '持ち主', '変える者'], [40, 470, 230, 220],
  [[str(n + 1)] + r for n, r in enumerate(d['boundary'])], group_col=0)

E = d['extension_contract']
rows = [['責任', a, f'Core：{b} ／ Concrete：{c}'] for a, b, c in E['responsibility']] + [['作法（Q7 で詰める）', a, b] for a, b in E['manners']] + [['拡張の仕方', a, b] for a, b in E['parts']] + [['言語に依らない', str(n + 1), t] for n, t in enumerate(E['language'])] \
     + [['外のツール', str(n + 1), t] for n, t in enumerate(E['external_tool'])] \
     + [['保存', str(n + 1), t] for n, t in enumerate(E.get('storage', []))]
G('q1-extension', '拡張の契約：Core は作法だけを持ち、足した x- の意味の検証は Concrete が持つ', ['', '項目', '中身'], [140, 250, 560], rows, group_col=0)

M = [['集約', a['name'], f"{a['holder']}。{a['members']}", a['subdomain']] for a in d['aggregates']] \
  + [['値オブジェクト', a, b, ''] for a, b in d['value_objects']] \
  + [['ドメインサービス', s[0], s[1], s[2]] for s in d['domain_services']] \
  + [['腐敗防止層', s[0], s[1], s[2]] for s in d['acl']]
G('q1-model', 'ドメインモデル：集約 ・ 値オブジェクト ・ ドメインサービス ・ 腐敗防止層', ['種類', '名前', '中身', '業務領域'], [130, 200, 470, 170], M, group_col=0)

P = [['駆動する側（外に差し出す口）', p[0], p[2], ' ・ '.join(p[1])] for p in d['ports']['inbound']] \
  + [['駆動される側（中が外に求める口）', p[0], p[1], p[2]] for p in d['ports']['outbound']]
G('q1-ports', 'ポートとアダプター：駆動する側は見せるアダプター、駆動される側は満たすアダプター', ['', 'ポート', 'アダプター', '使う場面 ・ 業務領域'], [150, 330, 270, 210], P, group_col=0)

G('q1-invariants', f'不変条件{len(d["invariants"])}行：対象ごとに固め、導いた使う場面を付けた表', ['#', '対象', '不変条件', '使う場面'], [50, 110, 640, 150],
  [[i[0], i[1], i[2], ' ・ '.join(i[3])] for i in d['invariants']], group_col=1)

cov = []
for k, kind in d['subdomains'].items():
    own = [a['name'] for a in d['aggregates'] if a['subdomain'] == k] + [s[0] for s in d['domain_services'] + d['acl'] if s[2] == k] \
        + [p[0].split('（')[0] for p in d['ports']['outbound'] if p[2] == k]
    cov.append([kind, k, ' ・ '.join(own)])
G('q1-coverage', '業務領域ごとの担い手：どの業務領域にも担い手がある', ['分類', '業務領域', '担い手（集約 ・ ドメインサービス ・ ポート）'], [70, 230, 660], cov, group_col=0)

UC = [[u[0], u[1], u[2], u[3], u[4]] for u in d['use_cases']]
G('q8-usecases', '使う場面：主体 ・ 目的 ・ 成功したときの保証 ・ 最低限の保証', ['#', '主体', '目的', '成功したとき', '最低限'], [60, 140, 240, 300, 300], UC, group_col=0)
G('q8-removed', '外した使う場面と理由', ['#', '場面', '外した理由'], [60, 260, 640], [list(r) for r in d['removed_use_cases']], group_col=0)
