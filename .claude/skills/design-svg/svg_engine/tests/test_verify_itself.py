"""検査そのものの検査 ── 壊れた図を渡して鳴ること、正しい図で鳴らないこと。

検査は2度、静かに壊れた。1度目は祖先の変換を積んでおらず全部が原点付近に
居ることになって偽の重なりを36件出し、2度目は折れ線の端点しか見ておらず
箱を突っ切る線を素通りさせた。どちらも「崩れ0件」という表示は出ていた。
だから、検査が実際に鳴ることを確かめる試験を、検査と同じだけ持つ。
"""
from __future__ import annotations

from svg_engine.verify import check, check_attachment, check_shapes


def _svg(inner: str, w: int = 400, h: int = 200) -> str:
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}">{inner}</svg>'


def _text(x, y, s, size=14):
    return f'<text x="{x}" y="{y}" font-size="{size}">{s}</text>'


def _box(x, y, w=80, h=40):
    return (f'<g class="svg-box" transform="translate({x},{y})">'
            f'<rect x="0" y="0" width="{w}" height="{h}"/></g>')


class TestTextChecks:
    def test_重なった文字で鳴る(self):
        assert any("重なる" in f for f in check(_svg(_text(10, 50, "あいうえお")
                                                     + _text(12, 52, "かきくけこ"))))

    def test_離れた文字では鳴らない(self):
        assert check(_svg(_text(10, 50, "あいうえお") + _text(10, 120, "かきくけこ"))) == []

    def test_画布の外へ出た文字で鳴る(self):
        assert any("画布の外" in f for f in check(_svg(_text(360, 50, "はみ出す文字列"))))

    def test_変換で運ばれた先も見る(self):
        """祖先の translate を積まないと、この2つは離れて見えてしまう。"""
        inner = (f'<g transform="translate(0,0)">{_text(10, 50, "あいうえお")}</g>'
                 f'<g transform="translate(2,2)">{_text(10, 50, "かきくけこ")}</g>')
        assert any("重なる" in f for f in check(_svg(inner)))

    def test_変換で離された文字では鳴らない(self):
        """逆に、変換を積まないと偽の重なりを出す形。"""
        inner = (f'<g transform="translate(0,0)">{_text(10, 50, "あいうえお")}</g>'
                 f'<g transform="translate(0,100)">{_text(10, 50, "かきくけこ")}</g>')
        assert check(_svg(inner)) == []

    def test_viewBoxが無ければそれ自体を崩れとする(self):
        assert check('<svg xmlns="http://www.w3.org/2000/svg"></svg>') == ["viewBoxが無い"]


class TestShapeChecks:
    def test_重なった箱で鳴る(self):
        assert "箱どうしが重なる" in check_shapes(_svg(_box(10, 10) + _box(50, 20)))

    def test_離れた箱では鳴らない(self):
        assert check_shapes(_svg(_box(10, 10) + _box(200, 10))) == []

    def test_箱を突っ切る線で鳴る(self):
        """両端は箱の外にある。端点だけを見る検査はこれを見逃した。"""
        inner = _box(150, 80) + ('<path d="M20,100 L380,100" fill="none" '
                                 'stroke="#000" stroke-width="2"/>')
        assert "辺が箱を突っ切る" in check_shapes(_svg(inner))

    def test_箱をよけた線では鳴らない(self):
        inner = _box(150, 80) + ('<path d="M20,20 L380,20" fill="none" '
                                 'stroke="#000" stroke-width="2"/>')
        assert check_shapes(_svg(inner)) == []

    def test_箱につながる線は突っ切りとしない(self):
        """始点が触れている箱は、その辺の相手なので除く。"""
        inner = _box(150, 80) + ('<path d="M170,100 L380,100" fill="none" '
                                 'stroke="#000" stroke-width="2"/>')
        assert check_shapes(_svg(inner)) == []

    def test_中途半端に交差する囲みで鳴る(self):
        f1 = '<rect x="10" y="10" width="120" height="80" fill="none" stroke-dasharray="4"/>'
        f2 = '<rect x="80" y="40" width="120" height="80" fill="none" stroke-dasharray="4"/>'
        assert "囲みが中途半端に交差する" in check_shapes(_svg(f1 + f2))

    def test_入れ子の囲みでは鳴らない(self):
        f1 = '<rect x="10" y="10" width="200" height="150" fill="none" stroke-dasharray="4"/>'
        f2 = '<rect x="30" y="30" width="100" height="80" fill="none" stroke-dasharray="4"/>'
        assert check_shapes(_svg(f1 + f2)) == []

    def test_囲みの線が中身に食い込むと鳴る(self):
        """余白が0だと、囲みの縁が中の箱の縁に乗る。囲めていない。"""
        frame = ('<rect x="20" y="20" width="80" height="40" fill="none" '
                 'stroke-dasharray="4" stroke-width="2"/>')
        assert "囲みの線が中身に重なる" in check_shapes(_svg(frame + _box(20, 20)))

    def test_余白のある囲みでは鳴らない(self):
        frame = ('<rect x="8" y="8" width="104" height="64" fill="none" '
                 'stroke-dasharray="4" stroke-width="2"/>')
        assert check_shapes(_svg(frame + _box(20, 20))) == []


class TestAttachmentChecks:
    """辺の終端がインクに着いているか ── 3度目の静かな崩れを捕まえる番人。

    22通りの置き方すべてで「崩れ0件」と出ていたが、実際には波・三日月・
    円グラフ・棒グラフで辺が何も無いところを指していた。0件という表示は、
    見ていない項目については何も言っていない。
    """

    def _fig(self, node_svg, edge_d, sw=1.2):
        """出どころの節点と、着き先の節点と、その間の辺。

        辺は必ず何かと何かを結ぶので、片端だけを節点に着けた図を作ると、
        もう片端が（正しく）鳴って、試したい側が見えなくなる。
        """
        return _svg(f'<g class="wf-node" transform="translate(10,10)">'
                    f'<rect x="0" y="0" width="20" height="20" fill="#eee"/></g>'
                    f'<path d="{edge_d}" fill="none" stroke="#000" stroke-width="{sw}"/>'
                    f'<g class="wf-node" transform="translate(100,100)">{node_svg}</g>')

    def test_インクへ着いていれば鳴らない(self):
        node = '<rect x="0" y="0" width="60" height="30" fill="#eee"/>'
        assert check_attachment(self._fig(node, "M30,20 L100,100")) == []

    def test_空白を指していれば鳴る(self):
        """節点の外接矩形の中だが、インクは左半分にしか無い。"""
        node = '<rect x="0" y="0" width="20" height="30" fill="#eee"/>'
        faults = check_attachment(self._fig(node, "M30,20 L160,100"))
        assert any("着いていない" in f for f in faults)

    def test_太い線の内側は着いているとみなす(self):
        """線の中心までは遠いが、線幅の半分ぶんで届いている。"""
        node = '<path d="M0,0 L60,0" fill="none" stroke="#000" stroke-width="24"/>'
        assert check_attachment(self._fig(node, "M30,20 L130,88")) == []

    def test_標本の粗さで誤って鳴らない(self):
        """線分どうしで測らず点で測ると、標本の隙間に落ちた終端で誤検知する。"""
        node = '<path d="M0,0 L200,0" fill="none" stroke="#000" stroke-width="1"/>'
        assert check_attachment(self._fig(node, "M30,20 L143.7,100")) == []

    def test_節点の印が無ければ何も言わない(self):
        """節点が1つも無い図（装飾だけ）に対して、偽の指摘を出さない。"""
        assert check_attachment(_svg('<path d="M0,0 L10,10" fill="none" stroke="#000"/>')) == []


class TestComponentContract:
    """部品の契約そのものを試す ── 申告した大きさの中にインクが収まっているか。

    この性質は型（ComponentResult）に書いてあったが、誰も測っていなかった。
    実測すると、名前の字面が申告の外（y=-3.6）へ出ている部品があり、
    帯を描いて初めて見えた。**検査されない性質は、書いてあっても性質ではない。**

    置いた後に描く部品（辺・囲み・囲みの札）は器を申告しないので対象外。
    """

    def _ink_box(self, r):
        from svg_engine.geometry import sample_ink
        pts = [q for q, _ in sample_ink(r.svg, max(min(r.width, r.height), 1) / 32)]
        if not pts:
            return None
        return (min(x for x, _ in pts), min(y for _, y in pts),
                max(x for x, _ in pts), max(y for _, y in pts))

    def test_台帳の全部品に入力が用意されている(self, sample_props):
        """入力が無い部品は契約の試験を素通りする。素通りを試験で落とす。"""
        from svg_engine.registry import OwnOrigin, known_kinds
        missing = [k for k in known_kinds() if k not in sample_props]
        assert missing == [], f"conftest の sample_props に足りない: {missing}"

    def test_インクが申告した大きさの中に収まる(self, sample_props, style):
        from svg_engine.registry import OwnOrigin, render_component
        out = []
        for kind, props in sample_props.items():
            r = render_component(kind, props, style)
            if not isinstance(r, OwnOrigin):
                continue
            box = self._ink_box(r)
            if box is None:            # 文字だけの部品はこの測り方では見えない
                continue
            x0, y0, x1, y1 = box
            # 線の太さの半分ぶんは、縁に乗るのが正常
            m = style.num("size.stroke-width")
            if x0 < -m or y0 < -m or x1 > r.width + m or y1 > r.height + m:
                out.append(f"{kind}: 申告 {r.width:.0f}x{r.height:.0f} / "
                           f"インク {x0:.1f},{y0:.1f}〜{x1:.1f},{y1:.1f}")
        assert out == [], "申告した大きさの外へインクが出ている: " + " / ".join(out)

    def test_わざと外へ出せば鳴る(self, style):
        """この検査自身が効いていることを確かめる。"""
        from svg_engine.registry import OwnOrigin
        r = OwnOrigin(svg='<rect x="-9" y="0" width="20" height="10"/>',
                            width=20, height=10)
        x0, y0, x1, y1 = self._ink_box(r)
        assert x0 < -style.num("size.stroke-width")
