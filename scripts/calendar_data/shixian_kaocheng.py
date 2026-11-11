"""清时宪历《历象考成》（康熙甲子元法，雍正四年起「造時憲書一遵曆象考成之法」）推日躔、月离、节气、合朔，
照《清史稿·时宪志》卷四十八（康熙甲子元法中）逐步复原，卷四十七（立法之原）作模型说明与自检。

角度以度计；中法宫度自冬至起（= 现代黄经 − 270°）。日以京师子正为界。
- 天正冬至：中积 = 周岁 365.2421875 × 积年（积年 = 距元年数 − 1，元年为康熙二十三年甲子，天正冬至在癸亥年十一月），加气应。
- 日躔：引数 = 平行 − 最卑；「以本輪半徑三分之二為對正角之邊，以引數為一角，求得對角之邊倍之。又求得對又一角之邊，
  與本天半徑相加減……以加倍之數為小邊，加減本天半徑之數為大邊……求得對小邊之角為均數」。
- 月离：用时太阴平行（均数时差、升度时差化为时差行）；初均（本轮半径之半，对角之边三因之）；二均（次轮通弦法，
  加减号依「加減泛限」「定限」）；三均（次均轮）；交均与黄白大距（弧三角形）、升度差。
- 节气：子正日躔实行内插（平时），再以时差总化用时；合朔：本日次日子正日月实行内插。太阴用时平行已化至用时子正，
  故所得合朔时刻即用时（「平时」对照：不加时差行）。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/shixian_kaocheng.py
"""
import math

ZHOU_SUI = 365.2421875
SUN_DAILY = 3548.3305169 / 3600
ZUIBEI_YEAR = 61.16666 / 3600
ZUIBEI_DAY = 0.167469 / 3600
R = 1e7
SUN_BENLUN, SUN_JUNLUN = 268812.0, 89604.0
ZUIBEI_YING = 7 + 10 / 60 + 11 / 3600 + 10 / 216000
QI_YING = 7.656374926
HUANG_CHI = 23 + 29 / 60 + 30 / 3600

MOON_DAILY = 47435.021177 / 3600
MOON_HOURLY = 1976.4592157 / 3600
BEI_DAILY = 401.077477 / 3600        # 月孛（最高）每日行
JIAO_DAILY = 190.64 / 3600
MOON_BENLUN, MOON_JUNLUN = 580000.0, 290000.0
CI_LUN, CI_JUNLUN = 217000.0, 117500.0
DA_JU_ZHONG = 5 + 8 / 60
DA_JU_BANJIAO = 9.5 / 60
MOON_YING = 30 + 8 + 40 / 60 + 57 / 3600 + 16 / 216000
BEI_YING = 3 * 30 + 4 + 49 / 60 + 54 / 3600 + 9 / 216000
JIAO_YING = 6 * 30 + 27 + 13 / 60 + 37 / 3600 + 48 / 216000

YUAN_YEAR = 1684     # 康熙二十三年甲子：天正冬至在癸亥年（1683）十一月
EPOCH_DZ_JDN = None  # 康熙甲子天正冬至所在日（干支序 7，辛未）之儒略日数，由 _epoch() 求


def sin(d):
    return math.sin(math.radians(d))


def cos(d):
    return math.cos(math.radians(d))


def tan(d):
    return math.tan(math.radians(d))


def atan2(y, x):
    return math.degrees(math.atan2(y, x))


def norm(d):
    return d % 360.0


def fold(d):
    d = norm(d)
    return d if d <= 180 else 360 - d


def signed(d):
    return norm(d + 180) - 180


def tri_opp(a, b, c_deg):
    """平三角形：两边 a、b 夹角 C，求对 b 之角与第三边"""
    ang = atan2(b * sin(c_deg), a - b * cos(c_deg))
    return ang, math.sqrt(a * a + b * b - 2 * a * b * cos(c_deg))


def jdn(y, m, d):
    a = (14 - m) // 12
    yy, mm = y + 4800 - a, m + 12 * a - 3
    return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - yy // 100 + yy // 400 - 32045


def _epoch():
    """康熙甲子天正冬至：气应 7.656… 日，即甲子后第 7 日（辛未）。取 1683 年 12 月 20—24 日中干支为辛未者。"""
    for d in range(18, 26):
        j = jdn(1683, 12, d)
        if (j + 49) % 60 == 7:
            return j
    raise ValueError


EPOCH_DZ_JDN = _epoch()


def jdn_of(daynum):
    return EPOCH_DZ_JDN - 7 + daynum


def daynum_of(j):
    return j - (EPOCH_DZ_JDN - 7)


def dongzhi(y):
    ji = y - YUAN_YEAR
    return ZHOU_SUI * ji + QI_YING, ji


def sui_of(t):
    y = YUAN_YEAR + int(math.floor((t - QI_YING) / ZHOU_SUI))
    while dongzhi(y + 1)[0] <= t:
        y += 1
    while dongzhi(y)[0] > t:
        y -= 1
    return y


def sun_jun(yin):
    """日躔均数：本轮均轮法。对正角之边 h = 本轮 × 2/3（= 本轮 − 均轮），小边 = 2h·sin(引数)，
    大边 = 本天半径 ∓ h·cos(引数)（引数三宫至八宫相加、九宫至二宫相减，即 R − h·cos），均数 = atan(小边/大边)；
    引数初宫至五宫为加。"""
    h = SUN_BENLUN * 2 / 3
    m = norm(yin)
    jun = atan2(2 * h * abs(sin(m)), R - h * cos(m))
    return jun if m <= 180 else -jun


def sun(t):
    """(平行, 引数, 均数, 实行)"""
    y = sui_of(t)
    dz, ji = dongzhi(y)
    ping = norm(SUN_DAILY * (t - dz))
    next_zi = math.floor(dz) + 1
    zuibei = ZUIBEI_YING + ZUIBEI_YEAR * ji + ZUIBEI_DAY * (t - next_zi)
    yin = norm(ping - zuibei)
    jun = sun_jun(yin)
    return ping, yin, jun, norm(ping + jun)


def shi_cha(t):
    """时差总（日）：均数时差（均数变时，均数加者时差为减）+ 升度时差（黄、赤经差变时，二分后加、二至后减）。"""
    _, _, jun, shi = sun(t)
    lam = norm(shi + 270)
    ra = norm(atan2(cos(HUANG_CHI) * sin(lam), cos(lam)))
    return (-jun + signed(lam - ra)) / 360


def er_jun(m, chu_jun_signed, chu_jun_abs, d1, ci_yin):
    """二均数（卷四十八「求白道實行」）。m 引数，chu_jun_signed 初均（减为负），d1 次轮最近点距地心，ci_yin 次引。
    返回 (二均（加为正）, 次均轮心距地心)。"""
    th = norm(ci_yin)
    chord = 2 * CI_LUN * abs(sin(th))                         # 倍次引之通弦
    a1 = chu_jun_abs + (180 - m if m < 180 else m - 180)      # 初均数与引数减半周之度相加
    # 次引距象限度
    if th < 90:
        dq = 90 - th
    elif th < 180:
        dq = th - 90
    elif th < 270:
        dq = 90 - (th - 180)
    else:
        dq = th - 270
    jian = chu_jun_signed < 0
    over_q = (90 <= th < 180) or (th >= 270)                 # 过象限或过三象限
    add = over_q if jian else not over_q
    ang = a1 + dq if add else a1 - dq
    ang = abs(ang)
    if ang > 180:
        ang = 360 - ang
    if chord == 0 or ang == 0 or ang == 180:
        mag, side = 0.0, d1 + (chord if ang == 0 else -chord)
        side = abs(side)
    else:
        mag, side = tri_opp(d1, chord, ang)
    # 加减号：加减泛限 = 初均 + 均轮心距最卑度（= |180 − 引数|）
    fan = chu_jun_abs + abs(180 - m)
    two = norm(2 * th)
    if abs(fan - 90) < 1e-12:
        same = True
    elif fan < 90:
        ding = 2 * (90 - fan)
        x = two if jian else norm(360 - two)
        same = x > ding
    else:
        ding = 2 * (fan - 90)
        x = norm(360 - two) if jian else two
        same = x > ding
    sg_chu = -1 if jian else 1
    sg = sg_chu if same else -sg_chu
    return sg * mag, side


def moon(t, yongshi=True):
    """时刻 t（日序；子正）：(黄道实行, 太阳实行, 细目)。yongshi 为真则太阴平行化至用时（「求用時太陰平行」）。"""
    n = t - (math.floor(dongzhi(YUAN_YEAR)[0]) + 1)
    ping = norm(MOON_YING + MOON_DAILY * n)
    bei = norm(BEI_YING + BEI_DAILY * n)
    jiao = norm(JIAO_YING - JIAO_DAILY * n)
    s_ping, s_yin, s_jun, s_shi = sun(t)
    if yongshi:
        cha_sec = shi_cha(t) * 86400                           # 时差总（时之秒）
        ping = norm(ping - cha_sec * MOON_HOURLY / 3600)       # 时差总为加者则减
    # 初实行
    m = norm(ping - bei)
    h = MOON_JUNLUN                                          # 本轮半径之半
    small = 3 * h * abs(sin(m))
    big = R + h * cos(m)                                     # 九宫至二宫相加，三宫至八宫相减
    chu_abs = atan2(small, big)
    d1 = math.hypot(small, big)                              # 次轮最近点距地心
    chu_signed = -chu_abs if m <= 180 else chu_abs           # 引数初宫至五宫为减
    chu = norm(ping + chu_signed)
    ci_yin = norm(chu - s_shi)
    e2, side = er_jun(m, chu_signed, chu_abs, d1, ci_yin)
    two = norm(2 * ci_yin)
    e3, _ = tri_opp(side, CI_JUNLUN, fold(two))
    e3 = e3 if two < 180 else -e3
    bai = norm(chu + e2 + e3)
    # 黄道实行：弧三角形求黄白大距与交均
    c = fold(two)
    a, b = DA_JU_ZHONG, DA_JU_BANJIAO
    cos_i = cos(a) * cos(b) + sin(a) * sin(b) * cos(c)
    da_ju = math.degrees(math.acos(cos_i))
    jiao_jun = atan2(sin(b) * sin(c), sin(a) * cos(b) - cos(a) * sin(b) * cos(c))
    jiao_shi = norm(jiao - jiao_jun) if two < 180 else norm(jiao + jiao_jun)   # 次引倍度不及半周为减
    ju = norm(bai - jiao_shi)
    hd = atan2(cos(da_ju) * sin(ju), cos(ju))
    huang = norm(bai + signed(hd - ju))
    return huang, s_shi, dict(chu_jun=chu_signed, er_jun=e2, san_jun=e3, da_ju=da_ju, jiao_jun=jiao_jun)


def jieqi_times(y):
    out = []
    dz, _ = dongzhi(y)
    d = math.floor(dz) - 2
    lam_d = sun(d)[3]
    for k in range(25):
        target = (k * 15) % 360
        while True:
            lam_n = sun(d + 1)[3]
            a = signed(lam_d - target)
            b = signed(lam_n - target)
            if a < 0 <= b:
                out.append(d + (-a) / (b - a))
                break
            d, lam_d = d + 1, lam_n
    return out


def shuo_times(d0, d1, yongshi=True):
    out = []

    def el(d):
        mo, su, _ = moon(d, yongshi)
        return signed(mo - su)
    e_d = el(d0)
    for d in range(d0, d1):
        e_n = el(d + 1)
        if e_d < 0 <= e_n:
            out.append(d + (-e_d) / (e_n - e_d))
        e_d = e_n
    return out


def sui_months(y, yongshi=True):
    """冬至岁 y：(朔日 JDN, 月序, 闰否)，自天正至次年天正前。用时：合朔由用时太阴平行所得，中气化用时；平时：皆不化。"""
    q0, q1 = jieqi_times(y), jieqi_times(y + 1)
    conv = (lambda t: t + shi_cha(t)) if yongshi else (lambda t: t)
    zhong = [math.floor(conv(q0[k])) for k in range(0, 24, 2)] + [math.floor(conv(q1[0]))]
    sh = shuo_times(math.floor(q0[0]) - 32, math.floor(q1[0]) + 32, yongshi)
    days = [math.floor(t) for t in sh]
    i0 = max(i for i, dd in enumerate(days) if dd <= zhong[0])
    i1 = max(i for i, dd in enumerate(days) if dd <= zhong[12])
    ms = days[i0:i1 + 1]
    n = i1 - i0
    has = [any(ms[i] <= z < ms[i + 1] for z in zhong) for i in range(n)]
    leap_at = next((i for i in range(1, n) if not has[i]), None) if n == 13 else None
    out, mo = [], 11
    for i in range(n):
        if i == leap_at:
            out.append((jdn_of(ms[i]), out[-1][1], True))
            continue
        out.append((jdn_of(ms[i]), mo, False))
        mo = mo % 12 + 1
    return out


GZ = [("甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12]) for i in range(60)]


def ymd(j):
    a = j + 32044
    b = (4 * a + 3) // 146097
    c = a - 146097 * b // 4
    d = (4 * c + 3) // 1461
    e = c - 1461 * d // 4
    m = (5 * e + 2) // 153
    return f"{100 * b + d - 4800 + m // 10}-{m + 3 - 12 * (m // 10):02d}-{e - (153 * m + 2) // 5 + 1:02d}"


def self_check():
    out = []
    dz, _ = dongzhi(YUAN_YEAR)
    out.append(("康熙甲子天正冬至", f"{ymd(jdn_of(math.floor(dz)))} {GZ[math.floor(dz) % 60]} {(dz % 1) * 24:.3f} 时", "气应 7.656374926：辛未"))
    mx = max(abs(sun_jun(a / 10)) for a in range(3600))
    out.append(("日躔最大均数", f"{int(mx)}°{(mx % 1) * 60:.2f}′", "志未直书（卷四十七只给两心差 358416）"))
    # 两弦（次引 90°）、自行三宫（引数 90°）时初均 + 二均（卷四十七「極大均數七度二十五分四十六秒」）
    m, th = 90.0, 90.0
    h = MOON_JUNLUN
    small, big = 3 * h, R
    chu = atan2(small, big)
    e2, _ = er_jun(m, -chu, chu, math.hypot(small, big), th)
    tot = chu + abs(e2)
    out.append(("两弦、自行三宫：初均 + 二均", f"{int(tot)}°{(tot % 1) * 60:.2f}′", "7°25.77′（七度二十五分四十六秒）"))
    mj = max(abs(atan2(sin(DA_JU_BANJIAO) * sin(c / 10), sin(DA_JU_ZHONG) * cos(DA_JU_BANJIAO) - cos(DA_JU_ZHONG) * sin(DA_JU_BANJIAO) * cos(c / 10))) for c in range(1800))
    out.append(("最大交均", f"{mj * 60:.2f}′", "志未直书"))
    return out


if __name__ == "__main__":
    import csv
    from pathlib import Path
    root = Path(__file__).resolve().parents[2]
    for name, got, want in self_check():
        print(f"{name}: {got}（{want}）")
    zh = {}
    for r in csv.DictReader(open(root / "vendor/calendar/chinese_calendar/实历朔日表.csv", encoding="utf-8")):
        zh[jdn(int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"]))] = (int(r["lunar_month"]), r["is_leap"] == "1")
    y0, y1 = 1724, 1760
    path = root / "data/calendar/shixian_kaocheng_check.csv"
    n = ny = npg = 0
    with open(path, "w", encoding="utf-8", newline="") as fh:
        fh.write("# 《历象考成》（康熙甲子元法）复原（scripts/calendar_data/shixian_kaocheng.py）逐月朔日与张表比较：冬至岁 1724—1760。"
                 "用时 = 太阴平行化至用时子正、中气化用时；平时 = 不化。雍正四年（1726）起行用，至《后编》起用止。\n")
        w = csv.writer(fh)
        w.writerow(["冬至岁", "月", "闰", "用时朔日", "平时朔日", "用时合张表", "平时合张表"])
        for y in range(y0, y1 + 1):
            for (ja, m, l), (jb, mb, lb) in zip(sui_months(y, True), sui_months(y, False)):
                a, b = zh.get(ja) == (m, l), zh.get(jb) == (mb, lb)
                n += 1
                ny += a
                npg += b
                w.writerow([y, m, int(l), ymd(ja), ymd(jb), int(a), int(b)])
    print(path.relative_to(root), f"{n} 月：用时合张表 {ny}，平时合张表 {npg}")
