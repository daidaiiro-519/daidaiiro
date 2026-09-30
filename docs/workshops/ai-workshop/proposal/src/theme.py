"""配色の正本は slide-deck の warm-paper テーマである。ここはその値を読むだけで、色を持たない。"""
import pathlib, re

_CSS = pathlib.Path(__file__).resolve().parents[5] / '.claude' / 'skills' / 'slide-deck' / 'references' / 'themes' / 'warm-paper.css'
T = dict(re.findall(r'--([a-z-]+)\s*:\s*([^;]+);', _CSS.read_text()))
