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
├── scripts/              # Python・Shell等のヘルパースクリプト
│   └── ...
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
| `scripts/` | 自動化スクリプト（Python/Shell） | バリデーション、生成、ビルド処理が必要なとき |
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
├── scripts/
│   └── validate.sh
└── agents/
    └── sub-agent.md
```

## ルール

- `SKILL.md` は必ずスキルフォルダのルートに置く
- テンプレートファイルは `references/` に置く（`assets/` ではない）
- スクリプトは `scripts/` にまとめ、SKILL.md からパスで参照する
- 不要なフォルダは作らない（使うものだけ作る）
