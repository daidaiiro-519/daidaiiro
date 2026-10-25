"""正式反映する差分と判断理由からACDR入力を作る。"""
from pathlib import Path
import difflib
import json

ROOT = next(p for p in Path(__file__).resolve().parents if (p / 'CLAUDE.md').exists())
WORK = Path(__file__).resolve().parents[1] / 'workspace'
STAGE = WORK / 'promotion-20261025'
record = ROOT / '.acdr/0205-三教材共通導入の正式版改訂'
manifest = json.loads((STAGE / 'manifest.json').read_text())
reasons = {
 'lesson-01-lead.json': '確認済みの表紙と本文9枚の見出し・原稿へ改訂し、開発全体の改善から三教材へつなぐ。',
 'ledes.json': '確認済みの導入文を本文9枚と対応させ、不要になった旧10枚目の項目を外す。',
 'build_lessons.py': '表紙の専用入力を使用し、表紙の原稿を台本へ含める。図の対応表は本文9枚へ合わせる。',
 'build_narration.py': '次回の音声制作で表紙の確認済み原稿を使用し、題名だけの読み上げへ戻らないようにする。',
 'common_intro_visuals.py': '本文と表紙の確認済み図を同じ表示classで読み込む。',
 'words.json': '既存のタスク説明の例外と同じ用途で、担当する変更の説明を指す二つの表現を許容する。',
 'theme.json': 'レビューで確認済みのwarm-paperの図の配色を使用する。',
 'index.html': '最新のスライドと原稿へリンクし、動画と音声が改訂前の内容であることを明示する。',
 'README.md': '三教材共通の導入の位置づけと全10枚、入力と出力、旧音声・動画の状態を明示する。',
}
def reason(entry):
    name = Path(entry['target']).name
    if entry['staged'] is None:
        return '本文を9枚へ再構成したため、旧10枚目の図を外す。'
    return reasons.get(name, '同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。')
report = STAGE / 'promotion-plan.md'
lines = ['# 共通の「はじめに」の正式反映', '', '利用者が2026年10月25日に「この内容で正本化」と承認した、表紙1枚と本文9枚を正本へ反映する。対象は共通の「はじめに」のスライドと原稿であり、教材1本編、企画書、オリエンテーション、音声・動画の制作は含まない。', '', '現行の正本は参照漏れを扱う旧題材である。確認済みの題材は、要件の解釈、不具合と再作業、動作修正後に残る設計、追加変更で広がり得る確認範囲から三教材の学びへつなぐ。', '', '## 反映する差分', '', '| 対象 | 理由 |', '|---|---|']
for e in manifest['files']:
    lines.append(f"| `{e['target']}` | {reason(e)} |")
lines += ['', '## 確認したこと', '', '表紙と本文の見出し・導入文・原稿、および本文9枚の図がレビュー入力と一致することを照合した。表紙を含む音声制作用の入力も10枚となり、表紙の専用原稿を使用する。図の配色は確認済みテーマを使用する。', '', '作業コピーから全10枚を描画し、見出しと図のはみ出しを検査した。旧音声・動画はそのままであり、正式反映後も改訂前と明記する。新しい動画の制作は別工程とする。', '', '## 文書の10判定', '', '1. 指し先：共通導入の入力と出力を明示。', '2. 型：変更するファイルと理由を表で表示。', '3. 数：表紙1枚と本文9枚の全10枚を照合。', '4. 見出し：正式反映の対象を記載。', '5. 結論：利用者が承認した反映対象を先に説明。', '6. 形：既存の線画とwarm-paperのテーマを使用。', '7. 用語：要件・モデル・設計・AI作業の役割を区別。', '8. 読み手：旧動画と新しいスライドを混同させない。', '9. 量：音声や動画の制作は別工程と明示。', '10. 比喩：承認済み入力と正式反映の差分を直接記述。', '']
report.write_text('\n'.join(lines))
docs = [dict(key='scope', tab='反映する差分', file=str(report.relative_to(ROOT)), marks=[])]
for i, e in enumerate(manifest['files']):
    if not e['staged']:
        continue
    p = ROOT / e['staged']
    if p.suffix not in ['.json', '.py', '.md']:
        continue
    before = (ROOT / e['target']).read_text() if (ROOT / e['target']).exists() else ''
    after = p.read_text()
    old, new = before.splitlines(), after.splitlines()
    marks = []
    for group in difflib.SequenceMatcher(None, old, new).get_grouped_opcodes(3):
        changed = [(tag,a,b,c,d) for tag,a,b,c,d in group if tag != 'equal']
        _,aa,bb,cc,dd = changed[0]
        candidates = new[cc:dd] or new[max(0,cc-1):cc+1]
        find = next((x.strip() for x in candidates if x.strip()), '')
        if p.suffix == '.md' and find.startswith('|'):
            find = find.strip('|').split('|')[-1].strip()
        if find:
            marks.append(dict(find=find, before='\n'.join(old[aa:bb]) or '（新規追加）', why=reason(e)))
    docs.append(dict(key=f'file-{i}', tab=Path(e['target']).name, file=e['staged'], before=before, marks=marks))
# 承認済みの見た目と原稿も、ACDRの同じページから確認できるようにする。
for key, tab, staged, target in [
 ('slides','スライド全10枚',STAGE/'material-1/out/slides/lesson-01-lead.html',ROOT/'docs/workshops/ai-workshop/material-1/out/slides/lesson-01-lead.html'),
 ('script','読み上げ原稿',STAGE/'material-1/out/scripts/lesson-01-lead-script.md',ROOT/'docs/workshops/ai-workshop/material-1/out/scripts/lesson-01-lead-script.md')]:
    docs.append(dict(key=key,tab=tab,file=str(staged.relative_to(ROOT)),before=target.read_text(),marks=[]))
record_data = dict(no='ACDR 0205',title='三教材共通導入の正式版改訂',date='2026-10-25',status='proposed',
 decision='三教材共通の「はじめに」を、確認済みの全10枚に改訂する。',
 why='正本に残る旧題材を、要件と設計の問題から三教材の学びへつなぐ確認済みの内容へ改める。',
 shift=[dict(what='共通導入の題材',**{'from':'関連仕様の参照漏れを中心にした説明。','to':'要件の解釈、不具合修正後に残る設計、追加変更の確認負担から三教材へつなぐ説明。'}),dict(what='表紙と本文',**{'from':'自動生成した表紙と旧本文10枚。','to':'確認済みの専用表紙と本文9枚。'}),dict(what='再生成する入力',**{'from':'表紙の原稿を独立して記録していない。','to':'表紙・本文・導入文・図を入力から再生成できる。'})],
 alternatives=[dict(option='旧題材を正本へ残す。',why_not='確認済みのストーリーや教材2・3の位置づけと食い違う。'),dict(option='レビュー用HTMLだけを正式版として複製する。',why_not='教材の生成元と原稿が古いままになり、次の生成で旧版へ戻る。'),dict(option='音声と動画も同時に作り直す。',why_not='今回の正式反映と動画制作を分け、先にスライドと原稿の正本を確定する。')],supersedes='',docs=docs)
(record/'acdr.json').write_text(json.dumps(record_data,ensure_ascii=False,indent=2)+'\n')
