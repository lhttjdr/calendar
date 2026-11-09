"""由 zengshan_buyi_extract.py 的 JSON 生成 liuyao.rs 卦例测试常量。
用法（仓库根）：python3 scripts/folk/zengshan_buyi_gen.py recs.json data.rs
"""
import sys
import json,re
R=json.load(open(sys.argv[1]))
Z='子丑寅卯辰巳午未申酉戌亥'; G='甲乙丙丁戊己庚辛壬癸'
NUM={'正':1,'一':1,'二':2,'三':3,'四':4,'五':5,'六':6,'七':7,'八':8,'九':9,'十':10,'十一':11,'冬':11,'十二':12,'臘':12}
def gz(g,z): 
    gi,zi=G.index(g),Z.index(z)
    for n in range(60):
        if n%10==gi and n%12==zi: return n
    return None
def norm(s): return s.replace('夘','卯').replace('已','己') if s else s
NAMES=['乾','坤','屯','蒙','需','訟','師','比','小畜','履','泰','否','同人','大有','謙','豫','隨','蠱','臨','觀','噬嗑','賁','剝','復','无妄','大畜','頤','大過','坎','離','咸','恒','遯','大壯','晉','明夷','家人','睽','蹇','解','損','益','夬','姤','萃','升','困','井','革','鼎','震','艮','漸','歸妹','豐','旅','巽','兌','渙','節','中孚','小過','既濟','未濟']
PURE=['乾','坤','坎','離','震','艮','巽','兌']
NORM=[('恆','恒'),('旣','既'),('卽濟','既濟'),('無妄','无妄'),('遁','遯'),('覆','履'),('騫','蹇'),('兑','兌'),('暌','睽')]
def res(t):
    for a,b in NORM: t=t.replace(a,b)
    best=None
    for n in NAMES:
        X='天地水火山澤風雷'
        if t==n or (len(t)==len(n)+2 and t.endswith(n) and t[0] in X and t[1] in X) or (n in PURE and len(t)==3 and t.startswith(n+'爲')):
            if best is None or len(n)>len(best): best=n
    return best
SAFE={'逐','畜'}  # 录文疑讹：保留以报异
def pick(h):
    h=re.sub(r'[「」：﹕“”"]','',h)
    for m in reversed(list(re.finditer(r'得',h))):
        rest=h[m.end():]
        mm=re.match(r'([^，。﹐、\s之化變变卦]{1,6})卦?(?:(?:變出|之|化|變|变)([^，。﹐、\s卦]{1,5}))?',rest)
        if not mm: continue
        b,z=mm.group(1),mm.group(2) or ''
        if not z:
            for k in range(1,len(b)):
                if res(b[:k]) and res(b[k:]): return b[:k],b[k:]
        if (res(b) or b in SAFE) and (not z or res(z) or z in SAFE):
            if False:
                for k in range(1,len(b)):
                    if res(b[:k]) and res(b[k:]) and b[:k]==b[:k] and len(b[k:])<=3 and b[:k] in NAMES+[x+'爲'+y for x in PURE for y in '天地水火山澤風雷']:
                        return b[:k],b[k:]
            return b,z
    return '',''
out=[]
for r in R:
    yue=ri=None; used=''
    for h in (r['hdr'],r['hdr2']):
        h2=norm(h)
        m=re.search(r'([子丑寅卯辰巳午未申酉戌亥])月',h2) or re.search(r'(十二|十一|正|二|三|四|五|六|七|八|九|十|冬|臘)月',h2)
        d=re.search(r'([甲乙丙丁戊己庚辛壬癸])([子丑寅卯辰巳午未申酉戌亥])(?:日|占)',h2)
        if m or d:
            if m:
                t=m.group(1); yue = Z.index(t) if t in Z else (NUM[t]+1)%12
            if d:
                ri=gz(d.group(1),d.group(2))
                if ri is None: ri=None
            used=h; break
    hdr=r['hdr']
    ben,zhi=pick(hdr)
    q=lambda s: '"'+s.replace('"','')+'"'
    o=lambda v: f'Some({v})' if v is not None else 'None'
    src=r['src'].replace('增刪卜易','').replace('卷之','卷')
    out.append(f'        ({q(src)}, {q(hdr[:40])}, {o(yue)}, {o(ri)}, {q(ben)}, {q(zhi)}, [{", ".join(q(x) for x in r["blk"])}]),')
open(sys.argv[2],'w').write('\n'.join(out)+'\n')
print(len(out)); print(out[0]); print(out[100])
import collections
print(sum('Some' in x.split(',')[2] for x in out))
