# 宣言をリポジトリの spec/ から読む（道具は宣言のファイルだけを読み、ソースコードは読まない）
import json,os
ROOT=os.environ.get('CONCRETE_ROOT',os.getcwd())
_o=json.load(open(os.path.join(ROOT,'spec/order.json')))
_extra=sorted(f[:-5] for f in os.listdir(os.path.join(ROOT,'spec/decls')) if f[:-5] not in _o)
D={k:json.load(open(os.path.join(ROOT,'spec/decls',k+'.json'))) for k in _o+_extra}
IMPL=json.load(open(os.path.join(ROOT,'impl/definition.json')))['business_logic']
RETIRED=json.load(open(os.path.join(ROOT,'spec/retired.json')))
