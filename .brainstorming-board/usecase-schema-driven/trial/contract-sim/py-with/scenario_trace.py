"""テストが検査するシナリオID を、環境変数 SCENARIO_TRACE のファイルへ追記する補助。"""
import os


def record_scenario(scenario_id):
    path = os.environ.get("SCENARIO_TRACE")
    if not path:
        return
    with open(path, "a", encoding="utf-8") as f:
        f.write(scenario_id + "\n")
