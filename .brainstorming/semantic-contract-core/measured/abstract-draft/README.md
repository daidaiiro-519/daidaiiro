# Abstract の本体の案と、ACDR での試し

Q3（Agents 契約）の検討から根幹 Q0 に戻って整理した決定を、Abstract の本体と ACDR の Concrete に入れた案。ボードにはまだ当てていない。

| ファイル | 中身 |
|---|---|
| abstract.settled.json | Q7 で決めた本体（比べる元） |
| abstract.json | 案の本体。$schema は自分の $id で、自分を検査する |
| concrete.json | ACDR の Concrete（urn:semantic-agent-schema:acdr:decision-record:v1）。build.py が書く |
| instances.json | ACDR 0197 の3つの姿（proposed ・ 書き足し後 ・ accepted）。build.py が書く |
| build.py ・ tpl.html | シミュレーション simulation.html を組む |
| abs_page.py | 完成イメージ abstract-and-concrete.html を組む |
| selfcheck.py | jsonschema で、本体を本体自身で、Concrete を本体で、インスタンスを Concrete で検査する |

組み直しと検査：

```
python3 build.py && python3 abs_page.py && python3 selfcheck.py
```
