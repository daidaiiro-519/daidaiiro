# オリエンテーションの正式反映

利用者が2026年10月26日に全14枚を1枚ずつ確認し、全14枚の通し確認と受講者（Haiku 5.5）のレビューを反映した版を、正本へ反映する。対象はオリエンテーションのスライドと原稿である。共通の「はじめに」、教材本編、企画書、音声・動画の制作は含まない。

現行の正本は、旧教材名（目的の把握・言葉の定義・仕組みの構築）と、仕事の目的をAIで達成するという旧方針のままである。確認済みの版は、共通の「はじめに」と共通方針revision 2026-10-25-13に合わせ、仕事全体の生産性と品質を高める到達点と、目的と要件の明確化・業務のモデル化と設計・AI活用の実践と改善の三教材を説明する。

## 反映する差分

| 対象 | 理由 |
|---|---|
| `docs/workshops/ai-workshop/material-1/src/build_lessons.py` | 2枚目：開催の背景・位置づけ・目指す姿の構成を維持し、三教材の名称と、自分の仕事に合うSkillsを作りAgentで作業を進める目指す姿に合わせる。Skillsとエージェントを初出の原稿で定義する。 |
| `docs/workshops/ai-workshop/material-1/src/derived_visuals.py` | 1枚目：表紙を、仕事全体の生産性と品質を高める到達点と、共通の「はじめに」の教材名・内容に合わせる。表紙専用の原稿を入力に持たせ、本文の原稿を流用しない。 |
| `docs/workshops/ai-workshop/material-1/src/ledes.json` | オリエンテーション13枚の導入文を、見出しや図と同じ文を繰り返さず、その枚の役割を一文で述べる確認済みの版に改める。共通の「はじめに」と教材本編の導入文は変えない。 |
| `docs/workshops/ai-workshop/material-1/src/lesson-00-start.json` | 旧教材名、言葉の定義が中心の教材2、条件の記録としてのSkillsなど、共通の「はじめに」と共通方針に食い違う見出しと原稿を、利用者が1枚ずつ確認した全14枚の版に改める。資料名を表紙の見出しに合わせ、表紙専用の原稿を加える。 |
| `docs/workshops/ai-workshop/material-1/src/orientation_boundary_visuals.py` | 2枚目の図を、開催の背景・位置づけ・目指す姿の構成のまま三教材の名称とSkills・Agentの目指す姿に合わせる。14枚目の図は、旧教材名と目的の達成を描く共用の関数を使わず、表紙と同じ到達点へつなぐ形をこの関数で描く。 |
| `docs/workshops/ai-workshop/material-1/src/orientation_support_visuals.py` | 11枚目：教材名、参加者が載せるSkillsと記録、載せる項目を現在の教材に合わせる。トライアルとアカウントの運営条件は変えない。 13枚目：学ぶ場所・相談する場所・実践に使う環境を図の順に示し、人物を枠で囲み、Teamsでの相談を双方向にし、定員を15〜20名にする。 |
| `docs/workshops/ai-workshop/material-1/src/reference_slide_02.py` | 3枚目：作成から確認・修正を経て完了するまでを仕事全体として示し、工数・案件の期間・必要な品質で効果を問う。2枚目と5枚目に重なる説明を外す。 |
| `docs/workshops/ai-workshop/material-1/src/reference_slide_03.py` | 4枚目：人が整理して書いていた作業の一部をAIが担い、同じ時間で作れる案や資料の数が増える変化に絞り、5枚目の確認の話へつなぐ。 |
| `docs/workshops/ai-workshop/material-1/src/reference_slide_04.py` | 5枚目：3枚目と同じ問いをやめ、成果物が増えると照合する対象も増え、確認と修正の時間が同じように減るとは限らないことを示す。職種の固定分担に見える表示を外す。 |
| `docs/workshops/ai-workshop/material-1/src/reference_slide_05.py` | 6枚目：示し方で確認と判断のしやすさが変わることと、何に使い何を確かめるかを人にもAIにも説明できる必要を示し、7枚目へつなぐ。 |
| `docs/workshops/ai-workshop/material-1/src/reference_slide_06.py` | 7枚目：共通方針と「はじめに」に合わせ、Skillsを条件の記録ではなく、参照する情報・判断の観点と確認・照合の手順として示し、Agentの照合と人による根拠の確認を加える。 |
| `docs/workshops/ai-workshop/material-1/src/reference_slide_07.py` | 8枚目：旧教材名を共通の「はじめに」の名称に改め、教材2を業務のモデル化と設計、教材3を判断の観点と確認の手順をSkillsにまとめて使う内容にする。 |
| `docs/workshops/ai-workshop/material-1/src/reference_slide_08.py` | 9枚目：資料作成の例を進捗の整理に、開発の例を業務のルールとの照合と設計の見直しにし、6枚目と7枚目との重なりを外す。 |
| `docs/workshops/ai-workshop/material-1/src/reference_slide_09.py` | 10枚目：教材名を改め、DDDを言葉の定義からモデルと設計へ、ユースケースを目的レベルと範囲へ合わせ、書籍を先に読まなくてよいことを導入文で示す。 |
| `docs/workshops/ai-workshop/material-1/src/reference_slide_10.py` | 12枚目：目的の達成から何が改善したかを確かめる内容に改め、確認と修正の振り返りを加え、定員を先着15〜20名にする。 |
| `docs/workshops/ai-workshop/material-1/out/slides/lesson-00-start.html` | 上の入力から生成したスライド全14枚。利用者が確認した比較用のスライドと、書籍の表紙の参照方法を除いて一致する。 |
| `docs/workshops/ai-workshop/material-1/out/scripts/lesson-00-start-script.md` | 上の入力から生成した読み上げ原稿。作業コピーで確認した原稿と一致する。 |

## 変えないもの

共通の「はじめに」（lesson-01-lead）は、同じ生成元から生成し直しても出力が変わらないことを確認した。企画書の入力、教材本編、音声・動画は変更しない。企画書に残る旧教材名と定員、Kiroの費用は、企画書の見直しで扱う。
