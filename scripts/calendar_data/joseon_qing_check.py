"""《朝鮮王朝實錄》孝宗至肅宗（1650—1720）直书朔日与清初时宪历「西洋新法」复原（data/calendar/shixian_xinfa_check.csv 用时朔日）、
张表逐月比较，并以 Meeus 合朔（北京、汉阳地方平时）检验朝鲜朔日是否加汉阳里差：北京合朔在子夜前而汉阳已过子夜之月，朝鲜从北京日
则不加里差。写 data/calendar/joseon_vs_qing_check.csv。
用法（仓库根）：python3 scripts/calendar_data/joseon_qing_check.py
"""
import csv,sys,math,collections; sys.path.insert(0,'scripts/calendar_data')
import discriminating_records as dr, gaiyuan_date as g
GZ=g.GANZHI; G=lambda j:GZ[(j+49)%60]
def nm(k):
    T=k/1236.85; r=math.radians; s=math.sin
    JDE=2451550.09766+29.530588861*k+0.00015437*T*T-0.00000015*T**3+0.00000000073*T**4
    E=1-0.002516*T-0.0000074*T*T
    M=r(2.5534+29.10535670*k-0.0000014*T*T); Mp=r(201.5643+385.81693528*k+0.0107582*T*T+0.00001238*T**3)
    F=r(160.7108+390.67050284*k-0.0016118*T*T); O=r(124.7746-1.56375588*k+0.0020672*T*T)
    c=(-0.40720*s(Mp)+0.17241*E*s(M)+0.01608*s(2*Mp)+0.01039*s(2*F)+0.00739*E*s(Mp-M)-0.00514*E*s(Mp+M)+0.00208*E*E*s(2*M)
       -0.00111*s(Mp-2*F)-0.00057*s(Mp+2*F)+0.00056*E*s(2*Mp+M)-0.00042*s(3*Mp)+0.00042*E*s(M+2*F)+0.00038*E*s(M-2*F)
       -0.00024*E*s(2*Mp-M)-0.00017*s(O)-0.00007*s(Mp+2*M)+0.00004*s(2*Mp-2*F)+0.00004*s(3*M)+0.00003*s(Mp+M-2*F)
       +0.00003*s(2*Mp+2*F)-0.00003*s(Mp+M+2*F)+0.00003*s(Mp-M+2*F)-0.00002*s(Mp-M-2*F)-0.00002*s(3*Mp+M)+0.00002*s(4*Mp))
    return JDE+c
def nm_local(jdn,lon):
    k=round((jdn-2451550.1)/29.530588861); best=None
    for kk in (k-1,k,k+1):
        jd=nm(kk)-15/86400+lon/360; d=math.floor(jd+0.5)
        if best is None or abs(d-jdn)<abs(best[0]-jdn): best=(d,(jd+0.5-d)*24)
    return best
zh=dr.tables()['张表']
xf={}
f=open('data/calendar/shixian_xinfa_check.csv',encoding='utf-8'); next(f)
for r in csv.DictReader(f):
    y,m,d=map(int,r['用时朔日'].split('-')); xf[(int(r['农历年']),int(r['月']),r['闰']=='1')]=g.jdn(y,m,d)
jo={}
f=open('data/calendar/joseon_shuo.csv',encoding='utf-8'); next(f)
for r in csv.DictReader(f):
    if int(r['公历年'])>=1650: jo[(int(r['公历年']),int(r['月']),r['闰']=='1')]=(r['朔干支'],r['依据'],r['备注'])
rows=[]
for k,(gz,how,note) in sorted(jo.items()):
    z=zh.get(k)
    if z is None or how!='直书' or k[0]<1654:
        continue
    x=xf.get(k); bd,bh=nm_local(z,116.4); sd,sh=nm_local(z,126.98)
    rows.append([k[0],k[1],int(k[2]),gz,G(x) if x else '',G(z),int(gz==G(x)) if x else '',int(gz==G(z)),
                 f"{bh:.2f}",bd-z,f"{sh:.2f}",sd-z,note])
with open('data/calendar/joseon_vs_qing_check.csv','w',encoding='utf-8',newline='') as fo:
    fo.write('# 《朝鮮王朝實錄》1654—1720 年直书朔日与清新法复原、张表逐月比较（scripts/calendar_data/joseon_qing_check.py）。'
             '北京/汉阳合朔为 Meeus 平合朔加近似加数（地方平时，时）；日差为合朔所在日减张表朔日。朝鲜 1653 年改行时宪历。\n')
    w=csv.writer(fo,lineterminator='\n')
    w.writerow(['公历年','月','闰','朝鲜朔','新法复原朔','张表朔','合新法','合张表','北京合朔时','北京日差','汉阳合朔时','汉阳日差','朝鲜备注'])
    w.writerows(rows)
n=len(rows); print(n,'合新法',sum(r[6]==1 for r in rows),'合张表',sum(r[7]==1 for r in rows))
split=[r for r in rows if r[9]==0 and r[11]==1]
print('北京子夜前、汉阳子夜后之月',len(split),'朝鲜从北京',sum(r[7]==1 for r in split))
