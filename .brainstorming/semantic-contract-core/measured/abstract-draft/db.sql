-- 手元の DB（SQLite）。基盤の SQLite アダプターが持つ（案）。コミットしない
-- 行の形は Abstract の一員 session（session.schema.json）に合わせる

CREATE TABLE sessions (
  tool         TEXT NOT NULL,                 -- AI ツールの種類（claude-code ・ codex ・ kiro ・ copilot）
  id           TEXT NOT NULL,                 -- そのツールが発行した文字列。基盤は解釈しない（I34）
  purpose_ref  TEXT,                          -- 目的の参照。オーケストレーターのセッションは持たない
  parent_tool  TEXT,
  parent_id    TEXT,                          -- 親。オーケストレーターのセッションは持たない
  role         TEXT,                          -- 役割：任せる先の参照（<Concrete の URN>#<x-delegates の名前>）。任せる先のセッションだけが持つ
  state        TEXT NOT NULL CHECK (state IN ('open','closed')),
  last_used_at TEXT NOT NULL,                 -- ISO 8601（UTC）
  PRIMARY KEY (tool, id),
  FOREIGN KEY (parent_tool, parent_id) REFERENCES sessions (tool, id),
  CHECK ((parent_id IS NULL) = (parent_tool IS NULL)),
  CHECK (role IS NULL OR parent_id IS NOT NULL)            -- 役割があれば親がある
);

-- I46：目的の参照を持つセッションは、目的 ・ 親 ・ 役割が同じで開いているのを1つまで。オーケストレーターのセッション（目的なし）はいくつでもよい。NULL どうしは SQLite では別の値になるので、空の文字列に寄せてから一意にする
CREATE UNIQUE INDEX one_open_per_purpose_parent_role
  ON sessions (COALESCE(purpose_ref,''), COALESCE(parent_tool,''), COALESCE(parent_id,''), COALESCE(role,''))
  WHERE state = 'open' AND purpose_ref IS NOT NULL;

-- I45：閉じたセッションは開き直さない。ID ・ 目的 ・ 親 ・ 役割は立てた時に決めて変えない（Q11 の不変条件①）
CREATE TRIGGER no_reopen BEFORE UPDATE OF state ON sessions
  WHEN OLD.state = 'closed' AND NEW.state = 'open'
  BEGIN SELECT RAISE(ABORT, 'I45: 閉じたセッションは開き直さない'); END;
CREATE TRIGGER fixed_at_creation BEFORE UPDATE OF tool, id, purpose_ref, parent_tool, parent_id, role ON sessions
  BEGIN SELECT RAISE(ABORT, '不変条件①: ID ・ 目的 ・ 親 ・ 役割は変えない'); END;

-- 手元のセッションの索引：セッション → 変更の記録の id（I38）。変更の記録はコミットするファイルで、セッション ID を持たない（I34）
CREATE TABLE session_changes (
  tool      TEXT NOT NULL,
  id        TEXT NOT NULL,
  change_id TEXT NOT NULL,
  PRIMARY KEY (tool, id, change_id),
  FOREIGN KEY (tool, id) REFERENCES sessions (tool, id)
);
