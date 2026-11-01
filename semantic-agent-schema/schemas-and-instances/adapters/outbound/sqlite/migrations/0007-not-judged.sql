-- 版33：判定の区分の値 unsupported を not-judged にする（まだ判定していない、を名前で言う）。
-- rule_results の result の CHECK を変えるので、表を作り直して行を移す（CODING.md：CHECK を変える移行は作り直す。
-- rule_results はほかの表の trigger から読まれないので作り直してよい）。外部キーの検査は入口が止めて、移したあとに確かめる。
-- 追記だけの決まり（trigger）は、作り直した表に付け直す。

CREATE TABLE rule_results_new (
    validation_id TEXT NOT NULL REFERENCES validations (id),
    ord           INTEGER NOT NULL,
    rule_id       TEXT NOT NULL,
    pointer       TEXT NOT NULL,
    by            TEXT NOT NULL CHECK (by IN ('machine', 'ai')),
    result        TEXT NOT NULL CHECK (result IN ('verified', 'violated', 'unverified', 'error', 'not-judged')),
    reason        TEXT NOT NULL,
    reads_hash    TEXT NOT NULL,
    PRIMARY KEY (validation_id, ord)
) STRICT;

INSERT INTO rule_results_new (validation_id, ord, rule_id, pointer, by, result, reason, reads_hash)
    SELECT validation_id, ord, rule_id, pointer, by,
           CASE result WHEN 'unsupported' THEN 'not-judged' ELSE result END,
           reason, reads_hash
    FROM rule_results;

-- 消すと trigger も消える（DROP TABLE は DELETE の trigger を起こさない）
DROP TABLE rule_results;

ALTER TABLE rule_results_new RENAME TO rule_results;

CREATE TRIGGER rule_results_no_update BEFORE UPDATE ON rule_results
BEGIN SELECT RAISE(ABORT, 'rule results are append-only'); END;

CREATE TRIGGER rule_results_no_delete BEFORE DELETE ON rule_results
BEGIN SELECT RAISE(ABORT, 'rule results are append-only'); END;
