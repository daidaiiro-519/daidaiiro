# 相談の材料：配布物への署名の方式と、導入時の確かめ方

## 前提（決着済み）
- 基盤 schema-driven のエンジンは、署名付きの実行ファイルとして GitHub Release で配る（ボード skill-roles の Q3）。
- 作るのは1人。リポジトリ github.com/daidaiiro-519/daidaiiro は公開リポジトリ。
- 利用者は Claude Code の利用者で、Skill を導入スクリプトで入れる。導入スクリプトは install.sh（macOS ・ Linux ・ WSL）と install.ps1（Windows）。

## いまの配る仕組み（実在の .github/workflows/release.yml）
- tag `<Skill の名前>-v<版>` で、その Skill を公開する。
- 3つの OS（Linux musl ・ Windows ・ macOS universal2）で組み立て、起動を試し、梱包し、別のランナーで導入スクリプトから展開して起動を試してから、GitHub Release に公開する。
- 配布物の SHA-256 の一覧（SHA256SUMS）を一緒に公開する。導入スクリプトは https で取得して SHA-256 を照合する。
- 署名は無い。immutable releases は無効（`"enabled":false`）。

## 原文で分かったこと（引用は照合済み）
| 方式 | 要点 |
|---|---|
| GitHub artifact attestations（actions/attest） | 鍵を持たない（Sigstore の短命証明書）。何を・どのリポジトリ・どのコミット・どのワークフローで作ったかを証明する（SLSA v1.0 Build Level 2）。公開リポジトリならどのプランでも使える。公開リポジトリの記録は公開の透明性ログにも書かれる。Actions に id-token: write ・ attestations: write ・ contents: read（README は artifact-metadata: write も挙げる）。利用者は `gh attestation verify <file> -R <owner/repo>`。bundle を渡さないときは gh のログインかトークンが要るとコードから読める。bundle と trusted root を持ち込めばオフラインで確かめられる。trusted root には期限が無く、失効を知る手段が無い。「安全の保証ではない」と原文が明記 |
| immutable releases | 公開したあと、アセットと tag を変えられなくする。リリースの証明（tag ・ コミット SHA ・ アセット）が自動で付く。設定で有効にし、下書きにアセットを付けてから公開する。利用者は `gh release verify-asset <tag> <file>` |
| cosign keyless（sign-blob） | 鍵を持たない。署名は Rekor の透明性ログに残る。bundle でオフライン検証できる。利用者は cosign（3 OS の版あり）。漏れへの対応は作者がログを監視する |
| minisign | Ed25519。作者がパスワードで守った秘密鍵を持ち、Actions の秘密に置く必要がある。利用者は minisign（3 OS の版、scoop ・ choco ・ brew）。Rust の検証用 crate がある。失効の仕組みは原文に無い |
| GPG | 作者が鍵を持つ。失効証明書と keyserver。利用者は gpg（Windows は Gpg4win） |
| cargo-dist | attestations は既定で無効、設定で有効。検証は gh attestation verify だけ。Sigstore 署名は予定 |

## 決めたいこと
1. 署名（または証明）の方式
2. 導入スクリプトで、署名の確かめを必須にするか（道具が無ければ導入を止める）、道具がある人だけ確かめるか
3. immutable releases を有効にするか
