---
name: ai-workshop-shared
description: AIワークショップの共同制作。共通導入と、企画書・オリエンテーションの担当を分け、方針と制作資料を共有中。
metadata:
  node_type: memory
  type: project
---

# AIワークショップの共同制作の現在地

2026年10月24日、利用者が担当を分けた共同制作を希望した。当初は共通の「はじめに」と企画書・オリエンテーションの担当を分けた。2026年10月25日、共通導入の完成後に、利用者の指示で同じセッションがオリエンテーションの見直しを開始した。最新の担当情報と以下の変更記録を参照する。

- [共同作業の入口](../docs/workshops/ai-workshop/review/shared/README.md)
- [共通方針](../docs/workshops/ai-workshop/review/shared/direction.md)
- [教材内容の主要な参照先](../docs/workshops/ai-workshop/review/shared/material-references.md)
- [制作上の注意点とSkillの参照先](../.claude/skills/slide-deck/references/ai-workshop-production.md)
- [メイン画面と画面遷移・生成元](../docs/workshops/ai-workshop/review/shared/viewer-guide.md)
- [担当情報](../docs/workshops/ai-workshop/review/shared/tracks/)
- [方針の変更記録](../docs/workshops/ai-workshop/review/shared/changes/)
- [別セッションへ入力する開始用プロンプト](../docs/workshops/ai-workshop/review/shared/start-proposal-orientation.md)

枚ごとの進捗は担当情報が指すファイルにある。共同制作の開始は、スライドの正式反映や動画制作の承認ではない。

2026年10月24日、利用者が画面遷移とメイン画面の導線を確認し、共同制作への共有を依頼した。メインは <http://127.0.0.1:8772/>、画面遷移図は <http://127.0.0.1:8772/navigation.html>、制作情報は <http://127.0.0.1:8772/workflow.html>。生成元と別担当の追加方法は上記の画面の案内にある。教材の確認状態は変更していない。

2026年10月24日、利用者がddd-advisor、meta-thinking-advisor、usecase-advisorを教材内容の重要な参照先と指定した。共通の参照資料と、別セッションの開始用プロンプトへ追加した。スライドの確認状態は変更していない。


2026年10月25日、共通方針revisionを2026-10-25-08へ更新。教材2の中心を、教材1の目的・要件から業務の対象・判断・関係と守る条件をモデルに表して設計へつなげることと明確化した。言葉の意味と文脈の確認はその一部である。別担当も[共通方針](../docs/workshops/ai-workshop/review/shared/direction.md)と[変更記録](../docs/workshops/ai-workshop/review/shared/changes/2026-10-25-11-intro-material-2-model-design.md)を読む。

2026年10月25日、共通方針revisionを2026-10-25-09へ更新。AgentsがSkillsを使用し、要件定義・設計・実装・テストの工程を進める関係を明記。Skillsを工程として扱わず、各工程での判断と確認を支える手順として紹介する。8枚目の図と原稿は再修正して利用者確認待ち。詳細は[変更記録](../docs/workshops/ai-workshop/review/shared/changes/2026-10-25-13-intro-agents-use-skills.md)。

2026年10月25日、共通方針revisionを2026-10-25-10へ更新。この教材の基本例は一つのAgentが各工程でSkillsを使う形とし、工程ごとに別Agentを呼ぶ構成を勧めない。8枚目は上段Agent・下段各工程のSkillsへ図を修正して確認待ち。[変更記録](../docs/workshops/ai-workshop/review/shared/changes/2026-10-25-14-intro-single-agent-layout.md)を参照。

2026年10月25日、はじめに全8枚を通し確認しLuna再確認まで完了。単体に必須矛盾なし。現行教材1本編の旧題材と、オリエンテーション8・10・11・14枚目の教材2紹介には最新方針との差分が残る。[変更記録18](../docs/workshops/ai-workshop/review/shared/changes/2026-10-25-18-intro-final-sequence-review.md)を各担当が参照する。単体の確認と教材全体の完成、利用者承認は区別する。

2026年10月25日、共通方針revisionを2026-10-25-11へ更新。教材紹介名は目的と要件の明確化・業務のモデル化と設計・AI活用の実践と改善。開発の共通題材をPM・PLの対応範囲・変更影響・完了根拠の判断と、自身のAI活用へつなぐ方針を明記。具体的な役割別実践例は本編改訂への依存。[変更記録21](../docs/workshops/ai-workshop/review/shared/changes/2026-10-25-21-intro-learning-titles-roles.md)を参照。

2026年10月25日、各役割のAI活用という観点で受講者Lunaが最新の共通導入9枚を確認。PM・PLのAI利用が本文へつながらないこと、役割ごとの学ぶ重点が不明なことなどを指摘。感想と提案の採否は未決定で教材は未変更。[共有変更記録22](../docs/workshops/ai-workshop/review/shared/changes/2026-10-25-22-intro-luna-roles-feedback.md)を参照。

2026年10月25日、共通方針revisionを2026-10-25-12へ更新。中心の問いを開発全体の生産性・品質とし、同じ案件でPM・PLがAIを使う一枚を加えた本文9枚案を承認。旧版固定後、新案の本文1のみ図と原稿へ反映、確認待ち。全体の完成扱いにしない。[変更記録23](../docs/workshops/ai-workshop/review/shared/changes/2026-10-25-23-intro-whole-project-story.md)を参照。

2026年10月25日、共通方針revision 2026-10-25-12の新本文6を反映。PM・PL自身のAI利用を追加し、表紙＋本文9枚の全10枚。本文1〜5確認済み、6確認待ち、7〜9は移動のみで未改訂。[変更記録31](../docs/workshops/ai-workshop/review/shared/changes/2026-10-25-31-intro-pm-ai-slide06.md)を参照。

2026年10月25日、共通方針revision 2026-10-25-13へ更新。本文6のPM・PL入力・照合例を撤回し、事例の三つの問題から教材1〜3の学びへ接続する方針に同意。職種別の例を追加するために不自然な担当範囲を割り当てない。[変更記録32](../docs/workshops/ai-workshop/review/shared/changes/2026-10-25-32-intro-learning-bridge.md)を参照。

2026年10月25日、共通導入の本文7を教材1の紹介として修正。目的に合う範囲を定め、既存仕様で要件を確かめ、AIの作業範囲へ戻る内容。本文1〜6確認済み、7確認待ち、8〜9未改訂。共通方針revisionは2026-10-25-13のまま。[変更記録33](../docs/workshops/ai-workshop/review/shared/changes/2026-10-25-33-intro-material01-purpose.md)を参照。

2026年10月25日、共通導入の本文8を教材2の紹介として修正。目的と要件から業務上の対象・判断・関係をモデルに表し、状態と処理の設計へ反映する内容。本文1〜7確認済み、8確認待ち、9未改訂。共通方針revisionは2026-10-25-13のまま。[変更記録34](../docs/workshops/ai-workshop/review/shared/changes/2026-10-25-34-intro-material02-model.md)を参照。

2026年10月25日、共通導入の本文9を教材3の紹介として修正。既存の一つのAgent→各工程でSkillsを使う構図を維持し、判断観点と確認手順へ具体化する内容。本文1〜8確認済み、9確認待ち。全編へ新案反映したが最終通し確認と正式反映は未実施。共通方針revisionは2026-10-25-13のまま。[変更記録35](../docs/workshops/ai-workshop/review/shared/changes/2026-10-25-35-intro-material03-skills.md)を参照。

2026年10月25日、最新共通導入の全10枚を受講者Lunaが確認。話の大筋は理解でき、経理承認と経理確認の表記揺れを必須指摘。本文3の図と原稿を経理確認へ修正して再確認待ち。正本・音声動画は未変更。[変更記録36](../docs/workshops/ai-workshop/review/shared/changes/2026-10-25-36-intro-final-luna-terms.md)を参照。

2026年10月25日、利用者の正本化指示を受け、共通のはじめにをACDR0205で正本へ反映。表紙1枚＋本文9枚のスライドと原稿を再生成。表紙の専用原稿も入力へ記録。音声・動画は改訂前。[変更記録37](../docs/workshops/ai-workshop/review/shared/changes/2026-10-25-37-intro-canonical-release.md)を参照。

2026年10月25日、共通のはじめにの音声・字幕付き動画をACDR0206で更新。表紙＋本文9枚、約22分39秒、字幕253cue。スライドと原稿に一致することを照合し、ブラウザー再生も確認。[変更記録38](../docs/workshops/ai-workshop/review/shared/changes/2026-10-25-38-intro-video-release.md)を参照。

2026年10月25日、オリエンテーションを先に見直すことに同意。現行14枚のLunaレビューと助言を受け、既存図を活かす構成案を同じ比較HTMLへ提示。構成案は確認待ちで、図と原稿は未修正。正本と音声動画は変更していない。[変更記録39](../docs/workshops/ai-workshop/review/shared/changes/2026-10-25-39-orientation-story-assessment.md)を参照。

2026年10月26日、オリエンテーション全14枚を作業コピーで修正し、利用者が1枚ずつ確認。受講者（Haiku 5.5）のレビューを反映し、ACDR 0207の承認を受けて正本へ反映した。定員は先着15〜20名に変更。音声・動画と企画書の見直しは未着手。[変更記録26](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-26-orientation-canonical-release.md)を参照。

2026年10月26日、オリエンテーションの音声・字幕付き動画をACDR 0208で更新（全14枚、約16分26秒、字幕187件）。共通の閲覧ページもオリエンテーションの正本を指すよう生成元を修正。[変更記録27](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-27-orientation-video-release.md)を参照。
