"""記録の契約（concrete contract）の補助の関数。標準ライブラリだけで書き、テストの間で共有する"""
import json,os
def trace(condition,hash,level):
  path=os.environ.get('CONCRETE_TRACE')
  if not path: return
  test=os.environ.get('PYTEST_CURRENT_TEST','').split(' ')[0]
  with open(path,'a',encoding='utf-8') as f:
    f.write(json.dumps({"condition":condition,"hash":hash,"level":level,"test":test},ensure_ascii=False)+'\n')
