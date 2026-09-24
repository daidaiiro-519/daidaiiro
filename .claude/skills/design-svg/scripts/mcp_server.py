# SPDX-License-Identifier: MIT
"""design-svg の MCP の面。**同じ宣言から組む** ── 能力は1行も複製しない。

`mcp` の実装が無い環境では、この面は立たない ── **CLI は動く**。
呼び出し方が欠けても、能力は欠けない。
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from tools import TOOLS  # noqa: E402


def build():
    """宣言から、MCP の道具を組む。"""
    try:                                     # mcp 2.x
        from mcp.server.mcpserver import MCPServer as Server
    except ModuleNotFoundError:              # mcp 1.x
        from mcp.server.fastmcp import FastMCP as Server

    server = Server("design-svg")
    for t in TOOLS:
        server.add_tool(_callable(t), name=t.name, description=t.summary)
    return server


def _callable(tool):
    """宣言から、**引数を1つずつ持つ関数**を組む。

    `**kw` で受けると、公開される入力の形が `kw` 1個になり、**呼ぶ側が
    どの引数を渡せばよいかを認知できない**（実測 2026-09-24、`check` が
    `['kw']` として公開されていた）。
    """
    params, passes = [], []
    for a in tool.args:
        params.append(f"{a.key}: str" if a.required else f"{a.key}: str = ''")
        passes.append(f"{a.key}={a.key}")
    src = f"def call({', '.join(params)}):\n    return _run({', '.join(passes)})\n"
    ns: dict = {"_run": tool.run}
    exec(src, ns)                            # noqa: S102 ── 宣言から組む
    fn = ns["call"]
    fn.__name__ = tool.name
    fn.__doc__ = tool.summary
    return fn


if __name__ == "__main__":
    try:
        build().run()
    except ModuleNotFoundError as e:
        # **呼び出し方が欠けても、能力は欠けない。**CLI は動く。
        print(f"MCP の実装が無いので、このサーバーは立たない（{e}）── "
              "python3 scripts/cli.py help で、同じ道具を呼べる", file=sys.stderr)
        raise SystemExit(2)
