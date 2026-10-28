"""既存の書籍紹介の部品を使い、全体の土台を上に置く。"""
from intro_visuals import *
from visuals import cover,BOOKS

def slide_09():
    isbn,title,author,pub=BOOKS[0]
    a=rect(0,208,540,190,PAPER,12,LINE)+rect(572,208,540,190,PAPER,12,LINE)
    a+=rect(0,0,W,192,PAPER,12,ACCENT)
    a+=cover(20,16,112,160,isbn)
    a+=text(156,30,'教材全体の土台',14,ACCENT,700)
    a+=text(156,59,''.join(title),21,INK,700)
    a+=text(156,84,f'{author}（{pub}）',13,DIM)
    a+=text(156,116,'何を実現すべきかを考え、実行につなげる',18,INK,700)
    a+=text(156,147,'具体と抽象を往復し、目的と要件の大きさを見極める',16,INK)
    a+=text(156,174,'目的を、実行できる作業や条件に具体化する',14,DIM)
    for k,(book,tag,lines) in enumerate([(BOOKS[1],'教材1　目的と要件の明確化',['誰が、何を達成したいのか','目的レベルと範囲を決め、要件にする']),(BOOKS[2],'教材2　業務のモデル化と設計',['業務の言葉と判断をモデルに表す','言葉が通じる範囲と、一緒に守る条件を考える'])]):
        x=k*572;isbn,title,author,pub=book
        a+=cover(x+20,222,112,160,isbn)
        a+=text(x+156,241,''.join(title),16,INK,700)
        a+=text(x+156,265,f'{author}（{pub}）',13,DIM)
        a+=rect(x+156,282,372,108,PAPER,10,LINE)
        a+=text(x+176,308,tag,14,DIM,700)
        a+=text(x+176,335,lines,15,INK,gap=25)
    return svg('上に具体と抽象を教材全体の土台として置き、下に教材1の目的と要件の明確化、教材2の業務のモデル化と設計の参考書を並べ、選んだ理由を示す',a,402)
