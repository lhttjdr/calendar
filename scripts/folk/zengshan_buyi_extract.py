"""《增删卜易》卦例转录：自 vendor/folk/wikisource/增刪卜易/（卷一各子页、卷二至四全文页）取凡画六行卦图之例，
逐爻解析六神、六亲、纳支、五行、动变、世应、伏神、之卦，写 JSON。

用法（仓库根）：python3 scripts/folk/zengshan_buyi_extract.py recs.json
再用 scripts/folk/zengshan_buyi_gen.py recs.json data.rs 生成 rust/core/src/folk/liuyao.rs 测试常量（298 例）。
逐例检验见 doc/folk/liuyao.md。
"""
import re,sys,json
D='vendor/folk/wikisource/增刪卜易/'
order=['1','2','3','4','5','6','7','8','9','10','11','12','13','14','15','15又','16','17','18','19','20','21','22','23','24','25','26','26又1','26又2','26又3','26又4']
src=[]
for o in order:
    t=open(D+o+'.wikitext').read()
    sec=re.search(r'section = ([^\n|}]*)',t).group(1).strip()
    src.append(('卷一·'+sec,t))
full=open(D+'全文.wikitext').read()
for m in re.finditer(r'== (增刪卜易卷之[二三四]) ==\n(.*?)(?=\n== 增刪卜易卷之|\Z)',full,re.S):
    src.append((m.group(1),m.group(2)))
Q='父母|兄弟|妻財|官鬼|子孫'
Z='子丑寅卯夘辰巳午未申酉戌亥'
SH='青龍|靑龍|朱雀|勾陳|螣蛇|滕蛇|媵蛇|白虎|玄武|元武'
YL=re.compile(rf'^(?:({SH})　)?({Q})([{Z}])([金木水火土])([⚊⚋○ㄨ])(世|應)?(?:伏({Q})([{Z}])([金木水火土]))?(?:　+(?:({Q})([{Z}])([金木水火土])([⚊⚋]))?)?\s*$')
recs=[]
for name,t in src:
    lines=[l.replace('<br>','').strip() for l in t.split('\n')]
    # chapter headings inside 全文
    chap=name
    i=0
    while i<len(lines):
        l=lines[i]
        if (re.match(r'^[^\s]{2,14}章第',l) or re.match(r'^(《星煞章》|增刪《黃金策千金賦》)第',l)) and len(l)<24: chap=name+'·'+re.sub(r'﹝.*','',l).replace('《','').replace('》','')
        if YL.match(l):
            j=i
            while j<len(lines) and YL.match(lines[j]): j+=1
            blk=lines[i:j]
            # header: nearest non-empty preceding lines
            k=i-1; hdr=[]
            while k>=0 and len(hdr)<2:
                if lines[k] and not lines[k].startswith('<') and not lines[k].startswith('{{'): hdr.append(lines[k])
                k-=1
            h=hdr[0] if hdr else ''
            recs.append(dict(src=chap,line=i,hdr=h,hdr2=hdr[1] if len(hdr)>1 else '',blk=blk))
            i=j; continue
        i+=1
print(len(recs),file=sys.stderr)
from collections import Counter
print(Counter(len(r['blk']) for r in recs),file=sys.stderr)
json.dump(recs,open(sys.argv[1],'w'),ensure_ascii=False,indent=0)
