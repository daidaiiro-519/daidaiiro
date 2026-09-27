# Skills ベストプラクティス フォルダ構成

## 標準構成

```
.claude/skills/{{スキル名}}/
├── SKILL.md              # 必須・ルートに固定
├── README.md             # 任意・人間向け説明
├── references/           # Claudeが読む参考ドキュメント・スケルトンテンプレート
│   ├── skill-template.md
│   └── ...
├── examples/             # 使用例・サンプル出力
│   └── ...
├── rs/                   # 道具（Rust の workspace。道具の契約に従う）
│   └── ...
├── mcp.json              # 道具の MCP の登録（道具を持つときだけ）
├── agents/               # サブエージェント定義
│   └── ...
└── assets/               # HTML等の静的ファイル
    └── ...
```

## 各フォルダの用途

| フォルダ | 用途 | 使うケース |
|---|---|---|
| `references/` | Claudeが実行時に読む文書。スケルトンテンプレート、仕様書、定義票など | ほぼ全てのスキルで使う |
| `examples/` | サンプル入出力、使用例 | ユーザーへの説明や参考が必要なとき |
| `rs/` | 道具。`parts` ・ `declare` ・ `cli` ・ `mcp` の4つの crate（`references/tool-contract.md`） | 検査、生成、組み立ての処理が必要なとき |
| `agents/` | Claudeが呼び出すサブエージェントの定義 | 複数の専門エージェントに処理を分担させるとき |
| `assets/` | HTMLビューアー、画像等の静的ファイル | UIやレポート生成が必要なとき |

## ミニマム構成（シンプルなスキル）

```
.claude/skills/{{スキル名}}/
├── SKILL.md
└── references/
    └── template.md
```

## フル構成（複雑なスキル）

```
.claude/skills/{{スキル名}}/
├── SKILL.md
├── README.md
├── references/
│   ├── template-a.md
│   ├── template-b.md
│   └── definitions.md
├── examples/
│   └── sample-output.md
├── rs/
│   ├── Cargo.toml
│   ├── parts/
│   ├── declare/
│   ├── cli/
│   └── mcp/
├── mcp.json
└── agents/
    └── sub-agent.md
```

## ルール

- `SKILL.md` は必ずスキルフォルダのルートに置く
- テンプレートファイルは `references/` に置く（`assets/` ではない）
- 道具は `rs/` にまとめ、SKILL.md から入口の名前で参照する。**`scripts/` を置かない** ── 契約の検査が「Python が残っている」として検出する
- 道具の一式は `skills-creator scaffold <スキル名>` が置く
- 不要なフォルダは作らない（使うものだけ作る）
