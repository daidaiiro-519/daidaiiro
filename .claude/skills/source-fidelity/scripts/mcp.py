# SPDX-License-Identifier: MIT
"""source-fidelity の MCP の面。**同じ宣言から組む** ── 能力は1行も複製しない。

`mcp` の実装が無い環境では、この面は立たない ── **CLI は動く**。
呼び出し方が欠けても、能力は欠けない。
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from tools import TOOLS  # noqa: E402


def build():
    """宣言から、MCP の道具を組む。"""
    from mcp.server.fastmcp import FastMCP   # 実装が無ければ、ここで止まる

    server = FastMCP("source-fidelity")
    for t in TOOLS:
        def make(tool):
            def call(**kw):
                return tool.run(**kw)
            call.__name__ = tool.name
            call.__doc__ = tool.summary
            return call
        server.add_tool(make(t), name=t.name, description=t.summary)
    return server


if __name__ == "__main__":
    try:
        build().run()
    except ModuleNotFoundError as e:
        # **呼び出し方が欠けても、能力は欠けない。**CLI は動く。
        print(f"MCP の実装が無いので、この面は立たない（{e}）── "
              "python3 scripts/cli.py help で、同じ道具を呼べる", file=sys.stderr)
        raise SystemExit(2)
