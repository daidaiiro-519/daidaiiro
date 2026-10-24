"""参照の実装の基盤（ドメインサービス）。アダプターの名前を知らず、ポートの形だけで呼ぶ。
model/model.json の決定を実装する：変更を記録する（①）・ I33 を判定させる ・ 本人の承認と確定 ・ 変更を見せる ・ 再開か新規かを決める（②）。"""
import difflib, json, os
from datetime import datetime, timezone

REC = '.semantic-agent-schema/changes/'
JUD = '.semantic-agent-schema/judgments/'
LOCAL_ONLY = {'.semantic-agent-schema/settings.local.json'}
CHANGE_KIND = 'urn:semantic-agent-schema:base:change:v1'
I33 = CHANGE_KIND + '#intent-matches-change'
EXC_ORDER = ['intent-missing', 'intent-mismatch', 'unrecorded', 'no-change', 'record-rewritten', 'human-field-changed',
             'state-changed', 'unconfirmed-outside-intent', 'not-in-range', 'unreadable']


class Base:
    def __init__(self, work, changes, store, ai, judge, settings, ids, now):
        self.w, self.ch, self.st, self.ai, self.judge, self.settings = work, changes, store, ai, judge, settings
        self.ids, self.now = list(ids), now

    # ---------- 共通
    def _p(self, rel): return os.path.join(self.w, rel)
    def _write(self, rel, obj):
        os.makedirs(os.path.dirname(self._p(rel)), exist_ok=True)
        json.dump(obj, open(self._p(rel), 'w'), ensure_ascii=False, indent=1)
    def _load(self, rel):
        return json.load(open(self._p(rel))) if os.path.exists(self._p(rel)) else None
    def _pending(self):
        ba = self.ch.before_after(self.ch.last_confirmed()['ref'], None)
        return [i for i in ba['items'] if i['target'] not in LOCAL_ONLY]
    @staticmethod
    def _is_record(p): return p.startswith(REC) or p.startswith(JUD)

    def _target_of(self, item):
        """対象：インスタンス（$schema ・ id ・ kind を持つ JSON）なら id と kind、それ以外はパス"""
        txt = item['after'] if item['after'] is not None else item['before']
        if item['target'].endswith('.json') and txt and not txt.startswith('\0'):
            try:
                d = json.loads(txt)
                if isinstance(d, dict) and {'$schema', 'id', 'kind'} <= set(d): return {'id': d['id'], 'kind': d['$schema']}, d
            except ValueError: pass
        return {'path': item['target']}, None

    @staticmethod
    def _fields(before, after):
        b = json.loads(before) if before else {}; a = json.loads(after) if after else {}
        return [{'at': '/' + k, 'before': b.get(k), 'after': a.get(k)} for k in list(b) + [k for k in a if k not in b] if b.get(k) != a.get(k)]

    # ---------- ① 未確定の変更のうち、どの記録にも入っていないもの
    def outside_records(self):
        pend = self._pending()
        claimed = set()
        for i in pend:
            if i['target'].startswith(REC) and i['after']:
                for it in json.loads(i['after'])['items']:
                    claimed.add(json.dumps(it['target'], sort_keys=True))
        out = []
        for i in pend:
            if self._is_record(i['target']): continue
            t, _ = self._target_of(i)
            if json.dumps(t, sort_keys=True) not in claimed: out.append(t)
        return out

    def check_pending(self):
        """作業を始める前に呼ぶ。意図に入らない未確定の変更があれば、先に確定するよう返す"""
        out = self.outside_records()
        return {'exceptions': [{'type': 'unconfirmed-outside-intent', 'count': len(out), 'items': out}] if out else []}

    # ---------- ① 変更を記録する（AI は意図だけを渡す）
    def record_change(self, intent, session=None, purpose_ref=None, writer_kind='ai-agent'):
        if session:
            row = self.st.get(session)
            if row is None: return {'refused': 'unknown-session'}
            purpose_ref = row.get('purpose_ref') or purpose_ref
        if not purpose_ref: return {'refused': 'no-purpose'}
        pend = {i['target']: i for i in self._pending()}
        targets = self.outside_records()
        if not targets: return {'refused': 'nothing-changed'}
        items = []
        for t in targets:
            it = {'target': t}
            src = next((i for i in pend.values() if self._target_of(i)[0] == t), None)
            if 'id' in t and src: items += [dict(it, field=f['at']) for f in self._fields(src['before'], src['after'])] or [it]
            else: items.append(it)
        cid = self.ids.pop(0)
        rec = {'$schema': CHANGE_KIND, 'kind': 'change', 'id': cid, 'purpose_ref': purpose_ref, 'intent': intent, 'writer_kind': writer_kind, 'items': items}
        self._write(REC + cid + '.json', rec)
        if session and self.st.get(session).get('purpose_ref'): self.st.add_index(session, cid)
        return {'change': rec}

    # ---------- I33 を判定させる（任せる先 judge。本物の AI は呼ばない場面では StubJudge）
    def judge_intent(self, change_id):
        rec = self._load(REC + change_id + '.json')
        pend = {i['target']: i for i in self._pending()}
        ans = self.judge.ask({'delegate': 'judge', 'target': {'id': change_id, 'kind': CHANGE_KIND}, 'rules': [I33],
                              'inputs': {'intent': rec['intent'], 'targets': [it['target'] for it in rec['items']]},
                              'task': '意図と変更が合っているかを判定する', 'handoff': ''})
        key = f'change/{change_id}/intent-matches-change'
        befores = [{'at': '/' + p, 'value': pend[p]['before']} for p in pend if not self._is_record(p)]
        j = {'$schema': 'urn:semantic-agent-schema:base:judgment:v1', 'kind': 'judgment', 'id': key, 'key': key,
             'target': {'id': change_id, 'kind': CHANGE_KIND}, 'rule': I33, 'result': ans['result'], 'grounds': ans['grounds'],
             'meaning': ans['meaning'], 'judged_by': 'ai',
             'judged_values': {'reads': [{'at': '/intent', 'value': rec['intent']}], 'before': befores, 'prompts': ['意図と変更が合っているか'],
                               'handoff': '', 'delegate': 'judge', 'model': 'stub'}}
        if 'question' in ans: j['question'] = ans['question']
        self._write(JUD + key + '.json', j)
        return {'judgment': j}

    # ---------- 本人の承認（人の入口）と確定
    def approve(self, change_id, message=None):
        rec = self._load(REC + change_id + '.json')
        key = f'change/{change_id}/intent-matches-change'
        prev = self._load(JUD + key + '.json') or {}
        j = {'$schema': 'urn:semantic-agent-schema:base:judgment:v1', 'kind': 'judgment', 'id': key, 'key': key,
             'target': {'id': change_id, 'kind': CHANGE_KIND}, 'rule': I33, 'result': 'verified',
             'grounds': [{'at': '/intent', 'text': '本人が承認した'}], 'judged_by': 'human',
             'meaning': prev.get('meaning', {'whole': rec['intent'], 'per_target': []}),
             'judged_values': {'reads': [{'at': '/intent', 'value': rec['intent']}], 'before': prev.get('judged_values', {}).get('before', [])}}
        self._write(JUD + key + '.json', j)
        paths = [REC + change_id + '.json', JUD + key + '.json']
        pend = self._pending()
        for it in rec['items']:
            t = it['target']
            for i in pend:
                if self._target_of(i)[0] == t and i['target'] not in paths: paths.append(i['target'])
        return self.ch.confirm(paths, message or rec['intent'])

    # ---------- 変更を見せる（まとめ方：前の姿の参照 ・ 後の姿の参照）
    def change_view(self, before, after=None, session=None):
        ba = self.ch.before_after(before, after)
        if 'unreachable' in ba: return {'not_shown': 'unreachable', 'ref': ba['unreachable']}
        items = [i for i in ba['items'] if i['target'] not in LOCAL_ONLY]
        recs = [json.loads(i['after']) for i in items if i['target'].startswith(REC) and i['after']]
        juds = {json.loads(i['after'])['target']['id']: json.loads(i['after']) for i in items if i['target'].startswith(JUD) and i['after']}
        exc = {k: [] for k in EXC_ORDER}
        by_target, unreadable = {}, []
        for i in items:
            if self._is_record(i['target']): continue
            if (i['before'] or '').startswith('\0') or (i['after'] or '').startswith('\0'): unreadable.append(i['target']); continue
            by_target[json.dumps(self._target_of(i)[0], sort_keys=True)] = i
        covered = set(); purposes = {}
        for r in sorted(recs, key=lambda r: r['id']):
            ts = []
            for t in {json.dumps(it['target'], sort_keys=True) for it in r['items']}:
                src = by_target.get(t)
                if src is None: exc['no-change'].append({'change_id': r['id'], 'target': json.loads(t)}); continue
                covered.add(t)
                tgt = json.loads(t)
                if 'id' in tgt: ts.append({'target': tgt, 'kind': 'fields', 'fields': self._fields(src['before'], src['after'])})
                else:
                    a, b = (src['before'] or '').splitlines(), (src['after'] or '').splitlines()
                    lines = []
                    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, a, b).get_opcodes():
                        if tag == 'equal': lines += [{'mark': ' ', 'text': x} for x in b[j1:j2]]
                        else: lines += [{'mark': '-', 'text': x} for x in a[i1:i2]] + [{'mark': '+', 'text': x} for x in b[j1:j2]]
                    ts.append({'target': tgt, 'kind': 'lines', 'lines': lines})
            ts.sort(key=lambda x: json.dumps(x['target'], sort_keys=True))
            jd = juds.get(r['id'])
            res = jd['result'] if jd else 'none'
            if res == 'violated': exc['intent-mismatch'].append({'change_id': r['id']})
            purposes.setdefault(r['purpose_ref'], []).append({'change_id': r['id'], 'intent': r['intent'], 'judgment': res,
                                                              'meaning': (jd or {}).get('meaning', {}), 'targets': ts})
        for t in sorted(set(by_target) - covered): exc['unrecorded'].append({'target': json.loads(t)})
        if session:
            have = {r['id'] for r in recs}
            for cid in self.st.index_of(session):
                if cid not in have: exc['not-in-range'].append({'change_id': cid})
        if unreadable: exc['unreadable'] = [{'path': p} for p in unreadable]
        return {'range': {'before': before, 'after': after or 'unconfirmed'},
                'exceptions': [{'type': k, 'count': len(v), 'items': v} for k, v in exc.items() if v],
                'purposes': [{'purpose_ref': p, 'intents': v} for p, v in sorted(purposes.items())],
                'unreadable': sorted(unreadable)}

    # ---------- 判定が効いているか（I16）
    def judgment_effective(self, key):
        j = self._load(JUD + key + '.json')
        rec = self._load(REC + j['target']['id'] + '.json')
        return {'effective': all(r['value'] == rec.get(r['at'].lstrip('/')) for r in j['judged_values']['reads'])}

    # ---------- ② 再開か新規かを決める（AI ツールに依らない）
    def decide(self, tool, purpose_ref=None, parent=None, role=None):
        cur = self.st.open_in_slot(purpose_ref, parent, role)
        if cur is None: return {'action': 'new', 'close': [], 'seen': None}
        if cur['session']['tool'] != tool:
            return {'action': 'refuse', 'reason': 'open-in-another-tool', 'open': cur['session']}
        info = self.ai.inspect(cur['session']['tool'], cur['session']['id'])
        limit = self.settings.read().get('newWhenHistoryAbovePercent')
        expired = True
        if info['resumable'] and info['cacheLifetime'] != 'unknown':
            secs = _duration(info['cacheLifetime'])
            expired = (_t(self.now) - _t(info['lastUsedAt'])).total_seconds() > secs
        if expired or not info['resumable']:
            return {'action': 'new', 'close': [cur['session']], 'seen': cur, 'why': 'not-resumable-or-expired'}
        if limit is not None and info['historyPercent'] > limit:
            return {'action': 'new', 'close': [cur['session']], 'seen': cur, 'why': 'history-above-setting'}
        return {'action': 'resume', 'session': cur['session']}

    def register(self, decision, new_session, purpose_ref=None, parent=None, role=None):
        row = {'session': new_session, 'state': 'open', 'last_used_at': self.now}
        if purpose_ref: row['purpose_ref'] = purpose_ref
        if parent: row['parent'] = parent
        if role: row['role'] = role
        return self.st.register(row, decision.get('close', []), decision.get('seen'))


def _t(s): return datetime.fromisoformat(s.replace('Z', '+00:00'))
def _duration(s):
    s = s[2:]; n = 0; num = ''
    for ch in s:
        if ch.isdigit(): num += ch
        else: n += int(num) * {'H': 3600, 'M': 60, 'S': 1}[ch]; num = ''
    return n


# ---------- 描画の前の形と参照の検査（Q1 #23 ・ I26）。描画そのものはまだ作らない
def _lum(hex_):
    def ch(c):
        c = int(c, 16) / 255
        return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4
    r, g, b = hex_[1:3], hex_[3:5], hex_[5:7]
    return 0.2126 * ch(r) + 0.7152 * ch(g) + 0.0722 * ch(b)


def _contrast(a, b):
    la, lb = sorted([_lum(a), _lum(b)], reverse=True)
    return (la + 0.05) / (lb + 0.05)


def check_design(self, concretes):
    """文脈のビュー ・ カタログ ・ デザイントークン ・ glossary を、Abstract のファイルの rule（machine）で検査する"""
    W = self.w; out = []
    load = lambda rel: json.load(open(os.path.join(W, rel))) if os.path.exists(os.path.join(W, rel)) else None
    cons = {}
    for rel in concretes:
        c = load(rel); cons[c['$id']] = c
    cat = load('.semantic-agent-schema/design/catalog.json') or {'parts': {}}
    tok = load('.semantic-agent-schema/design/tokens.json') or {'tokens': {}, 'contrast': []}
    vdir = os.path.join(W, '.semantic-agent-schema/views')
    views = [json.load(open(os.path.join(vdir, f))) for f in sorted(os.listdir(vdir))] if os.path.isdir(vdir) else []
    seen = {}
    for v in views:
        seen.setdefault(v['for'], []).append(v['id'])
        c = cons.get(v['for'])
        if c is None: out.append({'rule': 'for-exists', 'at': v['id'], 'detail': v['for']}); continue
        for sec in v['sections']:
            for p in sec['parts']:
                head = p['value'].lstrip('/').split('/')[0]
                if head not in c.get('properties', {}): out.append({'rule': 'value-in-concrete', 'at': v['id'], 'detail': p['value']})
                if p['part'] not in cat['parts']: out.append({'rule': 'part-in-catalog', 'at': v['id'], 'detail': p['part']})
    for k, ids in seen.items():
        if len(ids) > 1: out.append({'rule': 'one-view-per-kind', 'at': k, 'detail': ids})
    for name, part in cat['parts'].items():
        for t in part['tokens']:
            if t not in tok['tokens']: out.append({'rule': 'tokens-exist', 'at': name, 'detail': t})
    for pair in tok.get('contrast', []):
        for mode in ('light', 'dark'):
            fg, bg = tok['tokens'].get(pair['fg']), tok['tokens'].get(pair['bg'])
            if fg and bg and _contrast(fg[mode], bg[mode]) < pair['min']:
                out.append({'rule': 'contrast-meets', 'at': f"{pair['fg']}/{pair['bg']}", 'detail': mode})
    g = load('.semantic-agent-schema/glossary.json')
    if g:
        for k, d in g['$defs'].items():
            if d.get('$anchor') != k: out.append({'rule': 'anchor-is-name', 'at': k, 'detail': d.get('$anchor')})
    return {'violations': out}


Base.check_design = check_design
