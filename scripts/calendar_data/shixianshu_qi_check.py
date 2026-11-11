"""《后编》中气用时（京师）与时宪书卷首「都城順天府節氣時刻」所载对照。用法（仓库根）：uv run --project scripts python scripts/calendar_data/shixianshu_qi_check.py 农历年"""
import sys, math
sys.path.insert(0, 'scripts/calendar_data')
import shixian_houbian as hb
G="甲乙丙丁戊己庚辛壬癸";Z="子丑寅卯辰巳午未申酉戌亥"
def gz(j): g=(j+49)%60; return G[g%10]+Z[g%12]
def shichen(h):
    # 用时小时 → 时辰初正刻分（子初 23 时）
    m=round(h*60) % 1440
    zi=((m+60)//120)%12; rem=(m+60)%120
    half='初' if rem<60 else '正'; rem%=60
    return f"{Z[zi]}{half}{rem//15}刻{rem%15}分"
y=int(sys.argv[1])
NAMES=['冬至','大寒','雨水','春分','谷雨','小满','夏至','大暑','处暑','秋分','霜降','小雪']
for sui in (y, y+1):
    q=hb.jieqi_times(sui)
    for k in range(0,24,2):
        t=q[k]; u=t+hb.shi_cha(t)
        j=hb.jdn_of(math.floor(u)); h=(u%1)*24
        print(sui, NAMES[k//2], gz(j), f"{h:6.3f}", shichen(h))
