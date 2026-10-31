# UI フレームワークの調査結果：Angular ・ Svelte ・ React

## 結論

**AI だけでコードを書く開発の UI フレームワークには、Angular を選ぶ。**

選ぶ条件は「ルールが厳格で、書き方が公式で決まっていること」である。3つのうち、この条件をいちばん満たすのは Angular だった。

| 条件 | Angular がどう満たすか |
|---|---|
| 書き方が公式で決まっている | ルーター ・ フォーム ・ HTTP ・ 試験 ・ ビルドを、すべて公式が提供している |
| AI が書いたものの合否を、ツールで決められる | コンパイラの設定と angular-eslint で、古い書き方と型の誤りをエラーにできる |
| UI の部品を公式から使える | ダイアログ ・ フォーカス ・ キーボード操作を、公式の CDK と Angular Aria から使える |
| AI が最新の書き方で書く | 公式が、AI 向けの指示（best-practices.md）と Agent Skills を出している |

この文書の読み手は、UI フレームワークを選ぶ人と、選んだ理由をあとから確かめる人である。

## 調べ方

**3つとも、公式の資料の原文を取得し、識別子と引用を原文の文字列で照合した。** 取得した日はすべて 2026-10-31 である。

| 対象 | 読んだもの | 照合の結果 |
|---|---|---|
| Angular | angular/angular（commit `74961c6`）・ angular/angular-cli ・ angular/components ・ angular-eslint ・ angular/skills（commit `588a377`） | 識別子 73 個のうち 70 個が原文にあった（無かった 3 個は、recommended に含まれないことを確かめるための照合）。引用 91 件はすべて一致した |
| Svelte | svelte.dev の llms.txt（svelte ・ kit ・ cli ・ ai）・ sveltejs/svelte ・ kit ・ eslint-plugin-svelte ・ language-tools ・ cli | 識別子 76 個のうち 75 個が原文にあった。引用 82 件はすべて一致した |
| React | reactjs/react.dev（commit `0821e7d`）・ facebook/react（commit `e84e149`）・ eslint-plugin-jsx-a11y ・ npm の版の情報 | 識別子 86 個と引用 106 件が、すべて原文にあった |

原文は `.brainstorming/ui-framework/sources/` の `angular/` ・ `svelte/` ・ `react/` に保存してある。angular.dev の公開ページは中身を返さなかったので、Angular は GitHub の main ブランチ（22.3.0-next）の docs を読んだ。

## 3つの比較

**差が出たのは、公式が決めている範囲と、UI の部品と、AI 向けの公式の仕組みである。** 型の検査は、3つともテンプレートまで行える。

| 観点 | Angular 22.2.4 | Svelte 5.57.3 ／ SvelteKit 3.1.1 | React 19.3.1 |
|---|---|---|---|
| ルーター ・ データ取得 ・ フォーム | すべて公式。Signal Forms は stable | ルーティングとデータ取得は公式。新しいフォームの仕組み（remote functions）は experimental | 本体は決めていない。Next.js などのフレームワークから始めるよう勧めている |
| 古い書き方をエラーにする | `strictStandalone` と angular-eslint の `prefer-standalone` ・ `prefer-control-flow` でエラーにする | runes モードにすると `export let` ・ `$:` がコンパイルエラーになる。runes モードが既定になるのは Svelte 6 から | eslint-plugin-react-hooks の約 17 本のルールで検出する。`exhaustive-deps` の既定は warn |
| 警告をエラーにする | `extendedDiagnostics` の設定で `error` にできる | `sv check --fail-on-warnings` のフラグだけでできる | lint の重大度で決める。React Compiler は本番で `panicThreshold: 'none'` にするよう原文が指示している |
| アクセシビリティの検査 | angular-eslint の accessibility の設定で 11 本、すべて error | コンパイラの `a11y_` 警告が 42 個 | 第三者の eslint-plugin-jsx-a11y |
| ダイアログ ・ フォーカスの部品 | 公式の CDK（Dialog ・ FocusTrap ・ LiveAnnouncer）と Angular Aria | 公式には無い | 第三者の React Aria ・ Radix UI |
| 版の更新 | メジャー版は年1回。deprecated にしてから 12 か月以上は残す。`ng update` が書き換える | `sv migrate` | 間隔の決まりは無い。前の版で警告を出してから変える。codemod がある |
| AI 向けの公式の仕組み | best-practices.md ・ Agent Skills ・ CLI の MCP サーバー（experimental） | MCP サーバー ・ `svelte-autofixer` ・ skills | llms.txt と、AI エージェント向けの DevTools のツール `react-devtools-cdt-mcp`（experimental） |
| 部品1つのコード量（候補を選んで承認する部品の例） | 30 行 | 23 行 | 27 行 |

コード量の例は、調査とは別に書いたもので、ビルドして確かめていない。

### Svelte を選ばなかった理由

**ダイアログ ・ フォーカスの部品と、フォームの仕組みを、自分たちで決めて作ることになるからである。** 原文は "you are still responsible for making sure your application code is accessible." と書いている。SvelteKit 3 で設定ファイルが `svelte.config.js` から `vite.config.js` へ移り、eslint-plugin-svelte の文書がまだ古い置き場所を前提にしているという食い違いもある。

コードの短さと、`svelte-autofixer` を AI の作業のループの中で呼べる点は、Angular より優れている。

### React を選ばなかった理由

**ルーター ・ データ取得 ・ フォーム ・ スタイルを、本体が決めていないからである。** 原文は "does require that you make choices on which tools to use for routing, data fetching, and other common usage patterns." と書いている。選んだ構成を固定するルールを、自分たちで作って維持することになる。

React Compiler 1.0 と eslint-plugin-react-hooks 7 を使えば、hooks の誤りは Angular と同じ程度に lint で検出できる。AI が最初の1回で正しく書ける割合がいちばん高いのも、React だと考えられる（推測）。

## Angular の厳格化の仕組み

**書き方の誤りは、コンパイラの設定と lint でエラーにする。** 設定キーとルール名は、すべて原文で照合した。

| 種類 | 設定キー ・ ルール名 | エラーにするもの |
|---|---|---|
| コンパイラ | `strictTemplates`（既定で有効） | テンプレートの型の誤り |
| コンパイラ | `strictStandalone` | standalone でない部品 |
| コンパイラ | `strictInjectionParameters` ・ `strictInputAccessModifiers` | 型が決まらない注入、`private` な input への書き込み |
| コンパイラ | `extendedDiagnostics`（`defaultCategory` を `error`） | 呼び忘れた signal など、18 件の拡張診断 |
| lint（recommended） | `prefer-standalone` ・ `prefer-inject` ・ `prefer-on-push-component-change-detection` ・ `template/prefer-control-flow` | NgModule ・ コンストラクターでの注入 ・ `*ngIf` などの古い書き方 |
| lint（個別に有効にする） | `prefer-signals` ・ `no-developer-preview` ・ `no-experimental` | decorator の input ・ 安定していない API |
| lint（accessibility） | `template/alt-text` ・ `template/label-has-associated-control` ・ `template/click-events-have-key-events` など 11 本 | アクセシビリティの誤り |
| lint（typescript-eslint） | `no-explicit-any` ・ `template/no-any` | 型の検査を無効にする書き方（`any` ・ `$any()`） |

コンパイラと lint で検出できない誤りは、別の検査で検出する。

| コンパイラと lint で検出できない誤り | 検出する検査 |
|---|---|
| 振る舞いの誤り（例：承認の前に承認ボタンが出る） | spec の単体テスト。`ng generate` が作る spec は「部品が作れる」ことしか確かめないので、振る舞いの試験は、AI PdM の Blueprint の設計書（`docs/08_IMPLEMENTATION_HANDOFF.md`）と受け入れの条件（`docs/09_POC_ACCEPTANCE_AND_TEST_PLAN.md`）から書く |
| 見た目の誤り | Playwright でライト ・ ダーク ・ 狭い幅のスクリーンショットを撮り、差分を取る |
| AXE が検出するアクセシビリティの誤り（best-practices.md が "It MUST pass all AXE checks." と求めている） | AXE の検査。公式の Skill の手順には入っていないので、試験に足す |

### AI が書いたときに起きること

**AI が誤った書き方をしても、検査がエラーを返し、AI がそのエラーを読んで直す。** 次の表は、決める前と決めたあとで、同じ誤りがどう扱われるかを並べたものである。

| AI が書いたもの | 決める前 | 決めたあと |
|---|---|---|
| `*ngIf` で条件を書いた | 何で検査するかが決まっていないので、古い書き方のまま残る | `template/prefer-control-flow` がエラーを返し、AI が `@if` に書き直す |
| `@Input()` のデコレーターで入力を書いた | 同じく残る | `prefer-signals` がエラーを返し、AI が `input()` に書き直す |
| 型を `any` にした | 同じく残る | `no-explicit-any` がエラーを返し、AI が型を書く |
| クリックで開くメニューを `<div>` で作り、キーボードで操作できない | 画面を触った人が気づくまで残る | `template/click-events-have-key-events` と AXE の検査がエラーを返し、AI が `<button>` か Angular Aria の Menu に直す |
| 候補を選ぶ前に承認ボタンを表示した | 画面を触った人が気づくまで残る | spec の試験が失敗し、AI が表示の条件を直す |
| 狭い幅で承認ボタンがはみ出した | 画面を触った人が気づくまで残る | Playwright のスクリーンショットの差分が出て、人が見て合否を決める |

最後の行だけは、合否を人が決める。見た目の正しさは、どの検査でも決められないためである。

## AI 向けの公式の仕組み

**best-practices.md は AI に正しい書き方を教えるもので、守らせるものではない。** 守ったかどうかは、上の厳格化の仕組みで確かめる。

| もの | 置き場所 | 中身 |
|---|---|---|
| best-practices.md | angular/angular の `packages/core/resources/best-practices.md` | standalone ・ signals ・ `@if`/`@for` ・ `input()`/`output()` ・ Signal Forms ・ `inject()` を使うこと、`any` ・ `ngClass` ・ `CommonModule` を使わないこと、WCAG AA と AXE を満たすこと。`ng new --ai-config=claude` などで、AI ツールごとの設定ファイルとして置かれる |
| Agent Skills | github.com/angular/skills | `angular-developer`（領域ごとの references 44 本の索引）と `angular-new-app`。Agent Skills の公開の仕様に沿っているので、Claude 以外の AI ツールでも使える。`angular-developer` は、書き終えたら `ng build` が通るまで直すことを手順にしている |

## Angular を選ぶときの制約

**AI が最初の1回で正しく書ける割合は、React より低い。** 学習しているコードの量が少なく、古い書き方（NgModule ・ `*ngIf`）のコードが多く含まれるためである（推測）。古い書き方は lint とコンパイラがエラーで返すので、できあがるものの正しさは下がらない。

| 制約 | 中身 |
|---|---|
| ネイティブのモバイルアプリ | 公式が提供する手段は PWA（`ng add @angular/pwa`）までである。React は推奨の一覧に Expo を挙げている |
| ファイルの数 | `ng generate component` は ts ・ html ・ css ・ spec の4つを作る |
| 拡張診断の増え方 | 公式は "The Angular team intends to add or enable new extended diagnostics in **minor** versions of Angular" と書いている。版を上げると、新しい警告やエラーが出ることがある |

デザインの自由さは、3つとも変わらない。CDK の Dialog と Angular Aria は見た目を持たないので、AI PdM の Blueprint（`AI_PdM_Purpose_Discovery_Blueprint_v0.9.zip`）の `design-system/tokens/` にあるデザイントークン（`palette-tokens.json` ・ `themes.css`）で、見た目を作れる。見た目が Material Design になるのは、Angular Material を使った場合だけである。

## 確かめていないこと

| 項目 | 状態 |
|---|---|
| 安定版 22.2.x の公開ページの文言 | main ブランチ（22.3.0-next）の docs で読んだ。公開ページとの一致は確かめていない |
| best-practices.md の禁止事項のうち、`ngClass` ・ `CommonModule` ・ `@Service` に対応する lint のルール | 照合していない |
| standalone が既定になった版 | best-practices.md は v20 から、部品の解説ページは v19 からと書いていて、原文どうしが食い違っている |
| この文書のコード例 | ビルドしていない |
