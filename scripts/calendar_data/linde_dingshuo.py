"""麟德历定朔（《新唐书·历志二》卷二十六），用以复核主表 665—728 年朔日与本纪明书朔日之异。

常数：总法 1340，朞实 489428，常朔实 39571，上元积年至麟德元年（664）269880 算；日躔表（各气先后率、盈朒积、消息总）、
月离表（变日 1—28 增减率、迟速积），变周 443077、变奇法 12。
算法（以总法之分为单位）：
1. 朞总 = 朞实 × 积算；闰余 = 朞总 mod 常朔实；天正常朔 = 朞总 − 闰余；累加常朔实得各月常朔。
2. 日躔：常气 = 冬至 + k × 朞实/24；定气 = 常气「息减、消加」消息总。常朔所在定气内，盈朒积按先后率在定气长度内线性插值，
   「盈加、朒减」常朔得盈朒朔。
3. 月离：总实（天正常朔夜半）× 奇法 mod 变周 ÷ 奇法 为天正常朔夜半入变，加常朔小余为经辰所入，各月累加常朔实；
   再加盈朒积，得盈朒经辰所入。迟速积按增减率线性插值（志文「速要」之法：「但以入餘乘增減率，總法而一」，不用二次差）。
   「速减、迟加」盈朒小余，得定朔。
4. 定朔小余满总法进日；志文无进朔之法（大衍历始有），日名即定朔所在日。
以甲子日夜半为上元，日序 mod 60 为干支（0 = 甲子）。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/linde_dingshuo.py
"""
import csv
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import discriminating_records as d  # noqa: E402
import gaiyuan_date as g  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]

F = 1340
QI = 489428          # 朞实
SHUO = 39571         # 常朔实
JI_664 = 269880      # 上元至麟德元年积算
BIAN_ZHOU, BIAN_QI = 443077, 12
# 日躔表：中节（冬至起）先后率（先 +、后 −）、盈朒积（盈 +、朒 −）、消息总（息 +、消 −）
XIANHOU = [54, 46, 38, 38, 46, 54, -54, -46, -38, -38, -46, -54, 54, 46, 38, 38, 46, 54, -54, -46, -38, -38, -46, -54]
YINGNU = [0, 54, 100, 138, 176, 222, 276, 222, 176, 138, 100, 54, 0, -54, -100, -138, -176, -222, -276, -222, -176, -138, -100, -54]
XIAOXI = [0, 722, 1340, 1854, 2368, 2986, 3708, 2986, 2368, 1854, 1340, 722, 0, -722, -1340, -1854, -2368, -2986, -3708, -2986,
          -2368, -1854, -1340, -722]
# 月离表：变日 1—28 增减率（增 +、减 −）、迟速积（速 +、迟 −）。十日减率从旧书作六十二（新书六十四，校勘记）；
# 七日、十四日、二十一日、二十八日初末分率按志文「初、末」数分段
ZENG = [134, 117, 99, 78, 56, 33, 9, -14, -38, -62, -85, -104, -121, -102, 128, 115, 95, 74, 52, 28, 4, -20, -44, -68, -89, -108,
        -125, -71]
CHISU = [0, 134, 251, 350, 428, 484, 517, 526, 512, 474, 412, 327, 223, 102, -29, -157, -272, -367, -441, -493, -521, -525, -505,
         -461, -393, -304, -196, -71]
# 第 14 日（index 13）初减 102 后末增 29 入迟；第 7、21 日末减隐；第 28 日只余 743 分（变日 27 日 743 分）


def chisu(x):
    """入变 x（日，Fraction）→ 迟速积（总法分，速 +）；按日内增减率线性。"""
    i = int(x)
    frac = x - i
    if i >= 27:
        return CHISU[27] + ZENG[27] * frac * F / 743  # 末日只 743 分
    if i == 13:  # 十四日：初减百二（至速 0），末增二十九入迟
        chu = Fr(1042, F)
        if frac <= chu:
            return CHISU[13] - 102 * frac / chu
        return -29 * (frac - chu) / (1 - chu)
    return CHISU[i] + ZENG[i] * frac


def year_moons(y, n=14):
    """农历年 y（以天正冬至起，即 y−1 年冬至所在月起）的常朔与定朔：[(常朔分, 定朔分)]，自上元甲子夜半起算。"""
    ji = JI_664 + (y - 664)
    qizong = QI * ji
    run = qizong % SHUO
    t0 = qizong - run                      # 天正常朔（分）
    dongzhi = Fr(qizong)
    zongshi = t0 - t0 % F                  # 天正常朔夜半
    bian0 = Fr((zongshi * BIAN_QI) % BIAN_ZHOU, BIAN_QI) / F   # 天正常朔夜半入变（日）
    # 定气：k = 0..47 覆盖两年
    qi_len = Fr(QI, 24)
    dingqi = [dongzhi + k * qi_len - XIAOXI[k % 24] for k in range(-2, 50)]
    out = []
    for m in range(n):
        ch = t0 + m * SHUO                 # 常朔
        # 所在定气
        k = max(j for j in range(len(dingqi) - 1) if dingqi[j] <= ch)
        kk = (k - 2) % 24
        seg = dingqi[k + 1] - dingqi[k]
        yn = YINGNU[kk] + XIANHOU[kk] * (ch - dingqi[k]) / seg
        # 入变：天正常朔夜半入变 + 常朔小余 + m 个常朔实，模变周
        bian = (bian0 + Fr(t0 % F, F) + Fr(m * SHUO, F) + yn / F) % (Fr(BIAN_ZHOU, BIAN_QI) / F)
        ding = ch + yn - chisu(bian)
        out.append((ch, ding))
    return out


def epoch_offset():
    """上元甲子夜半日序 → JDN 之差：664 年天正冬至（663 年 12 月）日序与干支对齐。"""
    dz_day = QI * JI_664 // F
    approx = g.jdn(663, 12, 20)
    return min((approx + k for k in range(-40, 40) if (approx + k + 49) % 60 == dz_day % 60), key=lambda j: abs(j - approx)) - dz_day


def main():
    main_t = d.tables()["主表"]
    off = epoch_offset()
    dz_day = QI * JI_664 // F
    print("664 年天正冬至", g.GANZHI[dz_day % 60], g.from_jdn(dz_day + off))
    moons = sorted({ding for y in range(664, 730) for _, ding in year_moons(y)})
    by_jdn = {v: k for k, v in main_t.items()}
    # 1. 与主表逐月：主表比麟德定朔晚一日者是否都在定朔小余之后段（进朔）
    late = []
    same = n = 0
    for t in moons:
        j = int(t // F) + off
        near = [k for k in range(j - 2, j + 3) if k in by_jdn and 665 <= by_jdn[k][0] <= 728]
        if not near:
            continue
        k = min(near, key=lambda x: abs(x - j))
        n += 1
        same += k == j
        if k == j + 1:
            late.append(float((t % F) / F * 24))
    print(f"665—728 年：麟德定朔与主表同日 {same}/{n}；主表晚一日 {len(late)} 月，其定朔小余 {min(late):.1f}—{max(late):.1f} 时")
    # 2. 本纪明书朔日（main_table_check.csv，类非「其他」「差一月」）三说比较
    def linde(jdn_hint, jinshuo):
        t = min(moons, key=lambda x: abs(int(x // F) + off - jdn_hint))
        h = float((t % F) / F * 24)
        return int(t // F) + off + (1 if jinshuo and h >= 18 else 0), h
    out = []
    with open(ROOT / "data/calendar/main_table_check.csv", encoding="utf-8") as f:
        rows = list(csv.DictReader(line for line in f if not line.startswith("#")))
    for r in rows:
        y = int(r["年"])
        if not 665 <= y <= 728 or not r["主表朔 JDN"] or r["类"] in ("其他", "差一月"):
            continue
        rec, mt = int(r["本纪朔 JDN"]), int(r["主表朔 JDN"])
        d0, h = linde(mt, False)
        d1, _ = linde(mt, True)
        out.append({"书": r["书"], "年": y, "纪年": r["纪年"], "月": r["月"], "本纪朔": r["本纪朔"], "主表朔": g.ganzhi_of_jdn(mt),
                    "麟德定朔": g.ganzhi_of_jdn(d0), "麟德定朔小余（时）": f"{h:.1f}", "麟德进朔（≥18 时）": g.ganzhi_of_jdn(d1),
                    "本纪合主表": int(rec == mt), "本纪合麟德无进朔": int(rec == d0), "本纪合麟德进朔": int(rec == d1),
                    "日食": r["日食"], "原文": r["原文"]})
    with open(ROOT / "data/calendar/linde_dingshuo_check.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 665—728 年本纪明书朔日与麟德历定朔（scripts/calendar_data/linde_dingshuo.py，线性插值「速要」法）、主表比较。\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    tot = len(out)
    print(f"本纪明书朔日 {tot} 条：合主表 {sum(o['本纪合主表'] for o in out)}，合麟德无进朔 {sum(o['本纪合麟德无进朔'] for o in out)}，"
          f"合麟德进朔 {sum(o['本纪合麟德进朔'] for o in out)}")
    for o in out:
        if not (o["本纪合主表"] and o["本纪合麟德无进朔"]):
            print("  ", o["年"], o["月"], o["本纪朔"], "主表", o["主表朔"], "麟德", o["麟德定朔"], o["麟德定朔小余（时）"], "时", o["原文"][:16])


if __name__ == "__main__":
    main()
