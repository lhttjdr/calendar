"""古六历书朔逐历分析的输入：把各处「明书朔（晦）」的记录与对照朔表整理成一张表，供
rust/core/examples/guliuli_shuo.rs 逐历推朔比较（结果见 doc/calendar/guliuli-shuo.md）。

每行给出记录所断言的「朔日」儒略日数 shuo_jdn（晦日 +1，先晦一日 +2），不含任何古六历推算：
- 春秋经书朔（data/calendar/chunqiu_check.csv 中 朔=1，29 条）：干支取距春秋历（上游，今人排定）同名月朔最近的那一日。
  春秋历只用来定「哪一个 60 日周期」，不影响日的干支；月份讹误超过半月才会错周期（26/29 与春秋历恰合，不在此列）。
- 左传书朔（data/calendar/zuozhuan_check.csv 中 朔=1 且不与同年经文重出，13 条）：同上。
- 汉书日食朔晦（data/phenomena/hanshu_solar_eclipses_check.csv，太初改历前 29 条）：表中 jdn 为该干支在秦汉初复原历
  该月的位置；verdict 为天象检验（「都城可见」「无日食」等），当日确有日食者，记录日由天象独立确定。
- 实历朔日表（vendor/calendar/chinese_calendar/实历朔日表.csv）前 106—前 105 年的全部月朔（主表与古六历唯一重叠段）。
- 秦汉初复原历（李忠林 2012 方案，scripts/calendar_data/qinhan_eval/hzx.py）前 220—前 105 年逐月朔：复原结果，不是史实。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/guliuli_shuo_records.py
输出 data/calendar/guliuli_shuo_records.csv。
"""
import csv
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.path.insert(0, str(Path(__file__).resolve().parent / 'qinhan_eval'))
import gaiyuan_date as g  # noqa: E402
import hzx  # noqa: E402
from chunqiu_check import chunqiu_year  # noqa: E402

OUT = ROOT / 'data/calendar/guliuli_shuo_records.csv'


def nearest_ganzhi_day(ref: int, gz: str) -> int:
    """距 ref 最近、干支为 gz 的儒略日数（±30 日内唯一）。"""
    k = g.GANZHI.index(gz)
    off = (k - (ref + 49) % 60) % 60
    return ref + off if off < 30 else ref + off - 60


def rows_lines(path):
    with open(path, encoding='utf-8') as f:
        return list(csv.DictReader(line for line in f if not line.startswith('#')))


out = []
# 1. 春秋经书朔
for r in rows_lines(ROOT / 'data/calendar/chunqiu_check.csv'):
    if r['朔'] != '1':
        continue
    y, m = int(r['年']), int(r['月'])
    ref = chunqiu_year(y)[m - 1]
    d = nearest_ganzhi_day(ref, r['干支'])
    out.append(dict(group='春秋经书朔', year=y, month=m, ganzhi=r['干支'], position='朔', shuo_jdn=d,
                    note=('日食' if r['日食'] == '1' else ''), text=f"{r['公']}{r['公年']}年 {m}月{r['干支']}朔"))
# 2. 左传书朔（不与经重出）
for r in rows_lines(ROOT / 'data/calendar/zuozhuan_check.csv'):
    if r['朔'] != '1' or r['同经'] != '0':
        continue
    y, m = int(r['年']), int(r['月'])
    months = chunqiu_year(y)
    ref = months[m - 1] if m - 1 < len(months) else months[-1]
    d = nearest_ganzhi_day(ref, r['干支'])
    out.append(dict(group='左传书朔', year=y, month=m, ganzhi=r['干支'], position='朔', shuo_jdn=d, note='',
                    text=f"{r['公']}{r['公年']}年 {m}月{r['干支']}朔"))
# 3. 汉书日食（太初前）
SHIFT = {'朔': 0, '晦': 1, '先晦一日': 2}
for r in rows_lines(ROOT / 'data/phenomena/hanshu_solar_eclipses_check.csv'):
    y, m = int(r['year']), int(r['month'])
    if y > -104 or r['position'] not in SHIFT:
        continue
    d = int(r['jdn']) + SHIFT[r['position']]
    out.append(dict(group='汉书日食朔晦', year=y, month=m, ganzhi=r['ganzhi'], position=r['position'], shuo_jdn=d,
                    note=r['verdict'], text=f"{r['emp']}{r['era']}{r['year_in_era']}年 {m}月{r['ganzhi']}{r['position']}"))
# 4. 实历朔日表重叠段
with open(ROOT / 'vendor/calendar/chinese_calendar/实历朔日表.csv', encoding='utf-8') as f:
    for r in csv.DictReader(f):
        y = int(r['lunar_year'])
        if y > -104:
            break
        gy, gm, gd = int(r['gregorian_year']), int(r['gregorian_month']), int(r['gregorian_day'])
        d = g.jdn(gy, gm, gd)  # 1582 前按儒略历
        m = int(r['lunar_month'])
        out.append(dict(group='实历朔日表', year=y, month=m, ganzhi=g.ganzhi_of_jdn(d), position='月朔', shuo_jdn=d,
                        note='闰' if r['is_leap'] == '1' else '', text=f"{y}年{'闰' if r['is_leap'] == '1' else ''}{m}月"))
# 5. 秦汉初复原历
for y in range(-220, -104):
    for lab, d in hzx.year(y):
        out.append(dict(group='秦汉初复原历', year=y, month=lab, ganzhi=g.ganzhi_of_jdn(d), position='月朔', shuo_jdn=d,
                        note='', text=f"{y}年{lab}月"))

with open(OUT, 'w', encoding='utf-8', newline='') as f:
    f.write('# 古六历书朔逐历分析输入（scripts/calendar_data/guliuli_shuo_records.py 生成）：shuo_jdn 为记录所断言之朔日（晦 +1、先晦一日 +2）\n')
    w = csv.DictWriter(f, fieldnames=['group', 'year', 'month', 'ganzhi', 'position', 'shuo_jdn', 'note', 'text'])
    w.writeheader()
    w.writerows(out)
from collections import Counter
print(OUT, Counter(r['group'] for r in out))
