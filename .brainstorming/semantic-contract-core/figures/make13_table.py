def e(s): return s.replace('&','&amp;').replace('<','&lt;')

def table(name, label, heads, widths, rows, emph=(1,), row_h=40, note=None):
    X=[16]
    for w in widths[:-1]: X.append(X[-1]+w+8)
    W=X[-1]+widths[-1]+16
    H=32+len(rows)*(row_h+6)+(28 if note else 8)
    o=[f'<svg viewBox="0 0 {W} {H}" style="min-width:{min(W,900)}px" role="img" aria-label="{e(label)}">']
    for x,h in zip(X,heads): o.append(f'<text class="note" x="{x+4}" y="22">{e(h)}</text>')
    for i,r in enumerate(rows):
        y=32+i*(row_h+6)
        for j,(x,w) in enumerate(zip(X,widths)):
            cell=r[j]; lines=cell if isinstance(cell,list) else [cell]
            cls="box focus" if j in emph else ("boundary" if j==len(widths)-1 and heads[-1].startswith("構想") else "box")
            tc="title" if j in emph else ("note" if cls=="boundary" else "label")
            o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{w}" height="{row_h}"/>')
            if len(lines)==1: o.append(f'<text class="{tc}" x="{x+10}" y="{y+row_h//2+5}">{e(lines[0])}</text>')
            else:
                for k,l in enumerate(lines): o.append(f'<text class="{tc}" x="{x+10}" y="{y+row_h//2-4+16*k}">{e(l)}</text>')
    if note: o.append(f'<text class="note" x="20" y="{H-10}">{e(note)}</text>')
    o.append('</svg>'); open(name+'.src.svg','w').write('\n'.join(o))
