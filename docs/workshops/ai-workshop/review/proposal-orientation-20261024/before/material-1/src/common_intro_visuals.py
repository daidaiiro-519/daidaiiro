"""確認済みの三教材共通の図を読み込む。"""
from pathlib import Path
HERE = Path(__file__).resolve().parent / 'figures' / 'common-intro'

def figure(number):
    return (HERE / f'slide-{number:02}.svg').read_text()
