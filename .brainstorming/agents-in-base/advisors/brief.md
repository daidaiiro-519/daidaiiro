# 相談の材料：基盤 schema-driven に「AI の CLI をプロセスで呼ぶ」契約と「意味の検証」を持たせる

## 背景（2026-10-17 時点の事実）
- 基盤 schema-driven（0.1.1 公開済み、src/schema-driven、Rust、ヘキサゴナル）は abstract スキーマとエンジン（create ・ get ・ update ・ delete ・ prompt ・ check ・ check-schemas ・ approve ・ render）を持つ。check は形（JSON Schema）・参照（x-ref）・導出値（x-derive）・コントラストを機械的に見る。結果は合格 ・ ずれ ・ 確かめ直し。approve は検査を通ったインスタンスのハッシュを approval.json に書く。
- 基盤の SKILL.md のガードレールに「基盤はネットワークに出ない」「同じ入力から同じ結果（render）」がある。基盤のエンジンは skills-creator が各 Skill の scripts/bin/ に照合して入れる（全 Skill に届く）。
- concrete スキーマはプロパティごとに x-prompt を持つ。x-prompt.read（means ・ parse ・ shown ・ when）は値の読み方、x-prompt.write は決まりの配列で、1件が id ・ rule（決まり）・ why（なぜ）・ test（守れているかを何で見るか）・ 任意の bad/good を持つ。
- brainstorming（0.1.0、ACDR 0183、今日公開）は、画面の「AI に聞く」で serve がログイン済みの Claude Code ・ Codex ・ Kiro を子のプロセスとして呼ぶ。実装は src/brainstorming の domain/agent.rs（呼び方と、3つの CLI の出来事を共通の形に直す）・ domain/scope.rs（作業の前後で範囲の外の変更を見つける）・ application/chat.rs（写す ・ 起動 ・ 中継 ・ 止める ・ 時間切れ ・ serve 自身が check ・ 描き直す ・ 記録）・ outbound/agent.rs（シェルを通さず起動、子孫まで止める）。権限を外す旗は使わない。Claude は --allowedTools で Skill の実行ファイルだけ、Codex は workspace-write の sandbox、Kiro は --trust-tools。3つとも手元で動いた。
- 実測：Claude は許していないコマンドを1回断られると Bash 全体が使えないと受け取って止まった（最初の依頼に「シェルで使えるのは X だけ」と書くと最後まで動いた）。Codex は AGENTS.md と .agents/skills/ を、Kiro は .kiro/skills/ を自動で読み、.claude/skills/ は読まない。

## 利用者が決めた方向（2026-10-17、前提として固定）
1. 基盤が持つべき契約は、Agents（AI の CLI）をプロセスで呼べること。
2. schema 駆動における意味の検証は、そのプロセスを呼ぶことで、自身のスキーマから作成したインスタンスに対して行えること。
3. そのとき、x-prompt と外からのコンテキストをベースにして意味の検証がされること。
- 利用者の動機：ACDR でも AI に聞く画面が欲しい。ほかの Skill でも使う。検証には、スキーマで機械的に見られる部分と、意味の部分がある。

## いまの見解（反証してほしい）
- Q0 完成の条件：基盤の新しい操作（仮に semantic）で、brainstorming のボードと ACDR の記録の意味の検証を、3つの CLI で行え、結果が判断の記録として残り、brainstorming の AI に聞くが基盤の能力の上で今と同じに動けば完成。
- Q1 基盤の AI の契約：domain/agent.rs ・ scope.rs ・ chat.rs の中身（呼び方 ・ 出来事の共通の形 ・ 権限の最小化 ・ 作業前の写し ・ 範囲の検査 ・ 止め方と時間切れ）を基盤へ移す。serve の HTTP の中継まで基盤に入れるかは未定（render は基盤にあるので、配る serve も基盤に置く案がある）。
- Q2 意味の検証の形：check（決まった合否）とは別の操作にする。入力はインスタンスと、そのスキーマの x-prompt（write の rule ・ why ・ test と read）。出力は合否ではなく「判断の記録」のインスタンス（新しい abstract のスキーマ。どの AI が ・ どの決まり（プロパティの場所と rule の id）について ・ 守れている／守れていない／判断できない ・ 根拠の引用 ・ 直し方の案）。結果は揺れるので、approve の条件には入れず、人が判断の記録を確かめて承認する。
- Q3 外からの文脈：呼ぶ側が渡す（CLI の引数でファイルやフォルダを指す）。スキーマの側で「この欄は何を文脈に要るか」を宣言する案もある（x-context のような注釈）。
- Q4 移行と約束：基盤の「ネットワークに出ない」を「基盤自身はネットワークに出ない。AI の CLI を呼ぶ操作だけは、利用者の手元のログイン済みの CLI を通して外に出る」に改める。基盤の版を 0.2.0 にし、abstract に判断の記録のスキーマを足す。brainstorming は基盤の能力を使う形に移し、ACDR の作り直しで使う。

## 聞きたいこと
上の見解 Q0〜Q4 を、それぞれの専門の判断基準で反証し、採ってよいか、足りない所、別の案を出してほしい。
