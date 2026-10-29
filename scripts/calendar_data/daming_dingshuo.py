"""金重修大明历（赵知微，1182 年起行用）定朔、进朔，用以核验主表（宋历）与《金史》本纪明书朔日之异。

术文据《金史·历志》卷二十一、二十二（维基文库本两表「表略」）；日躔朓朒表、月离转定分朓朒表、二十四气日出分表取《元史·历志》
卷五十六耶律楚材庚午元历（「因金大明历」，日法、岁实、朔实、转终、诸表与重修大明历同，点校本校勘记屡以「金大明曆與驗算合」定字）。
庚午元历表中明显讹字按对称与累计改正：霜降初末率同穀雨，芒种末率同冬至初率，月离十六、十九日「朓」为「朒」。

常数（日法 5230 分）：岁实 1910224，朔实 154445，旬周 313800，转终分 144110 秒 6066（秒母一万），上元甲子距大定庚子（1180）
88639656 年。
算法：
1. 通积分 = 积年 × 岁实；闰余 = 通积分 mod 朔实；天正经朔 = 通积分 − 闰余；累加朔实得各月经朔。
2. 入气：经朔距天正冬至（恒气）之时，按气策分恒气；日躔朓朒「每日损益率」以初末定率、日差逐日等差，
   朓朒积按日累计，入日余分乘该日损益率（「求经朔弦望入气朓朒定数」）。
3. 入转：天正经朔入转 = 朔积分 mod 转终分，累加朔实；朓朒定数按日损益率，七、十四、二十一、二十八日按初数、末数分段。
4. 定朔 = 经朔「朓减朒加」二定数。
5. 进朔：秋分后（至春分）定朔小余 ≥ 日法 3/4 进一日；春分后（至秋分）限为 3/4 日法 −（春分日出分 − 定朔日出分）/3；
   「有交，亏初在日入前者不进」——本脚本不算日食，以定朔入交距交点日数（近交）标出，供逐条判读。
进朔限校准：本纪明书朔日、閏月与天文志在临限之月显示实际进朔限比术文高 0.14—0.36 时（1178 年十月超 0.14 时不进、
1162 年超 0.10 时不进致閏二月；1152 年四月超 0.36 时、1214 年正月超 0.37 时而进），取中点 0.25 时（CALIB），与
rust/core/src/calendar/daming_li.rs 的 JINSHUO_CALIB 同；输出兼列术文原限之结果。
干支：上元甲子日夜半为日序 0（甲子）；以 1180 年天正冬至（1179 年 12 月）与儒略日对齐。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/daming_dingshuo.py
"""
import csv
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import discriminating_records as d  # noqa: E402
import gaiyuan_date as g  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]

F = 5230
SUI = 1910224                  # 岁实
SHUO = 154445                  # 朔实
XUNZHOU = 313800               # 旬周 = 60 × 日法
ZHUAN = Fr(1441106066, 10000)  # 转终分
JIAO = Fr(1423199368, 10000)   # 交终分
JI_1180 = 88639656             # 上元甲子至大定庚子积年
QI = Fr(SUI, 24)               # 气策（分）
CALIB = Fr(F, 96)              # 进朔限校准 0.25 时
Q_DAYS = QI / F
# 日躔朓朒：冬至起十二气（夏至后同数异名）损益率（日法分）、初定率（万分之一分）、末定率；朒为正（加）、朓为负
SUN_L = [276, 232, 185, 135, 83, 29, -29, -83, -135, -185, -232, -276]
SUN_A = [194864, 166874, 136911, 104670, 71114, 35631, 2480, 38576, 73359, 107136, 138940, 167852]
SUN_B = [167852, 138019, 106214, 72745, 37963, 2480, 35631, 70501, 104056, 135991, 165952, 194864]
# 恒气初日出分（冬至起，分；夏至后对称）
SUNRISE = [Fr(156792, 100), Fr(155752, 100), Fr(152879, 100), Fr(148523, 100), Fr(1434), Fr(136614, 100), Fr(129696, 100),
           Fr(123227, 100), Fr(117318, 100), Fr(112234, 100), Fr(108248, 100), Fr(105642, 100), Fr(104707, 100)]
# 月离：入转一—二十八日损益率（日法分，益为朓向，即疾为正、计入定朔时取负）；朓朒积（朓 +）
MOON_L = [513, 469, 411, 332, 243, 141, None, -63, -164, -258, -352, -427, -481, None,
          -505, -462, -395, -309, -219, -117, None, 86, 184, 278, 368, 438, 493, 293]
# 七、十四、二十一日：初数、初率、末数、末率；二十八日只有初数 2901
SPLIT = {6: (4648, 43, 582, -4), 13: (4065, -403, 1165, -117), 20: (3483, -27, 1747, 11)}


def moon_ji():
    """入转各日之前朓朒积（朓 +）。"""
    acc, out = 0, []
    for i, l_ in enumerate(MOON_L):
        out.append(acc)
        acc += SPLIT[i][1] + SPLIT[i][3] if i in SPLIT else l_
    assert acc == 0, acc
    return out


MOON_JI = moon_ji()


def moon_def(x):
    """入转 x（分，Fraction）→ 月离朓朒定数（分，朓 +）。"""
    i, r = int(x // F), x % F
    if i in SPLIT:
        n0, r0, n1, r1 = SPLIT[i]
        return MOON_JI[i] + (r0 * r / n0 if r <= n0 else r0 + r1 * (r - n0) / n1)
    if i >= 27:
        return MOON_JI[27] + 293 * min(r, 2901) / 2901
    return MOON_JI[i] + MOON_L[i] * r / F


def sun_def(t):
    """距天正冬至 t（分）→ 日躔朓朒定数（分，朒 +）。恒气内逐日损益率等差（初定率起，日差 = (初 − 末)/(气日 − 1)）。"""
    k, x = divmod(t, QI)
    k = int(k) % 24
    half, kk = divmod(k, 12)
    sign = 1 if half == 0 else -1           # 夏至后朓朒异名
    ji = sign * sum(SUN_L[:kk])
    a, b = Fr(SUN_A[kk], 10000), Fr(SUN_B[kk], 10000)
    s = 1 if SUN_L[kk] > 0 else -1
    day, r = int(x // F), x % F
    cha = (a - b) / (Q_DAYS - 1)
    acc = sum(a - i * cha for i in range(day)) + (a - day * cha) * r / F
    # 逐日损益率之和按本气损益率归一（初末率取至万分之一，累计与表列损益率差不足 1 分）
    tot = sum(a - i * cha for i in range(int(Q_DAYS))) + (a - int(Q_DAYS) * cha) * (Q_DAYS - int(Q_DAYS))
    return ji + sign * s * acc * abs(SUN_L[kk]) / tot


def sunrise(t):
    """距天正冬至 t（分）→ 日出分（恒气间线性）。"""
    k, x = divmod(t, QI)
    k = int(k) % 24
    i0 = k if k <= 12 else 24 - k
    i1 = k + 1 if k < 12 else 23 - k
    return SUNRISE[i0] + (SUNRISE[i1] - SUNRISE[i0]) * x / QI


def year_moons(y, n=14, calib=CALIB):
    """冬至在 y−1 年之岁各月：[(经朔分, 定朔分, 进朔后日序, 定朔日序, 定朔小余时, 进朔限时（术文）, 定朔距交日)]，自上元甲子夜半起算。"""
    ji = JI_1180 + (y - 1180)
    tong = SUI * ji
    run = tong % SHUO
    t0 = tong - run
    out = []
    for m in range(n):
        jing = t0 + m * SHUO
        dt = jing - tong                       # 距天正冬至（负 = 冬至前）
        sd = sun_def(dt % (24 * QI))
        md = moon_def((t0 % ZHUAN + m * SHUO) % ZHUAN)
        ding = jing + sd - md
        day, xiao = int(ding // F), ding % F
        ts = (ding - tong) % (24 * QI)
        if 6 * QI <= ts < 18 * QI:             # 春分后至秋分
            lim = Fr(3 * F, 4) - (SUNRISE[6] - sunrise(ts)) / 3
        else:
            lim = Fr(3 * F, 4)
        jiao = ((t0 % JIAO + m * SHUO) % JIAO + sd - md) / F   # 定朔加时入交日（略去月离入交度修正）
        near = min(jiao, abs(jiao - JIAO / F / 2), JIAO / F - jiao)
        out.append((jing, ding, day + (1 if xiao >= lim + calib else 0), day, float(xiao / F * 24), float(lim / F * 24), float(near)))
    return out


def epoch_offset():
    """上元日序 → JDN：1180 年天正冬至（1179 年 12 月中下旬）干支对齐。"""
    dz = SUI * JI_1180 // F
    approx = g.jdn(1179, 12, 22)
    return min((approx + k for k in range(-40, 40) if (approx + k + 49) % 60 == dz % 60), key=lambda j: abs(j - approx)) - dz


def main():
    off = epoch_offset()
    dz = SUI * JI_1180 // F
    print("1180 年天正冬至", g.GANZHI[dz % 60], g.from_jdn(dz + off))
    moons = {}
    for y in range(1130, 1240):
        for jing, ding, jin, day, h, lim, near in year_moons(y):
            moons[day + off] = (jin + off, h, lim, near)
    cal_h = float(CALIB / F * 24)
    main_t = d.tables()["主表"]
    main_days = set(main_t.values())

    def nearest(j):
        k = min(moons, key=lambda x: abs(x - j))
        return k, moons[k]

    # 1. 与主表（宋历）逐月
    stat = {}
    for (y, m, lp), s in main_t.items():
        if not 1130 <= y <= 1234:
            continue
        k, (jin, h, lim, near) = nearest(s)
        per = "1182—1234" if y >= 1182 else "1130—1181"
        st = stat.setdefault(per, [0, 0, 0])
        st[0] += 1
        st[1] += jin == s
        st[2] += k == s
    for per, (n, ok, ok0) in sorted(stat.items()):
        print(f"{per}：主表（宋历）朔与重修大明进朔后同日 {ok}/{n}（不进朔 {ok0}/{n}）")
    # 2. 本纪明书朔日
    with open(ROOT / "data/calendar/main_table_check.csv", encoding="utf-8") as f:
        rows = list(csv.DictReader(line for line in f if not line.startswith("#")))
    out = []
    for r in rows:
        if r["政权"] != "金" or not r["主表朔 JDN"] or r["类"] in ("其他", "差一月"):
            continue
        y, rec, mt = int(r["年"]), int(r["本纪朔 JDN"]), int(r["主表朔 JDN"])
        k, (jin, h, lim, near) = nearest(mt)
        out.append({"书": r["书"], "年": y, "纪年": r["纪年"], "月": r["月"], "本纪朔": r["本纪朔"], "主表朔": g.ganzhi_of_jdn(mt),
                    "大明定朔": g.ganzhi_of_jdn(k), "定朔小余（时）": f"{h:.2f}", "术文进朔限（时）": f"{lim:.2f}",
                    "术文进朔": g.ganzhi_of_jdn(k + (h >= lim)), "大明进朔（校准）": g.ganzhi_of_jdn(jin),
                    "临限": "是" if 0 <= h - lim < 2 * cal_h else "", "距交（日）": f"{near:.1f}",
                    "本纪合主表": int(rec == mt), "本纪合大明不进朔": int(rec == k), "本纪合术文进朔": int(rec == k + (h >= lim)),
                    "本纪合大明进朔": int(rec == jin),
                    "日食": r["日食"], "原文": r["原文"]})
    with open(ROOT / "data/calendar/daming_dingshuo_check.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 《金史》本纪明书朔日与重修大明历定朔（scripts/calendar_data/daming_dingshuo.py）、主表（宋历）比较；"
                "1182 年前行杨级大明历（术不传），以重修大明历试算。距交 = 定朔加时距黄白交点日数（< 1.5 日方可有日食）。\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    for per, lo, hi in (("1137—1181（杨级历，试算）", 1137, 1181), ("1182—1234（重修大明历）", 1182, 1234)):
        sub = [o for o in out if lo <= o["年"] <= hi]
        print(f"本纪明书朔日 {per} {len(sub)} 条：合主表 {sum(o['本纪合主表'] for o in sub)}，"
              f"合大明不进朔 {sum(o['本纪合大明不进朔'] for o in sub)}，合术文进朔 {sum(o['本纪合术文进朔'] for o in sub)}，"
              f"合大明进朔（校准）{sum(o['本纪合大明进朔'] for o in sub)}")
        for o in sub:
            if not (o["本纪合主表"] and o["本纪合大明进朔"]):
                print("  ", o["年"], o["月"], o["本纪朔"], "主表", o["主表朔"], "大明", o["大明定朔"], o["定朔小余（时）"], "限",
                      o["术文进朔限（时）"], "进", o["大明进朔（校准）"], "距交", o["距交（日）"], o["原文"][:18])


if __name__ == "__main__":
    main()
