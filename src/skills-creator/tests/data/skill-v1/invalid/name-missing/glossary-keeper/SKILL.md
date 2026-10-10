---
description: 用語集を JSON のインスタンスで持ち、語を足す ・ 直す ・ 検査するときに使う。
---

# 用語集を JSON で持つ Skill：glossary-keeper

## 目的
用語の定義を、検証できる形で1か所に持つ。

## 役割
語を足す ・ 直す ・ 検査する。

## 入力の想定
足す語と、その定義。

## 実行手順
### Step 1: 語を足す
scripts/schema-driven --skill . create --schema assets/term.schema.json --path terms/<語>.json

## 出力形式
作ったインスタンスのパス。

## ガードレール
定義の無い語を足さない。
