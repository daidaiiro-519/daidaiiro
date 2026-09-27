import json,subprocess,os,sys,glob,re
S="/tmp/claude-1000/-home-daidaiiro-workspace-daidaiiro/4a179eab-201c-42f3-b3d6-fa325e16ded0/scratchpad/"
R=S+"real/"; NEW=S+"nms/rs/target/release/no-more-spaghetti"
OLD="/home/daidaiiro/workspace/daidaiiro/.claude/skills/no-more-spaghetti/rs/target/release/no-more-spaghetti"
def dirs(base,pat="*"): return sorted(d for d in glob.glob(base+pat) if os.path.isdir(d))
def cases():
    rg=R+"ripgrep/"
    yield ("ripgrep","rust",rg,[f"{os.path.basename(d)}=crates.{os.path.basename(d)}" for d in dirs(rg+"crates/")],
           [re.search(r'^name\s*=\s*"([^"]+)"',open(c).read(),re.M).group(1).replace('-','_') for c in glob.glob(rg+"crates/*/Cargo.toml")])
    ot=R+"opentelemetry-go/"
    top=[os.path.basename(d) for d in dirs(ot) if glob.glob(d+"/**/*.go",recursive=True)]
    yield ("opentelemetry-go","go",ot,[f"{x}={x}" for x in top if x not in("internal",)]+["internal=internal"],["go.opentelemetry.io/otel"])
    tr=R+"trpc/"
    yield ("trpc","typescript",tr,[f"{os.path.basename(d)}=packages/{os.path.basename(d)}" for d in dirs(tr+"packages/")],["@trpc/"])
    bl=R+"black/"
    pk=[os.path.basename(d) for d in dirs(bl+"src/")]
    yield ("black","python",bl,[f"{x}=src.{x}" for x in pk],pk)
    ru=R+"rubocop/"
    yield ("rubocop","ruby",ru,["rubocop=lib/rubocop"],["rubocop"])
    fm=R+"fmt/"
    yield ("fmt","cpp",fm,["fmt=include/fmt","src=src"],["fmt/"])
def inside(points,to):
    for p in points:
        q=p
        if '/' in p and '.' in p.rsplit('/',1)[-1]: q=p.rsplit('.',1)[0]
        if q==to: return True
        for sep in ("/",".","\\","::"):
            if q.startswith(to+sep) or to.startswith(q+sep): return True
    return False
for name,lang,root,layers,internal in cases():
    row=[name,lang]
    for B in (OLD,NEW):
        r=subprocess.run([B,"inward",lang,root,*layers,"--json"],capture_output=True,text=True)
        try: d=json.loads(r.stdout)
        except Exception: print(name,"JSON でない",r.stdout[:200],r.stderr[:200]); continue
        data=d.get("data",{}); f=d.get("findings",[])
        if B==NEW:
            edges=data.get("edge_list",[]); pts=data.get("points",[])
            looks=[e for e in edges if any(e[1].startswith(i) or e[1]==i.rstrip('/') for i in internal) or any(e[1].split('.')[0]==i for i in internal)]
            unres=[e for e in looks if not inside(pts,e[1])]
            cfg=[x for x in f if "設定を読めない" in x]
            unl=[x for x in f if "どの層にも属さない" in x]
            row+= [f"辺{len(edges)}", f"内部と見える参照{len(looks)}", f"うち解決できず{len(unres)}", f"設定を読めない{len(cfg)}", f"層に属さない{len(unl)}", f"層どうしの辺{data.get('layered_edges')}"]
            samples=(unres[:3],cfg[:2],unl[:2])
        else:
            row+=[f"いま：辺{data.get('edges')}"]
    print(" | ".join(map(str,row)))
    for s in samples:
        for x in s: print("      ",str(x)[:180])
