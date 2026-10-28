"""秦与汉初（前221—太初元年四月）历法复原：按 ytliu0/ChineseCalendar QinHanCalendars 说明页所列公式与参数
（李忠林 2012《秦至汉初历法研究》三历元 A/B/C + 十九年七闰 3-3-3-2-3-3-2，汉文帝后元元年改章首）独立实现，
未执行上游代码；用说明页算例（元光四年十月朔壬申，前132-11-25）校验。评估结论见 doc/calendar/qin-han-calendar-evaluation.md。
N 年（年首最接近公历 y 年 1 月 1 日的历年）的十月朔起，月序 十、十一、十二、正……九、后九（闰）。"""
import math, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import gaiyuan_date as g
PL = 29 + 499/940
J = [0,0,1,0,0,1,0,0,1,0,1,0,0,1,0,0,1,0,1]
R = [0]
for k in range(1,19): R.append(R[-1]+12+J[k-1])
def params(y):
    if y >= -162: return 1646163.5, -179, 321   # 历法 C
    if y > -201:  return 1633701.5, -225, 174   # 历法 B
    return 1589523.5, -225, 1670                # 历法 A
LABELS=['10','11','12','1','2','3','4','5','6','7','8','9','后9']
def year(y):
    jd0, yz, dA = params(y)
    c = math.floor((y-yz)/19); k = y-yz-19*c
    A = dA + 235*c + R[k]; n = 12 + J[k]
    out=[]
    for j in range(n):
        if y == -201 and j >= 3:   # 汉高祖五年正月起改用历法 B
            jd = 1633701.5 + 470*PL + j*PL
        else:
            jd = jd0 + (A+j)*PL
        jdn = math.floor(jd + 0.5 + 1e-4)   # 正午 JDN；+1e-4 对应上游 .5001
        out.append((LABELS[j], jdn))
    return out
if __name__=='__main__':
    for lab,jdn in year(-130)[:3]: print(lab, jdn, g.from_jdn(jdn), g.ganzhi_of_jdn(jdn))
