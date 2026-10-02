---
name: schema-driven-base-board
description: ブレストボード「スキーマ駆動の道具（抽象の基盤）」の現在地
metadata:
  node_type: memory
  type: project
---

`.brainstorming-board/schema-driven-base/`（Artifact https://claude.ai/artifact/2SFURsbhMHgDHg9Vj5oLgG ）。スキーマ駆動の道具を、どの concrete にも使える抽象の基盤として形を決める。ユースケース駆動はその concrete の1つで、ボード usecase-modeling-coding と ACDR 0092 の決定は concrete の側の決定として扱う。

現在地（2026-10-02、1回目）：
- 論点1（基盤が持つ形：kind ・ id ・ name ・ body だけ。body の形は concrete の JSON Schema、節と参照の在りかは concrete が JSONPath で申告）は回答待ち
- 論点2（concrete の契約）・ 3（射影と変換）・ 4（配り方と名前）は待ち

試作の基盤（usecase-schema-driven/trial/spike/base）は、宣言の形として rules ・ ops（pre ・ post ・ writes ・ nodes）を固定している。これを concrete の Schema へ移すのが論点1の要点。
