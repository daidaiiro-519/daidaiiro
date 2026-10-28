"""本の識別子（lesson-0N-<key> の 0N-<key>）と、配るものを置くフォルダの対応。

配るものは本ごとに1フォルダで、中の名前はどの本でも同じ（slides.html ・ script.md ・ video.mp4 …）。
"""
from pathlib import Path

DIST = Path(__file__).resolve().parents[2] / 'dist'
PLACE = {'00-start': 'orientation', '01-lead': 'intro',
         '02-intro': 'lesson-1/01-cause', '03-word': 'lesson-1/02-word', '04-scope': 'lesson-1/03-scope',
         '05-rule': 'lesson-1/04-rule', '06-summary': 'lesson-1/05-summary'}


def place(stem):
    """lesson-00-start でも 00-start でも、その本のフォルダを返す。"""
    folder = DIST / PLACE[stem.removeprefix('lesson-')]
    folder.mkdir(parents=True, exist_ok=True)
    return folder
