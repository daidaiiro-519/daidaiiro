"""場面の正本を JSON に書き出す。場面は使う場面の事後条件 ・ 失敗時と不変条件から起こす（1場面に操作1つ）。"""
import json, os
HERE = os.path.dirname(os.path.abspath(__file__))
U = lambda i: f'01JBA0C{i}Q4M6R2T9V5X7Y1A3C5'  # 場面で決める ULID（26文字）
ACDR = 'urn:semantic-agent-schema:acdr:decision-record:v1'
O1 = {'tool': 'claude-code', 'id': 'O1'}; P1 = {'tool': 'claude-code', 'id': 'P1'}
S_O1 = {'session': O1, 'state': 'open', 'last_used_at': '2026-10-24T09:00:00Z'}
S_P1 = {'session': P1, 'purpose_ref': 'ACDR 0197', 'parent': O1, 'state': 'open', 'last_used_at': '2026-10-24T09:50:00Z'}
FILES = {'docs/SKILL.md': '# brainstorming\n\n画面の AI に何を許すかは agents.json で決める。\n',
         'tool/agent.rs': 'fn spawn_agent(cfg: &AgentCfg, allowed: &[Tool]) -> Child {\n    todo!()\n}\n',
         '.acdr/0197-ai-permissions/acdr.json': json.dumps({'$schema': ACDR, 'kind': 'decision-record', 'id': '0197-ai-permissions', 'no': '0197', 'title': '画面の AI の許可', 'status': 'proposed', 'why': '食い違っていた。'}, ensure_ascii=False, indent=1)}
EDIT = {'docs/SKILL.md': '# brainstorming\n\n画面の AI に何を許すかは各 AI ツールの設定ファイルで決める。\n',
        'tool/agent.rs': 'fn spawn_agent(cfg: &AgentCfg) -> Child {\n    todo!()\n}\n'}
GIVEN = lambda **kw: dict(files=FILES, sessions=[S_O1, S_P1], ids=[U(1), U(2), U(3)], **kw)
BOTH = ['git', 'snapshot']
CV = 'urn:semantic-agent-schema:base:change:v1#/$defs/view'
REC = lambda intent='許可の一覧を渡さない', **kw: {'op': 'record_change', 'args': dict(intent=intent, session=P1, **kw)}
JUD = lambda i=1: {'op': 'judge_intent', 'args': {'change_id': U(i)}}
APP = lambda i=1, ref='after1': {'op': 'approve', 'args': {'change_id': U(i)}, 'save_ref': ref}
T = lambda i, at, msg, **kw: dict(id=i, at=at, message=msg, **kw)

changes = [
 dict(id='S-REC-1', title='AI が Markdown とコードを直し、意図だけを渡して記録する', traces=['U5:事後', 'U6:事後', 'I38', 'I37', 'I34'], adapters=BOTH,
      given=GIVEN(pending=EDIT), when=REC(),
      then=[T('E1', '/result/change/items', '前後に出た2つのファイルが変更の項目に入っていない', count=2),
            T('E2', '/result/change/items/0/target/path', '対象がパスで入っていない', equals='docs/SKILL.md'),
            T('E3', '/result/change/purpose_ref', 'セッションの目的が写されていない（I38）', equals='ACDR 0197'),
            T('E4', '/state/index/0/change_id', '手元の索引に足されていない', equals=U(1)),
            T('E5', '/result/change/session', '変更の記録にセッション ID が入っている（I34）', exists=False)],
      result_schema='urn:semantic-agent-schema:base:change:v1', result_schema_at='/change'),
 dict(id='S-REC-2', title='JSON のインスタンスは id と kind と欄で記録する', traces=['U6:事後', 'I31'], adapters=BOTH,
      given=GIVEN(pending={'.acdr/0197-ai-permissions/acdr.json': json.dumps({**json.loads(FILES['.acdr/0197-ai-permissions/acdr.json']), 'why': '食い違っていた。最小権限。'}, ensure_ascii=False, indent=1)}),
      when=REC('least-privilege の根拠を書く'),
      then=[T('E1', '/result/change/items', '欄の単位で1件になっていない', count=1),
            T('E2', '/result/change/items/0/target', '対象が id と kind になっていない', equals={'id': '0197-ai-permissions', 'kind': ACDR}),
            T('E3', '/result/change/items/0/field', '欄が /why になっていない', equals='/why')]),
 dict(id='S-REC-3', title='変わったものが無ければ記録を作らない', traces=['I31'], adapters=BOTH, given=GIVEN(), when=REC(),
      then=[T('E1', '/result/refused', '変更の無い記録を作った', equals='nothing-changed')], unchanged=['records', 'index']),
 dict(id='S-REC-4', title='目的を持たないオーケストレーターは、呼ぶ側から目的を受け取り、索引には足さない', traces=['I38'], adapters=['snapshot'],
      given=GIVEN(pending=EDIT), when={'op': 'record_change', 'args': {'intent': 'CLAUDE.md を直す', 'session': O1, 'purpose_ref': 'ACDR 0190'}},
      then=[T('E1', '/result/change/purpose_ref', '呼ぶ側の目的が使われていない', equals='ACDR 0190'), T('E2', '/state/index', '目的の無いセッションの索引を足した', count=0)]),
 dict(id='S-PEND-1', title='作業を始める前に、意図に入らない未確定の変更を見せる', traces=['U25:失敗時'], adapters=BOTH,
      given=GIVEN(pending={'CLAUDE.md': '# 運用ルール\n'}), when={'op': 'check_pending'},
      then=[T('E1', '/result/exceptions/0/type', '例外の種類が違う', equals='unconfirmed-outside-intent'), T('E2', '/result/exceptions/0/items/0/path', '対象が違う', equals='CLAUDE.md')]),
 dict(id='S-PEND-2', title='その変更を先に確定すれば、未確定の変更は空になる', traces=['U16:事後'], adapters=BOTH,
      given=GIVEN(steps=[{'op': 'edit', 'args': {'CLAUDE.md': '# 運用ルール\n'}}, {'op': 'record_change', 'args': {'intent': '運用ルールを足す', 'purpose_ref': 'ACDR 0190', 'writer_kind': 'human'}},
                         {'op': 'approve', 'args': {'change_id': U(1)}}]),
      when={'op': 'check_pending'}, then=[T('E1', '/result/exceptions', '先に確定したのに残っている', count=0), T('E2', '/state/pending', '未確定の変更が残っている', count=0)]),
 dict(id='S-APP-1', title='本人が承認したら、記録 ・ 判定 ・ 変更を1つの単位で確定する', traces=['U16:事後', 'I16'], adapters=BOTH,
      given=GIVEN(steps=[{'op': 'edit', 'args': EDIT}, REC(), JUD()]), when={'op': 'approve', 'args': {'change_id': U(1)}},
      then=[T('E1', '/result/ref', '確定の単位の参照が返らない', exists=True), T('E2', '/state/pending', '確定したのに未確定の変更が残る', count=0)]),
 dict(id='S-APP-2', title='同じ承認をもう一度しても、失敗ではなく「変わったものが無い」と返る', traces=['U16:失敗時'], adapters=BOTH,
      given=GIVEN(steps=[{'op': 'edit', 'args': EDIT}, REC(), JUD(), APP()]), when={'op': 'approve', 'args': {'change_id': U(1)}},
      then=[T('E1', '/result/nothing_changed', '確定し直しを失敗と取り違えた', equals=True)]),
 dict(id='S-VIEW-1', title='確定した範囲の変更を、目的 → 意図 → 対象（欄 ・ 行）で見せる', traces=['U25:事後', 'I35', 'I36'], adapters=BOTH,
      given=GIVEN(steps=[{'op': 'edit', 'args': EDIT}, REC(), JUD(), APP()]),
      when={'op': 'change_view', 'args': {'before': '$ref:start', 'after': '$ref:after1'}},
      then=[T('E1', '/result/exceptions', '例外が出た', count=0), T('E2', '/result/purposes/0/purpose_ref', '目的が先頭に出ない', equals='ACDR 0197'),
            T('E3', '/result/purposes/0/intents/0/judgment', '本人の承認が出ない', equals='verified'),
            T('E4', '/result/purposes/0/intents/0/targets/0/kind', 'Markdown が行で成型されていない', equals='lines'),
            T('E5', '/result/purposes/0/intents/0/targets/0/lines', 'ファイルの全体が出ていない', count=4)],
      result_schema=CV),
 dict(id='S-VIEW-2', title='記録に入っていない変更は「記録に無い」と出る', traces=['I31', 'U25:失敗時'], adapters=BOTH,
      given=GIVEN(steps=[{'op': 'edit', 'args': EDIT}, REC(), {'op': 'edit', 'args': {'README.md': '使い方\n'}}]),
      when={'op': 'change_view', 'args': {'before': '$ref:start'}},
      then=[T('E1', '/result/exceptions/0/type', '例外の種類が違う', equals='unrecorded'), T('E2', '/result/exceptions/0/items/0/target/path', '対象が違う', equals='README.md')], result_schema=CV),
 dict(id='S-VIEW-3', title='書き換えの無い記録は「変更の無い記録」と出る', traces=['I31'], adapters=BOTH,
      given=GIVEN(steps=[{'op': 'edit', 'args': EDIT}, REC(), {'op': 'edit', 'args': {'tool/agent.rs': FILES['tool/agent.rs']}}]),
      when={'op': 'change_view', 'args': {'before': '$ref:start'}},
      then=[T('E1', '/result/exceptions/0/type', '例外の種類が違う', equals='no-change'), T('E2', '/result/exceptions/0/items/0/target/path', '対象が違う', equals='tool/agent.rs')], result_schema=CV),
 dict(id='S-VIEW-4', title='I33 が violated なら「意図が合わない」と出る', traces=['I33', 'U25:失敗時'], adapters=BOTH,
      given=GIVEN(judge={'許可の一覧を渡さない': 'violated'}, steps=[{'op': 'edit', 'args': EDIT}, REC(), JUD()]),
      when={'op': 'change_view', 'args': {'before': '$ref:start'}},
      then=[T('E1', '/result/exceptions/0/type', '例外の種類が違う', equals='intent-mismatch')], result_schema=CV),
 dict(id='S-VIEW-5', title='手元の索引にあって、この前後に無い記録は「この前後には無い」と出る', traces=['U25:失敗時'], adapters=BOTH,
      given=GIVEN(index=[{'session': P1, 'change_id': U(9)}], steps=[{'op': 'edit', 'args': EDIT}, REC()]),
      when={'op': 'change_view', 'args': {'before': '$ref:start', 'session': P1}},
      then=[T('E1', '/result/exceptions/0/type', '例外の種類が違う', equals='not-in-range'), T('E2', '/result/exceptions/0/items/0/change_id', '記録が違う', equals=U(9))], result_schema=CV),
 dict(id='S-VIEW-6', title='参照に届かなければ、見せたことにしない', traces=['U25:失敗時', 'I42'], adapters=BOTH, given=GIVEN(),
      when={'op': 'change_view', 'args': {'before': 'no-such-ref'}},
      then=[T('E1', '/result/not_shown', '届かない参照で0件を返した', equals='unreachable')]),
 dict(id='S-JUD-1', title='判定の記録は対象 × rule で1件。判定し直すと上書きされる', traces=['I16'], adapters=BOTH,
      given=GIVEN(judge={'許可の一覧を渡さない': 'unverified'}, steps=[{'op': 'edit', 'args': EDIT}, REC(), JUD()]),
      when=JUD(), then=[T('E1', '/state/judgments', '同じ対象 × rule の記録が2件ある', count=1), T('E2', '/result/judgment/question', 'unverified に問いが無い', exists=True)],
      result_schema='urn:semantic-agent-schema:base:judgment:v1', result_schema_at='/judgment'),
 dict(id='S-JUD-2', title='判定した値といまの値が違えば、その判定は効かない', traces=['I16'], adapters=['snapshot'],
      given=GIVEN(steps=[{'op': 'edit', 'args': EDIT}, REC(), JUD(), {'op': 'edit', 'args': {f'.semantic-agent-schema/changes/{U(1)}.json': {'$schema': 'urn:semantic-agent-schema:base:change:v1', 'kind': 'change', 'id': U(1), 'purpose_ref': 'ACDR 0197', 'intent': '別の意図に書き換えた', 'writer_kind': 'ai-agent', 'items': [{'target': {'path': 'docs/SKILL.md'}}]}}}]),
      when={'op': 'judgment_effective', 'args': {'key': f'change/{U(1)}/intent-matches-change'}}, then=[T('E1', '/result/effective', '値が変わったのに効いている', equals=False)]),
]

def ses(sid, title, traces, given, decide, then, slot=None, new=None):
    return dict(id=sid, title=title, traces=traces, adapters=['snapshot'], given=dict(files={}, **given),
                when={'op': 'decide_and_register', 'args': {'decide': decide, 'new': new or {'tool': decide['tool'], 'id': 'NEW'}, 'slot': slot or {k: decide[k] for k in ('purpose_ref', 'parent', 'role') if k in decide}}}, then=then)
AI = lambda **kw: {'resumable': True, 'cacheLifetime': 'PT1H', 'lastUsedAt': '2026-10-24T09:50:00Z', 'historyPercent': 30, **kw}
sessions = [
 ses('S-SES-1', '開いているものが無ければ新しく立てる', ['U27:事後', 'I46'], {'sessions': [S_O1]}, {'tool': 'claude-code', 'purpose_ref': 'ACDR 0197', 'parent': O1},
     [T('E1', '/result/decision/action', '新しく立てない', equals='new'), T('E2', '/result/register/registered/session/id', '登録されない', equals='NEW')]),
 ses('S-SES-2', 'オーケストレーターがリポジトリに開いていれば、ほかのターミナルも同じ ID で再開する', ['I46', 'U27:事後'], {'sessions': [S_O1], 'ai': {'claude-code/O1': AI(lastUsedAt='2026-10-24T09:58:00Z')}},
     {'tool': 'claude-code'}, [T('E1', '/result/decision/action', '再開しない', equals='resume'), T('E2', '/result/decision/session/id', '別の ID', equals='O1')]),
 ses('S-SES-3', '有効時間が切れたら、古いほうを閉じて新しく立てる', ['I46', 'I45'], {'sessions': [S_O1, S_P1], 'ai': {'claude-code/P1': AI(lastUsedAt='2026-10-24T08:00:00Z')}},
     {'tool': 'claude-code', 'purpose_ref': 'ACDR 0197', 'parent': O1},
     [T('E1', '/result/decision/action', '新しく立てない', equals='new'), T('E2', '/state/sessions/2/state', '古いほうの P1 が閉じない', equals='closed'), T('E3', '/state/sessions/0/state', '新しいほうが開かない', equals='open')]),
 ses('S-SES-4', '有効時間が分からない AI ツールは新しく立てる', ['I46'], {'sessions': [S_O1, {**S_P1, 'session': {'tool': 'kiro', 'id': 'P1'}}], 'ai': {'kiro/P1': AI(cacheLifetime='unknown')}},
     {'tool': 'kiro', 'purpose_ref': 'ACDR 0197', 'parent': O1}, [T('E1', '/result/decision/action', '分からないのに再開した', equals='new')], new={'tool': 'kiro', 'id': 'NEW'}),
 ses('S-SES-5', '履歴の割合が設定の値を超えたら、有効時間内でも新しく立てる', ['U27:事後'], {'sessions': [S_O1, S_P1], 'ai': {'claude-code/P1': AI(historyPercent=70)}, 'settings': {'project': {'newWhenHistoryAbovePercent': 60}}},
     {'tool': 'claude-code', 'purpose_ref': 'ACDR 0197', 'parent': O1}, [T('E1', '/result/decision/why', '設定で新規にならない', equals='history-above-setting')]),
 ses('S-SES-6', '設定が無ければ、履歴が大きくても有効時間内なら再開する', ['U27:事後'], {'sessions': [S_O1, S_P1], 'ai': {'claude-code/P1': AI(historyPercent=70)}},
     {'tool': 'claude-code', 'purpose_ref': 'ACDR 0197', 'parent': O1}, [T('E1', '/result/decision/action', '既定の値が置かれている', equals='resume')]),
 ses('S-SES-7', '手元の設定がプロジェクトの設定より強い', ['U27:事後'], {'sessions': [S_O1, S_P1], 'ai': {'claude-code/P1': AI(historyPercent=70)}, 'settings': {'user': {'newWhenHistoryAbovePercent': 90}, 'project': {'newWhenHistoryAbovePercent': 60}, 'local': {'newWhenHistoryAbovePercent': 80}}},
     {'tool': 'claude-code', 'purpose_ref': 'ACDR 0197', 'parent': O1}, [T('E1', '/result/decision/action', '手元の 80 が効いていない', equals='resume')]),
 ses('S-SES-8', '親が替わっても、同じ目的のセッションを再開する', ['I46', 'U27:事後'], {'sessions': [{**S_O1, 'state': 'closed'}, {'session': {'tool': 'claude-code', 'id': 'O2'}, 'state': 'open', 'last_used_at': '2026-10-24T09:59:00Z'}, S_P1], 'ai': {'claude-code/P1': AI()}},
     {'tool': 'claude-code', 'purpose_ref': 'ACDR 0197', 'parent': {'tool': 'claude-code', 'id': 'O2'}}, [T('E1', '/result/decision/action', '再開しない', equals='resume'), T('E2', '/result/decision/session/id', '別の ID', equals='P1')]),
 ses('S-SES-9', 'ほかの AI ツールで同じ枠が開いていれば断る', ['I46'], {'sessions': [S_O1], 'ai': {}}, {'tool': 'codex'},
     [T('E1', '/result/decision/action', '同時に開こうとした', equals='refuse')], new={'tool': 'codex', 'id': 'NEW'}),
 dict(id='S-SES-10', title='2つの端末が同時に新しく立てると、後の側は先の ID に切り替わる', traces=['I46'], adapters=['snapshot'],
      given={'files': {}, 'sessions': [S_O1]},
      when={'op': 'concurrent_register', 'args': {'decide': {'tool': 'claude-code', 'purpose_ref': 'ACDR 0197', 'parent': O1}, 'first': {'tool': 'claude-code', 'id': 'A'}, 'second': {'tool': 'claude-code', 'id': 'B'},
            'slot': {'purpose_ref': 'ACDR 0197', 'parent': O1}}},
      then=[T('E1', '/result/first/registered/session/id', '先の側が登録されない', equals='A'), T('E2', '/result/second/switched_to/session/id', '後の側が切り替わらない', equals='A')]),
]

def sql(sid, title, traces, q, refused=True, fk=True, params=()):
    g = {'files': {}, 'sessions': [S_O1, S_P1, {'session': {'tool': 'claude-code', 'id': 'D1'}, 'purpose_ref': 'ACDR 0197', 'parent': P1, 'role': 'reviewer', 'state': 'open', 'last_used_at': '2026-10-24T09:55:00Z'},
                                   {'session': {'tool': 'claude-code', 'id': 'C1'}, 'purpose_ref': 'ACDR 0100', 'parent': O1, 'state': 'closed', 'last_used_at': '2026-10-20T09:00:00Z'}]}
    return dict(id=sid, title=title, traces=traces, adapters=['snapshot'], given=g, when={'op': 'sql', 'args': {'sql': q, 'params': list(params), 'foreign_keys': fk}},
                then=[T('E1', '/result/refused' if refused else '/result/ok', '断るべき行を通した' if refused else '通すべき行を断った', exists=True)])
INS = "INSERT INTO sessions VALUES (?,?,?,?,?,?,?,?)"
adapter = [
 sql('S-DB-1', 'オーケストレーターの入れ子を断る', ['I44'], INS, params=['claude-code', 'X', None, 'claude-code', 'O1', None, 'open', '2026-10-24T10:00:00Z']),
 sql('S-DB-2', '親の無い目的のセッションを断る', ['I44'], INS, params=['claude-code', 'X', 'ACDR 0300', None, None, None, 'open', '2026-10-24T10:00:00Z']),
 sql('S-DB-3', '目的のセッションの下の目的のセッションを断る', ['I44'], INS, params=['claude-code', 'X', 'ACDR 0300', 'claude-code', 'P1', None, 'open', '2026-10-24T10:00:00Z']),
 sql('S-DB-4', '目的が親と違う任せる先を断る', ['I44'], INS, params=['claude-code', 'X', 'ACDR 0300', 'claude-code', 'P1', 'advisor', 'open', '2026-10-24T10:00:00Z']),
 sql('S-DB-5', '任せる先の下の任せる先を断る（子は呼ばない）', ['I44'], INS, params=['claude-code', 'X', 'ACDR 0197', 'claude-code', 'D1', 'advisor', 'open', '2026-10-24T10:00:00Z']),
 sql('S-DB-6', '2つ目の開いているオーケストレーターを断る', ['I46'], INS, params=['codex', 'O9', None, None, None, None, 'open', '2026-10-24T10:00:00Z']),
 sql('S-DB-7', '同じ目的の2つ目の目的のセッションを、親が違っても断る', ['I46'], INS, params=['codex', 'X', 'ACDR 0197', 'claude-code', 'O1', None, 'open', '2026-10-24T10:00:00Z']),
 sql('S-DB-8', '閉じた行を開き直すのを断る', ['I45'], "UPDATE sessions SET state='open' WHERE id='C1'"),
 sql('S-DB-9', '行を消すのを断る', ['I45'], "DELETE FROM sessions WHERE id='C1'"),
 sql('S-DB-10', 'INSERT OR REPLACE で閉じた行を開き直すのを断る', ['I45'], "INSERT OR REPLACE INTO sessions VALUES ('claude-code','C1','ACDR 0100','claude-code','O1',NULL,'open','2026-10-24T10:00:00Z')"),
 sql('S-DB-11', '目的を変えるのを断る', ['I44'], "UPDATE sessions SET purpose_ref='ACDR 0999' WHERE id='P1'"),
 sql('S-DB-12', '最後に使った時刻だけを変えるのは通す', ['I44'], "UPDATE sessions SET last_used_at='2026-10-24T10:00:00Z', purpose_ref=purpose_ref WHERE id='P1'", refused=False),
 sql('S-DB-13', '外部キーを切った接続でも、無いセッションの索引の行を断る', ['I38'], "INSERT INTO session_changes VALUES ('claude-code','NOPE',?)", fk=False, params=[U(1)]),
 sql('S-DB-14', '変更の記録の id でない値を断る', ['I37'], "INSERT INTO session_changes VALUES ('claude-code','P1','c1')"),
]

BODY = 'urn:semantic-agent-schema:base:semantic-abstract-schema:v1'
def sch(sid, title, traces, schema, doc, valid, patch=(), concrete=None):
    g = {'schema': schema, 'doc': doc, 'patch': list(patch)}
    if concrete: g['concrete'] = concrete
    return dict(id=sid, title=title, traces=traces, given=g, when={'op': 'schema'}, then=[T('E1', '/result/valid', '合否が期待と違う', equals=valid)])
static = [sch(f'S-ST-{n}', f'{n} は本体に従う', ['I3'], BODY, f'schemas/{n}.schema.json' if n != 'abstract' else 'schemas/abstract.json', True)
          for n in ['abstract', 'change', 'judgment', 'session', 'settings', 'change-view', 'annotations']] + [
 sch('S-ST-acdr', 'ACDR の Concrete は本体に従う', ['I3'], BODY, 'examples/acdr.concrete.json', True),
 sch('S-ST-bad-1', '入れ子の欄の壊れた x-prompt を断る', ['I4'], BODY, 'examples/acdr.concrete.json', False, [{'at': '/properties/shift/items/properties/what/x-prompt', 'value': {'oops': 1}}]),
 sch('S-ST-bad-2', 'argv のシェルの記号を断る', ['I4'], BODY, 'examples/acdr.concrete.json', False, [{'at': '/x-delegates/x', 'value': {'role': 'r', 'run': {'kind': 'judging', 'argv': ['a|b']}}}]),
 sch('S-ST-bad-3', 'argv を持つ任せる先の answer を断る', ['I4'], BODY, 'examples/acdr.concrete.json', False, [{'at': '/x-delegates/x', 'value': {'role': 'r', 'answer': {}, 'run': {'kind': 'judging', 'argv': ['t']}}}]),
 sch('S-ST-bad-4', 'インスタンスを作る Concrete に id が無ければ断る', ['I3'], BODY, 'examples/acdr.concrete.json', False, [{'at': '/properties/id', 'remove': True}]),
 sch('S-ST-bad-5', 'by が ai の行に agent が無ければ断る', ['I4'], BODY, 'examples/acdr.concrete.json', False, [{'at': '/properties/decision/x-prompt/write/0/agent', 'remove': True}]),
 sch('S-ST-bad-6', 'Markdown の対象に欄を書いた変更の記録を断る', ['I31'], 'urn:semantic-agent-schema:base:change:v1',
     {'$schema': 'urn:semantic-agent-schema:base:change:v1', 'kind': 'change', 'id': U(1), 'purpose_ref': 'ACDR 0197', 'intent': 'x', 'writer_kind': 'ai-agent', 'items': [{'target': {'path': 'docs/SKILL.md'}, 'field': '/why'}]}, False),
 sch('S-ST-bad-7', 'AI の判定にモデルが無ければ断る', ['I14'], 'urn:semantic-agent-schema:base:judgment:v1',
     {'$schema': 'urn:semantic-agent-schema:base:judgment:v1', 'kind': 'judgment', 'id': 'k', 'key': 'k', 'target': {'path': 'a'}, 'rule': 'urn:semantic-agent-schema:acdr:decision-record:v1#x', 'result': 'verified', 'grounds': [], 'judged_by': 'ai', 'judged_values': {'reads': []}}, False),
 sch('S-ST-bad-8', '4つ目の種類のセッション（目的があって親が無い）を断る', ['I44'], 'urn:semantic-agent-schema:base:session:v1#/$defs/session',
     {'session': {'tool': 'claude-code', 'id': 'X'}, 'purpose_ref': 'ACDR 0197', 'state': 'open', 'last_used_at': '2026-10-24T10:00:00Z'}, False),
 sch('S-ST-bad-9', '設定に知らない欄があれば断る（許可などを持たない）', ['I40'], 'urn:semantic-agent-schema:base:settings:v1',
     {'$schema': 'urn:semantic-agent-schema:base:settings:v1', 'kind': 'settings', 'id': 'project', 'permissions': {}}, False),
 sch('S-ST-bad-10', 'accepted の ACDR に docs が無ければ断る', ['I3'], 'urn:semantic-agent-schema:acdr:decision-record:v1', {k: v for k, v in json.load(open(os.path.join(HERE, '..', 'examples', 'acdr.instances.json')))['I3'].items() if k != 'docs'}, False, [], 'examples/acdr.concrete.json'),
]
CON = json.load(open(os.path.join(HERE, '..', 'examples', 'acdr.concrete.json')))
VIEW = {'$schema': 'urn:semantic-agent-schema:base:view:v1', 'kind': 'view', 'id': 'acdr', 'for': ACDR, 'sections': [{'title': '決定', 'parts': [{'part': 'heading', 'value': '/title'}, {'part': 'text', 'value': '/decision'}]}, {'title': '差分', 'parts': [{'part': 'table', 'value': '/docs'}]}]}
CAT = {'$schema': 'urn:semantic-agent-schema:base:catalog:v1', 'kind': 'catalog', 'id': 'catalog', 'parts': {'heading': {'description': '見出し', 'tokens': ['color.ink']}, 'text': {'description': '本文', 'tokens': ['color.ink', 'color.bg']}, 'table': {'description': '表', 'tokens': ['color.line']}}}
TOK = {'$schema': 'urn:semantic-agent-schema:base:design-tokens:v1', 'kind': 'design-tokens', 'id': 'tokens', 'tokens': {'color.ink': {'light': '#13201D', 'dark': '#E6EEEB'}, 'color.bg': {'light': '#F4F7F6', 'dark': '#101816'}, 'color.line': {'light': '#C9D6D2', 'dark': '#2C3B37'}}, 'contrast': [{'fg': 'color.ink', 'bg': 'color.bg', 'min': 4.5}]}
GLO = {'$schema': 'urn:semantic-agent-schema:base:glossary:v1', '$id': 'urn:semantic-agent-schema:acdr:glossary:v1', 'kind': 'glossary', 'id': 'glossary', '$defs': {'decision': {'$anchor': 'decision', 'description': '決めたこと1件'}}}
J = lambda o: json.dumps(o, ensure_ascii=False, indent=1)
def dfiles(view=VIEW, cat=CAT, tok=TOK, glo=GLO, extra=None):
    f = {'concrete/acdr.json': J(CON), '.semantic-agent-schema/views/acdr.json': J(view), '.semantic-agent-schema/design/catalog.json': J(cat), '.semantic-agent-schema/design/tokens.json': J(tok), '.semantic-agent-schema/glossary.json': J(glo)}
    f.update(extra or {}); return f
def des(sid, title, traces, files, then):
    return dict(id=sid, title=title, traces=traces, adapters=['snapshot'], given={'files': files}, when={'op': 'check_design', 'args': {'concretes': ['concrete/acdr.json']}}, then=then)
import copy
v2 = copy.deepcopy(VIEW); v2['sections'][0]['parts'][1]['value'] = '/no-such-field'
c2 = copy.deepcopy(CAT); c2['parts']['heading']['tokens'] = ['color.nope']
t2 = copy.deepcopy(TOK); t2['tokens']['color.ink']['dark'] = '#1A2420'
g2 = copy.deepcopy(GLO); g2['$defs']['decision']['$anchor'] = 'decided'
design = [
 des('S-DES-1', 'ビュー ・ カタログ ・ トークン ・ glossary が合っていれば違反は無い', ['I26'], dfiles(), [T('E1', '/result/violations', '違反が出た', count=0)]),
 des('S-DES-2', 'ビューが Concrete に無い欄を指せば違反', ['I26'], dfiles(view=v2), [T('E1', '/result/violations/0/rule', '検出されない', equals='value-in-concrete')]),
 des('S-DES-3', 'カタログがデザイントークンに無いトークンを使えば違反', ['I26'], dfiles(cat=c2), [T('E1', '/result/violations/0/rule', '検出されない', equals='tokens-exist')]),
 des('S-DES-4', '暗い表示でコントラストが足りなければ違反', ['I26'], dfiles(tok=t2), [T('E1', '/result/violations/0/detail', '暗い表示の不足が出ない', equals='dark')]),
 des('S-DES-5', '同じ kind にビューが2つあれば違反', ['I26'], dfiles(extra={'.semantic-agent-schema/views/acdr2.json': J({**VIEW, 'id': 'acdr2'})}), [T('E1', '/result/violations/0/rule', '検出されない', equals='one-view-per-kind')]),
 des('S-DES-6', 'glossary の $anchor がキーと違えば違反', ['I25'], dfiles(glo=g2), [T('E1', '/result/violations/0/rule', '検出されない', equals='anchor-is-name')]),
]
static += [sch(f'S-ST-{n}', f'{n} は本体に従う', ['I3'], BODY, f'schemas/{n}.schema.json', True) for n in ['view', 'catalog', 'design-tokens', 'glossary', 'published']] + [
 sch('S-ST-view', 'ACDR のビューは view に従う', ['I26'], 'urn:semantic-agent-schema:base:view:v1', VIEW, True),
 sch('S-ST-tokens', 'デザイントークンは design-tokens に従う', ['I26'], 'urn:semantic-agent-schema:base:design-tokens:v1', TOK, True),
 sch('S-ST-bad-11', '色の値でないトークンを断る', ['I26'], 'urn:semantic-agent-schema:base:design-tokens:v1', {**TOK, 'tokens': {'color.ink': {'light': 'red', 'dark': '#000000'}}}, False)]
static = [x for x in static if x['id'] != 'S-ST-change-view']
for name, xs in [('changes', changes), ('sessions', sessions), ('sqlite', adapter), ('static', static), ('design', design)]:
    json.dump(xs, open(os.path.join(HERE, f'{name}.json'), 'w'), ensure_ascii=False, indent=1)
print(sum(map(len, [changes, sessions, adapter, static])), '場面')
