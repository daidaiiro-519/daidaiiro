# SPDX-License-Identifier: MIT
"""doc-writing-skills の MCP サーバー。**同じ宣言から組む** ── 能力は1行も複製しない。

`mcp` の実装が無い環境では、このサーバーは立たない ── **CLI は動く**。
呼び出し方が欠けても、能力は欠けない。

**このファイルを `mcp.py` と名付けない。** 実行すると自分の在る場所が探索の先頭に
入るので、`import mcp` がこのファイル自身を指し、**実装が在っても「無い」と報告する**
（実測 2026-09-24）。

**道具は標準出力へ1バイトも書かない。** 原典が禁じている ── `The server MUST NOT
write anything to its stdout that is not a valid MCP message.`
（modelcontextprotocol.io/specification/2025-06-18/basic/transports:33、2026-09-24 取得）。
ログは標準エラーへ書く（同 31行が許す）。
"""
import pathlib
import sys
import typing

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from tools import TOOLS  # noqa: E402


def build():
    """宣言から、MCP の道具を組む。"""
    try:                                     # mcp 2.x ── 原典の README がこの経路を示す
        from mcp.server import MCPServer as Server
    except ImportError:                      # mcp 1.x
        from mcp.server.fastmcp import FastMCP as Server

    server = Server("doc-writing-skills")
    for t in TOOLS:
        # **JSON を構造としても乗せる。** 原典が両方を求めている ──
        # `a tool that returns structured content SHOULD also return the
        # serialized JSON in a TextContent block`
        # （specification/2025-06-18/server/tools:298、2026-09-24 取得）。
        # 素の `dict` は自動判定では非構造になるので、明示して渡す。
        server.add_tool(_callable(t), name=t.name, description=t.summary,
                        structured_output=True)
    return server


def _callable(tool):
    """宣言から、**引数を1つずつ持つ関数**を組む。

    `**kw` で受けると、公開される入力の形が `kw` 1個になり、**呼ぶ側が
    どの引数を渡せばよいかを認知できない**（実測 2026-09-24、`check` が
    `['kw']` として公開されていた）。

    **型は緩く受ける** ── 呼ぶ側は JSON の値を渡すので、数を文字列で包むことを
    強制しない（実測 2026-09-24、`timeout` に 30 を渡すと検証で落ちていた）。

    **`ok` が偽なら、例外にする** ── 原典は2つを分けている。
    `Tool Execution Errors: Reported in tool results with isError: true`
    （specification/2025-06-18/server/tools:389、2026-09-24 取得）。
    **検出（findings）は誤りではない**ので、そちらは通常の結果のまま返す。
    """
    params, passes = [], []
    for a in tool.args:
        t = "list[str] | str" if a.many else "str | int | float | bool"
        params.append(f"{a.key}: {t}" if a.required else f"{a.key}: {t} = ''")
        passes.append(f"{a.key}={a.key}")
    src = (f"def call({', '.join(params)}) -> dict[str, _Any]:\n"
           f"    return _guard(_run({', '.join(passes)}))\n")
    ns: dict = {"_run": tool.run, "_guard": _guard, "_Any": typing.Any}
    exec(src, ns)                            # noqa: S102 ── 宣言から組む
    fn = ns["call"]
    fn.__name__ = tool.name
    fn.__doc__ = tool.summary
    return fn


def _guard(res: dict) -> dict:
    """`ok` が偽なら例外にする ── MCP は、それを `isError: true` として返す。

    **`ToolError` で投げる。** SDK の原典が述べている ── 予期した失敗はその型で
    投げると `is_error=True` と**こちらの文言**が呼ぶ側へ届き、それ以外の例外は
    `Error executing tool <name>` しか届かない。

        the call returns `is_error=True` with your message in `content` for the
        model to read … Any other exception bar `MCPError` (a protocol error) is
        treated as a crash: the model sees only `Error executing tool <name>`

    出典 ── python-sdk v2.2.0 の `src/mcp/server/mcpserver/exceptions.py:47`
    （https://raw.githubusercontent.com/modelcontextprotocol/python-sdk/v2.2.0/src/mcp/server/mcpserver/exceptions.py
    ・ 2026-09-24 取得 ・ 77行 ・ sha256 14564c0dedfe79db…）。
    **導入した版と原文が一致することを確認した。**
    """
    if res.get("ok", True):
        return res
    message = " ／ ".join(res.get("findings") or ["失敗した"])
    for path in ("mcp.server.mcpserver.exceptions", "mcp.server.fastmcp.exceptions"):
        try:
            module = __import__(path, fromlist=["ToolError"])
        except ImportError:
            continue
        raise module.ToolError(message)
    raise ValueError(message)


if __name__ == "__main__":
    try:
        build().run()
    except ModuleNotFoundError as e:
        # **呼び出し方が欠けても、能力は欠けない。**CLI は動く。
        print(f"MCP の実装が無いので、このサーバーは立たない（{e}）── "
              "python3 scripts/cli.py help で、同じ道具を呼べる", file=sys.stderr)
        raise SystemExit(2)
