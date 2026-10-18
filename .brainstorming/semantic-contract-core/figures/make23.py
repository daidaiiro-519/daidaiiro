# 第68回の図：Q3 をドメインモデルの案（measured/domain-model-v2.json）に合わせて作り直す。
import json, runpy
G = runpy.run_path('make18.py', run_name='lib')['grouped']
d = json.load(open('../measured/domain-model-v2.json'))

L = [['inbound adapter', 'CLI ・ MCP ・ CI ・ 人の操作の入口（画面）', '駆動する側のポートを呼ぶ。判定するは人の入口だけ']] \
  + [['アプリケーション層', p[0].split('（')[0], '駆動する側のポート（' + ' ・ '.join(p[1]) + '）'] for p in d['ports']['inbound']] \
  + [['ドメイン層', '集約', ' ・ '.join(a['name'] for a in d['aggregates'])],
     ['ドメイン層', 'ドメインサービス', ' ・ '.join(s[0] for s in d['domain_services'])],
     ['ドメイン層', '腐敗防止層', ' ・ '.join(s[0] for s in d['acl'])]] \
  + [['駆動される側のポート', p[0].split('（')[0], p[1]] for p in d['ports']['outbound']] \
  + [['outbound adapter', 'ファイル ・ Git ・ AI の CLI ・ 外のツール ・ HTML', '駆動される側のポートを満たす。技術はここだけに置く']]
G('q3-layers', 'エンジンの層：ポートとアダプター', ['層', '名前', '中身'], [190, 300, 470], L, group_col=0)

CMD = [
 ['意味を宣言する', 'Concrete（文脈）', 'Scaffold する ・ Concrete を照らす ・ 公開された言葉の版を出す', '契約を読む ・ 書く'],
 ['instance を書く', 'instance', 'Resolver ・ Validator ・ 取り込む', '契約を読む ・ instance を読む ・ 書く'],
 ['検証する ・ 変更を見せる', '判定の記録（AI と検証役の結果を足す）', 'Validator ・ Resolver ・ 状態を導く ・ 変更を見せる ・ 例外をまとめる', '契約 ・ instance ・ 判定の記録を読む ・ 変更の前後を受け取る ・ rule の判定を頼む ・ 外のツールを呼ぶ ・ tool permissions を読む'],
 ['判定する', '判定の記録（人）', '状態を導く', '判定の記録を読む ・ 書く'],
 ['探して取り出す ・ 案内する', '──', '意味で探して取り出す ・ Resolver', '契約 ・ instance ・ 判定の記録を読む'],
 ['描画する', '──', 'Renderer ・ 状態を導く', '契約 ・ instance ・ 判定の記録を読む ・ 描画したものを書き出す'],
 ['Abstract の版を出す ・ 移す', 'Semantic Abstract Schema（Core）・ Concrete', '移す ・ 公開された言葉の版を出す', '契約を読む ・ 書く'],
 ['経緯をたどる', '──', '──', '変更の前後を受け取る ・ 判定の記録を読む'],
]
G('q3-commands', '駆動する側のポートごとの、変える集約 ・ 使うドメインサービス ・ 呼ぶ駆動される側のポート', ['ポート', '変える集約', 'ドメインサービス', '呼ぶポート'], [190, 200, 280, 330], CMD, group_col=0)

V = [
 ['機械', 'JSON Schema の実装（形と値）', 'verified ・ violated'],
 ['機械', 'Core の x-（x-ref ・ x-derive など）', 'verified ・ violated。参照先が無ければ基盤が violated'],
 ['AI', 'Agents 契約で呼ぶ AI（Core の rule）', 'verified ・ violated ・ unverified（根拠が無い ・ 答えが割れた）'],
 ['検証役', 'Concrete が宣言した外のツール ・ AI（Concrete が足した x-）', '5つの値と根拠。意味は Concrete が持ち、Core はまとめるだけ'],
 ['人', '判定の記録（変更に入れる）', 'verified ・ violated（判定した者＝人）'],
 ['どれでもない', 'validator ・ 検証役が無い', 'unsupported'],
 ['どれでもない', '時間切れ ・ 読めない応答', 'error'],
]
G('q3-verifiers', 'validator ・ 検証役と、返す結果（どれも5つの値のどれか1つと根拠）', ['種類', 'validator ・ 検証役', '返す結果'], [140, 380, 440], V, group_col=0)
