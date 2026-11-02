"""西汉日食宿度「传抄讹误」假设的统计检验：单字改动（度数删/加「十」、改个位数字、宿名换相邻宿）能否把偏差大于 5 度的记录
改到与三统历推算相差 3 度以内，以及同样大小的随机错误被这样「改对」的偶合概率。逐条概率按每条自身偏差、随机方向与位置模拟 400 次，
合计按泊松二项分布求 P(命中数 ≥ 实际)。改法与容差是预先选定的，结果未作多重比较校正。

输出 data/phenomena/checks/han_lodge_emendation.csv。用法（仓库根）：python3 scripts/calendar_data/qinhan_eval/han_lodge_emendation_test.py
"""
import sys,csv,random
sys.path.insert(0,'scripts/calendar_data/qinhan_eval'); sys.path.insert(0,'scripts/calendar_data')
import han_eclipse_sun_by_calendar as h
from fractions import Fraction as F
O=h.ORDER
def edits(l,d):
    """单字讹误候选：度数中删/加「十」、改一个数字、宿名改为相邻宿（度数不变）。"""
    d=int(d); out=set()
    tens,units=divmod(d,10)
    # 数字字形：如 19 = 十九，9 = 九，13 = 十三
    if tens>=1: out.add((l,units if tens==1 else units))      # 删「十」（十九→九；二十二→二二 不合法，只处理一十几）
    if tens==0: out.add((l,10+d))                               # 加「十」（九→十九）
    for u in range(10):                                         # 改个位数字
        if u!=units: out.add((l,tens*10+u))
    i=O.index(l)
    for k in (-1,1): out.add((O[(i+k)%28],d))                   # 相邻宿
    out.discard((l,d)); out={(a,b) for a,b in out if b>=0 and b<=h.WIDTHS[a]+1}
    return out
recs=[]
for r in csv.DictReader(x for x in open('data/phenomena/hanshu_solar_eclipses.csv',encoding='utf8') if not x.startswith('#')):
    p=h.parse_lodge(r['lodge']) if r['lodge'] else None; y=int(r['year'])
    if p and -104<=y<=0 or (p and y in(-187,-180)):
        recs.append((y,int(r['jdn']),p[0],int(p[1]),r['lodge']))
TOL=3
rows_out=[]
for y,j,l,d,t in []:
    e=edits(l,d); hits=[(a,b,round(h.residual('三统',j,a,F(b)),1)) for a,b in e if abs(h.residual('三统',j,a,F(b)))<=TOL]
    r0=h.residual('三统',j,l,F(d))
# 对照：随机「错误记录」——在真实日躔上随机偏 5—20 度后，单字改动命中 ±3 的概率
random.seed(1); n=0; k=0; tot=float(h.CALENDARS['三统'][1]); dou=h.CALENDARS['三统'][1]-365
for y,j,_,_,_ in recs*200:
    x=(float(h.sun_degree('三统',j))+random.choice([-1,1])*random.uniform(5,20))%tot
    acc=0
    for c in O:
        wd=float(h.WIDTHS[c]+(dou if c=='斗' else 0))
        if x<acc+wd: l,d=c,int(x-acc); break
        acc+=wd
    e=edits(l,d); n+=1
    if any(abs(h.residual('三统',j,a,F(b)))<=TOL for a,b in sorted(e)): k+=1
k_all,n_all=k,n
tot=float(h.CALENDARS['三统'][1]); dou=h.CALENDARS['三统'][1]-365
def locate(x):
    acc=0
    for c in O:
        wd=float(h.WIDTHS[c]+(dou if c=='斗' else 0))
        if x<acc+wd: return c,int(x-acc)
        acc+=wd
ps=[];obs=0
random.seed(2)
for y,j,l,d,t in recs:
    r0=h.residual('三统',j,l,F(d))
    if abs(r0)<=5: continue
    hit=any(abs(h.residual('三统',j,a,F(b)))<=TOL for a,b in edits(l,d)); obs+=hit
    x0=float(h.sun_degree('三统',j)); k=0;N=400
    for _ in range(N):
        x=(x0+random.choice([-1,1])*abs(r0)+random.uniform(-.5,.5))%tot
        a,b=locate(x)
        k+=any(abs(h.residual('三统',j,aa,F(bb)))<=TOL for aa,bb in sorted(edits(a,b)))
    ps.append(k/N); rows_out.append([y,t,f"{r0:+.1f}",'是' if hit else '否',' '.join(f'{a}{b}({v:+.1f})' for a,b,v in [(a,b,h.residual('三统',j,a,F(b))) for a,b in sorted(edits(l,d))] if abs(v)<=TOL),f"{k/N:.2f}"])
# 泊松二项：P(命中数 >= obs)
dist=[1.0]
for p in ps:
    nd=[0]*(len(dist)+1)
    for i,v in enumerate(dist): nd[i]+=v*(1-p); nd[i+1]+=v*p
    dist=nd
summary=f'实际可改对 {obs}/{len(ps)}，偶然期望 {sum(ps):.1f}，P(≥{obs}) = {sum(dist[obs:]):.3f}'
print(summary)
import pathlib
out=pathlib.Path('data/phenomena/checks/han_lodge_emendation.csv')
with open(out,'w',encoding='utf-8',newline='') as f:
    f.write('# 西汉日食宿度单字改动检验（scripts/calendar_data/qinhan_eval/han_lodge_emendation_test.py）：偏差 > 5 度者；差 = 三统历推算 − 记载\n')
    f.write(f'# {summary}；对照（偏 5—20 度随机错误）整体偶合率 {k_all/n_all:.0%}\n')
    w=csv.writer(f); w.writerow(['年','记载','三统差','单字可改对','可改对的改法（改后差）','偶合概率']); w.writerows(rows_out)
