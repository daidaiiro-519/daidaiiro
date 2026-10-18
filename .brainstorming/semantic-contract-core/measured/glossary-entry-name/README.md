# 用語集の1件を何と呼ぶか：候補ごとの原典と、ずれ

取得日：2026-10-18。原文は `sources/` に保存した。引用は原文の文字列のままで、`fact-check verify` で照合した（照合の方法が違うものは、その行に書く）。

## 1. 呼ぶ対象（この1件の中身）

| 性質 | 中身 |
|---|---|
| 置き場所 | 文脈ごとの glossary（JSON Schema の文書）の `$defs`。`$anchor` で名前を付ける |
| 名前と意味 | `title`（人が読む名前）と `description`（意味の説明）を分けて持つ |
| 値の形 | `type` ・ `pattern` などを持つ件と、持たない件（例「承認者」）がある |
| 単位 | 持つ件だけが持つ（例 秒） |
| 使われ方 | 各スキーマの欄が `$ref` で指す。欄は隣に制約を足して範囲を絞れる |
| 版 | ほかの文脈へ渡す件だけが持つ |
| 持たないもの | 振る舞い（業務ルール ・ 制約の文）。rule が持つ |

## 2. 候補ごとの原典の引用

| 候補 | 出どころ | 原文 |
|---|---|---|
| term | W3C SKOS Primer 2.1 | "concepts exist in the mind as abstract entities which are independent of the terms used to label them." |
| term | ISO/IEC CD 11179-4（JTC 1/SC 32 N 0755、2002年の委員会原案）1 Scope | "terms in a language dictionary may have multiple definitions in the dictionary, whereas data definitions have a single meaning." |
| term | 同 5.2 e) | "Definitions of terms should be provided in an associated glossary." |
| glossary term（DataHub） | DataHub Business Glossary | "**Terms**: words or phrases with a specific business definition assigned to them." |
| glossary term（DataHub） | 同 | "you can define a very specific, concrete Glossary Term like `Email Address` to represent a physical data type" |
| glossary term（OpenMetadata） | OpenMetadata What is a Glossary Term | "A Glossary Term is a preferred terminology for a concept." |
| glossary term（OpenMetadata） | 同 | "**Description** - A unique and clear definition to establish consistent usage and understanding of the term. This is a mandatory requirement." |
| glossary term（OpenMetadata） | 同 | "**Assets** - After creating a glossary term, data assets can be associated with the term, which helps in data discovery." |
| data element | ISO/IEC CD 11179-4 3.4 | "data element: A unit of data for which the definition, identification, representation and permissible values are specified by  means of a set of attributes." |
| data element concept | Wikipedia「ISO/IEC 11179」（規格の本文ではない） | "The core object is the data element concept, since it defines a concept and, ideally, describes data independent of its representation in any one system, table, column or organisation." |
| value domain | 同 | "A value domain is the permitted range of values for a characteristic of an object class." |
| concept | W3C SKOS Reference 3.1 | "A SKOS concept can be viewed as an idea or notion; a unit of thought." |
| concept | 同 | "The notion of a SKOS concept is useful when describing the conceptual or intellectual structure of a knowledge organization system, and when referring to specific ideas or meanings established within a KOS." |
| concept（名前） | SKOS Reference 5.1 | "A lexical label is a string of UNICODE characters, such as "romantic love" or "れんあい", in a given natural language, such as English or Japanese (written here in hiragana)." |
| concept（名前） | 同 | "These labels provide the strongest clues as to the meaning of a SKOS concept." |
| concept（識別子） | SKOS Reference 6.1 | "A notation is a string of characters such as "T58.5" or "303.4833" used to uniquely identify a concept within the scope of a given concept scheme." |
| concept（意味） | SKOS Primer | "skos:definition supplies a complete explanation of the intended meaning of a concept." |
| value object | ddd-advisor criteria.json（value-object） | 「内部のフィールドの値の組み合わせで識別するオブジェクト。」 |
| value object | 同（性質の表） | 「メソッドは値を操作し、その結果を新しい値オブジェクトとして返す」 |
| 用語集 | ddd-advisor criteria.json（同じ言葉を育てる） | 「用語集はモノ ・ プロセス ・ 役割などの名詞の整理に役立つ。振る舞いは動詞の一覧ではなく、業務ルール ・ 前提条件 ・ 制約で、用語集の形では表せない」 |
| ユビキタス言語 | 同（同じ言葉の条件） | 「1つの用語に1つの意味」 |
| definition | ISO/IEC CD 11179-4 3.5 | "definition:  A word or phrase expressing the essential nature of a person or thing or class of persons or things: an answer to the question "what is x?" or "what is an x?"; a statement of the meaning of a word or word group" |
| definition | JSON Schema Core 2020-12 8.2.4 | "The "$defs" keyword reserves a location for schema authors to inline re-usable JSON Schemas into a more general schema." |
| 名前 | ISO/IEC CD 11179-4 3.8 | "name:  The primary means of identification of objects and concepts for humans." |

照合の方法の注記：CD 11179-4 は Word 形式（.doc）で、`fact-check verify` は文字として読めなかった。上の4行は、取得した .doc の本文を Read で開き、目で原文と照合した。

## 3. 合う点とずれる点

| 候補 | 名前と意味を分けるか | 値の形 ・ 単位 | 値の形を持たない件 | `$ref` で指される | 振る舞いを持たない |
|---|---|---|---|---|---|
| term | ずれる。SKOS と 11179-4 では term は名前（ラベル）の側で、意味の単位ではない | 原典に無い | 合う | DataHub ・ OpenMetadata では資産に付ける。合う | 合う（DDD の用語集は名詞を整理する） |
| glossary term | 合う。OpenMetadata は Description を必須にし、term は concept を指す名前とする | 原典に無い。DataHub は Glossary Term で「physical data type」を表す例を挙げるが、型を持つ欄は書いていない | 合う | 合う（data assets を term に関連づける） | 合う |
| data element | 合う（definition と identification を分ける） | 合う（representation ・ permissible values を持つ） | ずれる。representation を持たない「承認者」は data element にならない | ずれる。指す側の欄（隣に制約を足した欄）のほうが data element に近い | 合う |
| data element concept | 合う | ずれる。representation から独立させる | 合う | 合う | 合う |
| concept | 合う。prefLabel（名前） ・ skos:definition（意味） ・ notation（識別子）を分ける | 原典に無い | 合う | 合う（URI で指す） | 合う |
| value object | 対象外 | 合う（値と検証を持つ） | ずれる。「承認者」は値ではない | 対象外（コードの型） | ずれる。メソッド（業務ロジック）を内側に持つ |
| definition | ずれる。11179-4 では意味の文（description）だけを指す | ずれる | ずれる | ずれる | 対象外 |
| entry | 原典の定義が無い。中身を何も言わない | 原典に無い | 合う | 合う | 合う |

## 4. 紛れるおそれ

| 候補 | 紛れる相手 |
|---|---|
| term | このリポジトリの拡張キーワード `x-term`（Q2）と同じ語。指すものが同じなら紛れない |
| concept | doc-writing-skills の「概念8」のように、このリポジトリで「概念」を別の意味で使っている。`v03-concepts.json` も別の対象 |
| definition | JSON Schema の `$defs`、Concrete の `x-definitions` |
| data element | 11179 では1件ではなく、欄の側に当たる |
| value object | DDD のコードの部品（メソッドを持つ） |

## 5. 日本語の呼び名の候補

| 候補 | 日本語 |
|---|---|
| term | 用語 |
| glossary term | 用語集の用語（文書の中では「用語」で足りる） |
| concept | 概念 |
| data element | データ要素 |
| data element concept | データ要素概念 |
| value object | 値オブジェクト |
| definition | 定義 |
| entry | 項目 ・ 見出し語 |

## 6. 結論

| 候補 | 結論 | 理由 |
|---|---|---|
| glossary term | 条件付きで採れる | 用語集の製品で「意味を持ち、データ資産から指される1件」を指す。条件は、値の形と単位を持つことを glossary の定義に書くこと（原典の glossary term は型を持たない） |
| term | 条件付きで採れる | glossary の中では glossary term の短い形として通じる。条件は、名前は `title` が持ち、term は名前ではなく1件の全体を指すと書くこと（SKOS と 11179 では term は名前の側） |
| concept | 条件付きで採れる | 名前 ・ 意味 ・ 識別子を分ける構造は最も近い。条件は、値の形と単位の拡張を書くことと、リポジトリの「概念」の別の使い方と区別すること |
| data element | 採れない | 値の形を持たない件を含められない。11179 の data element に当たるのは、`$ref` で指して制約を足した欄の側である |
| data element concept | 採れない | 表現から独立させる定義なので、値の形と単位を持つ件と合わない |
| value object | 採れない | メソッド（業務ロジック）を内側に持ち、値でないもの（承認者）を含めない |
| definition | 採れない | 原典では意味の文だけを指し、`$defs` ・ `x-definitions` と紛れる |
| entry | 採れない | 原典の定義が無く、中身を何も言わない。文中で「1件」の意味に使うのは差し支えない |

## 7. 確かめられなかったこと

| 対象 | 状態 |
|---|---|
| ISO/IEC 11179-1:2015 ・ 11179-3:2013 の本文 | 取得していない。data element の定義は 2002年の委員会原案（CD 11179-4）から引いた。発行版の文言と同じかは確かめていない |
| 11179 の value domain が単位（unit of measure）を持つこと | 取得した範囲に原文が無く、確かめられなかった |
| data element concept の規格の定義文 | Wikipedia の説明だけで、規格の本文では確かめていない。メタデータ登録簿（metadata.nsw.gov.au）は 403 で取得できなかった |
| ISO 1087 の term の定義 | 取得していない |
| DataHub の glossary term が型 ・ 単位を持つ欄を持つか | 取得したページには無い。API のスキーマは見ていない |
| 『ドメイン駆動設計をはじめよう』の書籍本文 | ローカルの criteria.json（書籍から起こした判断基準）だけで書いた |
