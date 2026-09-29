"""論点2の描画の規則を試す試作である（正本ではない）。スキーマの title を見出しに、description を説明にし、
x-view（card ・ tag ・ table ・ steps ・ list ・ figure）で見せ方を決める。指定が無ければ JSON の形から決める。
見た目の値は、トークン（:root の変数）だけから引く。描画のコードは、回答の形を参照しない。"""
import html, json, os, sys

TOKENS = {"ink": "#1f2328", "muted": "#59636e", "line": "#d1d9e0", "paper": "#ffffff", "band": "#f3f5f7",
          "accent": "#0f6e5c", "accent-soft": "#e3f1ed", "warn": "#9a3412", "warn-soft": "#fdeee6",
          "radius": "10px", "gap": "16px"}

def esc(s): return html.escape(str(s))

def label_of(schema, value):
    for o in schema.get("oneOf", []):
        if o.get("const") == value: return o.get("title", value)
    return value

def tag(schema, value):
    v = label_of(schema, value)
    cls = "tag neg" if value in (False, "reject") else "tag"
    return f'<span class="{cls}">{esc(v)}</span>'

def cell(schema, value):
    if schema.get("x-view") == "tag" or "oneOf" in schema: return tag(schema, value)
    return esc(value)

def block(key, schema, value, base):
    view = schema.get("x-view")
    title, desc = schema.get("title", key), schema.get("description")
    head = f"<h2>{esc(title)}</h2>" + (f'<p class="desc">{esc(desc)}</p>' if desc else "")
    if view == "card":
        props = schema.get("properties", {})
        tags = "".join(tag(p, value[k]) for k, p in props.items() if k in value and (p.get("x-view") == "tag"))
        lines = "".join(f'<p class="{"lead" if i == 0 else "sub"}">{esc(value[k])}</p>'
                        for i, (k, p) in enumerate((k, p) for k, p in props.items() if k in value and p.get("x-view") != "tag"))
        return f'<section class="card"><div class="cardhead"><h2>{esc(title)}</h2>{tags}</div>{lines}</section>'
    if view == "figure":
        svg = open(os.path.join(base, value["svg"])).read() if value.get("svg") else ""
        return f'<section class="block">{head}<figure>{svg}<figcaption>{esc(value.get("caption",""))}</figcaption></figure></section>'
    if view == "table":
        cols = list(schema["items"]["properties"].items())
        th = "".join(f"<th>{esc(p.get('title', k))}</th>" for k, p in cols)
        trs = "".join("<tr>" + "".join(f"<td>{cell(p, v.get(k, ''))}</td>" for k, p in cols) + "</tr>" for v in value)
        return f'<section class="block">{head}<div class="scroll"><table><thead><tr>{th}</tr></thead><tbody>{trs}</tbody></table></div></section>'
    if view == "steps":
        props = list(schema["items"]["properties"].items())
        lis = "".join("<li>" + f'<p class="lead">{esc(v[props[0][0]])}</p>' + "".join(f'<p class="sub">{esc(v[k])}</p>' for k, _ in props[1:] if k in v) + "</li>" for v in value)
        return f'<section class="block">{head}<ol class="steps">{lis}</ol></section>'
    if view == "list" or isinstance(value, list):
        return f'<section class="block">{head}<ul>' + "".join(f"<li>{esc(v)}</li>" for v in value) + "</ul></section>"
    return f'<section class="block">{head}<p>{esc(value)}</p></section>'

schema_path, data_path = sys.argv[1], sys.argv[2]
schema = json.load(open(schema_path)); data = json.load(open(data_path)); data.pop("$schema", None)
base = os.path.dirname(os.path.abspath(data_path))
props = schema["properties"]
top = f'<header><p class="eyebrow">{esc(schema["title"])} ・ {tag(props["kind"], data["kind"])}</p><h1>{esc(data["question"])}</h1></header>'
body = "".join(block(k, props[k], data[k], base) for k in props if k in data and k not in ("kind", "question"))
root = ":root{" + "".join(f"--{k}:{v};" for k, v in TOKENS.items()) + "}"
css = root + """
*{box-sizing:border-box} body{margin:0;background:var(--band);color:var(--ink);font:15px/1.8 'Noto Sans JP',sans-serif}
main{max-width:880px;margin:0 auto;padding:24px var(--gap)}
header{margin:0 0 var(--gap)} .eyebrow{color:var(--muted);font-size:12px;margin:0} h1{font-size:21px;line-height:1.5;margin:4px 0 0}
h2{font-size:14px;color:var(--accent);margin:0 0 2px;letter-spacing:.02em}
.desc{color:var(--muted);font-size:12px;margin:0 0 8px}
.block,.card{background:var(--paper);border:1px solid var(--line);border-radius:var(--radius);padding:14px var(--gap);margin:0 0 12px}
.card{border:2px solid var(--accent)} .cardhead{display:flex;gap:10px;align-items:center;flex-wrap:wrap}
.lead{font-size:18px;font-weight:700;margin:6px 0 2px} .sub{color:var(--ink);margin:0}
.tag{display:inline-block;background:var(--accent-soft);color:var(--accent);border-radius:999px;padding:1px 10px;font-size:12px;font-weight:600;white-space:nowrap}
.tag.neg{background:var(--warn-soft);color:var(--warn)}
.scroll{overflow-x:auto} table{border-collapse:collapse;width:100%;font-size:14px}
th,td{border-bottom:1px solid var(--line);padding:8px 10px;text-align:left;vertical-align:top} th{color:var(--muted);font-weight:600;font-size:12px}
ol.steps{list-style:none;counter-reset:s;margin:0;padding:0} ol.steps li{counter-increment:s;position:relative;padding:4px 0 10px 40px}
ol.steps li::before{content:counter(s);position:absolute;left:0;top:6px;width:26px;height:26px;border-radius:50%;background:var(--accent);color:#fff;font-size:13px;display:flex;align-items:center;justify-content:center}
ol.steps .lead{font-size:15px;margin:0}
ul{margin:0;padding-left:1.2em} p{margin:0;overflow-wrap:anywhere}
figure{margin:0} figure svg{max-width:100%;height:auto} figcaption{color:var(--muted);font-size:12px}
@media (max-width:480px){main{padding:16px 12px} h1{font-size:18px} .lead{font-size:16px}}
"""
print(f"<!doctype html><html lang=ja><head><meta charset=utf-8><meta name=viewport content='width=device-width,initial-scale=1'><title>{esc(schema['title'])}</title><style>{css}</style></head><body><main>{top}{body}</main></body></html>")
