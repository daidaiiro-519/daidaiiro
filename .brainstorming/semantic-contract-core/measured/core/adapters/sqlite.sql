-- 手元の DB（SQLite）のアダプター。行の形は Abstract の一員 session（schemas/session.schema.json）から書く。確定しない。
-- 不変条件は、接続の設定（foreign_keys ・ recursive_triggers）に依らずに断れるよう、CHECK と一意の索引とトリガーで書く。
-- 登録は BEGIN IMMEDIATE で始め、「決める」が見た開いている行といまの開いている行を比べてから書く（アダプターのコード）。

CREATE TABLE sessions (
  tool         TEXT NOT NULL,                 -- AI ツールの種類（claude-code ・ codex ・ copilot ・ gemini ・ kiro）
  id           TEXT NOT NULL,                 -- そのツールが発行した文字列。基盤は解釈しない（I34）
  purpose_ref  TEXT,                          -- 目的の参照。オーケストレーターのセッションは持たない
  parent_tool  TEXT,
  parent_id    TEXT,                          -- 親。オーケストレーターのセッションは持たない
  role         TEXT,                          -- 役割：親の Concrete の x-delegates の名前。任せる先のセッションだけが持つ
  state        TEXT NOT NULL CHECK (state IN ('open','closed')),
  last_used_at TEXT NOT NULL CHECK (last_used_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T[0-9][0-9]:[0-9][0-9]:[0-9][0-9]*Z'),
  PRIMARY KEY (tool, id),
  FOREIGN KEY (parent_tool, parent_id) REFERENCES sessions (tool, id),
  CHECK ((parent_id IS NULL) = (parent_tool IS NULL)),
  -- セッションの種類は3つだけ（目的 ・ 親 ・ 役割の組）
  CHECK (
       (purpose_ref IS NULL     AND parent_id IS NULL     AND role IS NULL)       -- オーケストレーター
    OR (purpose_ref IS NOT NULL AND parent_id IS NOT NULL AND role IS NULL)       -- 目的のセッション
    OR (purpose_ref IS NOT NULL AND parent_id IS NOT NULL AND role IS NOT NULL)   -- 任せる先のセッション
  )
);

-- I46：開いているオーケストレーターは1つ。目的のセッションは AI ツールと親に関係なく同じ目的で1つ。
--      任せる先のセッションは目的 ・ 親 ・ 役割が同じで1つ（目的は親で決まるので、親と役割で足りる）。
CREATE UNIQUE INDEX one_open_orchestrator_session ON sessions ((1))
  WHERE state = 'open' AND purpose_ref IS NULL;
CREATE UNIQUE INDEX one_open_purpose_session ON sessions (purpose_ref)
  WHERE state = 'open' AND purpose_ref IS NOT NULL AND role IS NULL;
CREATE UNIQUE INDEX one_open_delegate_session ON sessions (parent_tool, parent_id, role)
  WHERE state = 'open' AND role IS NOT NULL;

-- I45：閉じたセッションは開き直さない。I44：ID ・ 目的 ・ 親 ・ 役割は変えない（値が変わるときだけ断る）。
CREATE TRIGGER no_reopen BEFORE UPDATE OF state ON sessions
  WHEN OLD.state = 'closed' AND NEW.state = 'open'
  BEGIN SELECT RAISE(ABORT, 'I45'); END;
CREATE TRIGGER fixed_at_creation BEFORE UPDATE OF tool, id, purpose_ref, parent_tool, parent_id, role ON sessions
  WHEN OLD.tool IS NOT NEW.tool OR OLD.id IS NOT NEW.id OR OLD.purpose_ref IS NOT NEW.purpose_ref
    OR OLD.parent_tool IS NOT NEW.parent_tool OR OLD.parent_id IS NOT NEW.parent_id OR OLD.role IS NOT NEW.role
  BEGIN SELECT RAISE(ABORT, 'I44'); END;
-- 行を消さない ・ 同じ ID を入れ直さない（DELETE と入れ直し、INSERT OR REPLACE で I45 を迂回させない）
CREATE TRIGGER no_delete BEFORE DELETE ON sessions
  BEGIN SELECT RAISE(ABORT, 'I45'); END;
CREATE TRIGGER no_reinsert BEFORE INSERT ON sessions
  WHEN EXISTS (SELECT 1 FROM sessions WHERE tool = NEW.tool AND id = NEW.id)
  BEGIN SELECT RAISE(ABORT, 'I45'); END;
-- I44：目的のセッションの親はオーケストレーター。任せる先の親は同じ目的の目的のセッション（入れ子にしない ・ 子は呼ばない）。
CREATE TRIGGER parent_kind BEFORE INSERT ON sessions WHEN NEW.parent_id IS NOT NULL
  BEGIN
    SELECT RAISE(ABORT, 'I44')
      WHERE NEW.role IS NULL AND NOT EXISTS (SELECT 1 FROM sessions p
        WHERE p.tool = NEW.parent_tool AND p.id = NEW.parent_id AND p.purpose_ref IS NULL);
    SELECT RAISE(ABORT, 'I44')
      WHERE NEW.role IS NOT NULL AND NOT EXISTS (SELECT 1 FROM sessions p
        WHERE p.tool = NEW.parent_tool AND p.id = NEW.parent_id AND p.purpose_ref = NEW.purpose_ref AND p.role IS NULL);
  END;

-- 手元のセッションの索引（I38）。変更の記録はセッション ID を持たない（I34）。
CREATE TABLE session_changes (
  tool      TEXT NOT NULL,
  id        TEXT NOT NULL,
  change_id TEXT NOT NULL CHECK (change_id GLOB '[0-7][0-9A-HJKMNP-TV-Z][0-9A-HJKMNP-TV-Z]*' AND length(change_id) = 26),
  PRIMARY KEY (tool, id, change_id),
  FOREIGN KEY (tool, id) REFERENCES sessions (tool, id)
);
CREATE TRIGGER session_change_session_exists BEFORE INSERT ON session_changes
  WHEN NOT EXISTS (SELECT 1 FROM sessions WHERE tool = NEW.tool AND id = NEW.id)
  BEGIN SELECT RAISE(ABORT, 'session-exists'); END;
