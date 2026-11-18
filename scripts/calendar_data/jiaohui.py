"""唐宋诸历步交会（入交定、入食限）与日食判定的复原，Rust 移植（rust/core/src/calendar/dingshuo.rs 之 JiaoConst、Jiaohui、
Dayan::xuanming_rishi、Song::chongtian_rishi、Song::rishi）的逐月比对基准。

术（唐宋诸历同）：经朔入交泛 = 朔积分 mod 交终（统天先减交差 80298）；「以其日入氣朓朒定數，朓減、朒加交汎，為入交常」；
「以交率乘其日入轉朓朒定數，如交數而一，以朓減、朒加入交常，為入交定」；入交定在交中已下为阳历，已上去之为阴历；
去交（距最近交点）不过望差（宋诸历「後限」，宣明「後準」）为入食限。常数出处见 JIAO 注。

日食（排月「有交不进」所用）：
- 宣明（《新唐书》卷三十上）：时差、气差、刻差、加差，去交定分 < 8700（阳历蚀限 2640 + 阴历蚀限 6060）者蚀；阴历不足减入阳历不蚀，
  阳历不足减入阴历蚀。日出用大衍步轨漏（宣明晷漏未复原，近似）。
- 崇天（《宋史》卷七十三）：时差、气差、刻差，日入食限、食定分、汎用分、定用分（转定分由月离损益率折算）。
- 余宋诸历：入食限为有食；食甚以时差加减（观天卷七十八，纪元卷八十，南宋诸历用纪元之数）；定用按观天、纪元汎用分术，
  去交分代交前后分（近似）。

检验：rust/core/examples/jiaohui_eclipse_check.rs（史载日食）、jiaohui_options_eval.rs（排月各选项）。
输出：data/calendar/jiaohui_reference.csv（每历隔 3 岁，冬至岁自天正经朔起 14 月的入交定分、日食判定）。
用法（仓库根）：uv run --project scripts python scripts/calendar_data/jiaohui.py
"""
import csv
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import dayan_dingshuo as dy  # noqa: E402
import song_dingshuo as sg  # noqa: E402
import wuji_zhengyuan_dingshuo as wz  # noqa: E402
import xuanming_dingshuo as xm  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
STEP = 3

# (交终分, 望差/后限分, 交率, 交数, 交差)，秒母一万
JIAO = {
    # 《新唐书》卷二十八下：終數 827251322；望差一日四百八十三、秒九千三百三十九；「以交數去朔積分」之「交數」从校勘记作「終數」
    "大衍": (Fr(827251322, 10000), Fr(35239339, 10000), 343, 4369, 0),
    # 卷二十九：五紀交終三億六千四百六十四萬三千七百六十七；望差一日二百一十三、秒三千一百一十六半
    "五纪": (Fr(364643767, 10000), Fr(155331165, 100000), 61, 777, 0),
    # 正元交終分二億九千七百九十七萬三千八百一十五；望差一日百七十四、秒三千九十二半（「交會悉如五紀法」）
    "正元": (Fr(297973815, 10000), Fr(126930925, 100000), 61, 777, 0),
    # 卷三十上：終率二十二萬八千五百八十二、秒六千五百一十二；後準一日千三百三十七、秒千七百四十四
    "宣明": (Fr(2285826512, 10000), Fr(97371744, 10000), 202, 2573, 0),
    # 卷三十下：交終分三十六萬七千三百六十四、秒九千六百七十三；望差一日二千一百四十九、秒百六十三半
    "崇玄": (Fr(3673649673, 10000), Fr(1564901635, 100000), 262, 3350, 0),
    # 《宋史》卷七十三：交終分二十八萬八千一百七十七、秒四千二百七十七；後限＝朔差之半
    "崇天": (Fr(2881774277, 10000), Fr(245515723, 20000), 141, 1796, 0),
    # 卷七十八：志作「三十二萬七千三百六十一、秒九千九百四十四」（交终日、交中日、朔差与之相应，点校本无校记），交点月 27.212136 日
    # 独小于他历（27.21221—27.21223）；以「六十一」作「六十二」则 27.212219 日，与史载日食五条全合（作六十一则无一合）。
    # 本项目据史载日食校改，非版本依据；朔差、后限随之
    "观天": (Fr(3273629944, 10000), Fr(278900056, 20000), 183, 2331, 0),
    # 卷八十
    "纪元": (Fr(1983770880, 10000), Fr(169009120, 20000), 324, 4127, 0),
    # 卷八十三（统元正文，乾道、淳熙、会元注文）。会元志不载交数，以交率 : 交数 ≈ 朔差 : 朔实推得 6458
    "统元": (Fr(1885806457, 10000), Fr(160663543, 20000), 42, 535, 0),
    "乾道": (Fr(8163666034, 10000), Fr(695511566, 20000), 80, 1019, 0),
    "淳熙": (Fr(1534769543, 10000), Fr(130756057, 20000), 61, 777, 0),
    "会元": (Fr(10531132140, 10000), Fr(897207860, 20000), 507, 6458, 0),
    # 卷八十四（统天正文，开禧、成天注文）；统天交差八萬二百九十八
    "统天": (Fr(326547), Fr(27821, 2), 19, 242, 80298),
    "开禧": (Fr(4598864825, 10000), Fr(391805175, 20000), 19, 242, 0),
    "成天": (Fr(2019147051, 10000), Fr(172022949, 20000), 19, 242, 0),
}


def jiaohui(name, jing, sun, moon, f):
    """→ (入交定分, 去交分, 阴历, 交前, 入食限)"""
    zhong, xian, lv, shu, cha = JIAO[name]
    ru = (((jing - cha) % zhong) + sun + moon * Fr(lv, shu)) % zhong
    half = zhong / 2
    yin = ru >= half
    x = ru - half if yin else ru
    qian = x * 2 > half
    qu = half - x if qian else x
    return ru, qu, yin, qian, qu <= xian


# ---------------------------------------------------------------- 大衍步轨漏（宣明日出代用），同 dingshuo.rs dayan_night_lou
DY_LOU = [(27, 230), (27, 135), (26, 380), (25, 475), (24, 470), (23, 360), (22, 240), (21, 120), (20, 10), (19, 5), (18, 100), (17, 345), (17, 250)]
DY_SHUAI = [64, 1191, 2242, 3025, 3578, 3950, 3965, 3889, 3356, 2838, 2012, 1012, 52, 1076, 2075, 2890, 3455, 3890, 3966, 3950, 3498, 2972, 2170, 1113]
DY_ZHIJIANG = [78, 72, 53, 34, 78, 1, -5, -1, -32, -52, -63, -64, 64, 63, 52, 32, 99, 5, -1, -1, -34, -53, -72, -78]
DY_XIAN = {4: [-12, -8, -3, -2, -1], 7: [1, 2, 3, 8, 19], 16: [-19, -8, -3, -2, -1], 19: [1, 2, 3, 8, 12]}


def dayan_lou(i):
    return Fr(DY_LOU[i][0] * 480 + DY_LOU[i][1], 480)


def dayan_night_lou(kk, n):
    base = dayan_lou(kk if kk <= 12 else 24 - kk)
    xi = kk < 12
    xian = DY_XIAN.get(kk)
    shuai, rate, sign, acc = DY_SHUAI[kk], abs(DY_ZHIJIANG[kk]), (DY_ZHIJIANG[kk] > 0) - (DY_ZHIJIANG[kk] < 0), 0
    for j in range(n):
        acc += shuai
        if xian and j >= 1:
            rate = max(rate + xian[min((j - 1) // 3, 4)], 0)
        shuai += sign * rate
    delta = Fr(acc, 100 * 480)
    return base - delta if xi else base + delta


def floor(x):
    return x.numerator // x.denominator if isinstance(x, Fr) else x // 1


def xm_tong(y):
    return xm.SUI * (xm.JI_822 + (y - 822))


def xm_seg(t):
    k = floor(t / xm.QI) - 1
    while xm.ding_qi(k + 1) <= t:
        k += 1
    return k


def xm_sunrise(y, t):
    """定朔 t 所在日之日出（占一日之比）：夜半漏 + 昏明二刻半"""
    f, tong = xm.F, xm_tong(y)
    day = floor(t / f)
    d = (day * f - tong) % xm.SUI
    k = xm_seg(Fr(d))
    qi_day = floor((tong + xm.ding_qi(k) + xm.SUI * floor(Fr(day * f - tong, xm.SUI))) / f)
    lou = dayan_night_lou(k % 24, max(day - qi_day, 0))
    return (lou + Fr(5, 2)) / 100


def xm_month(y, m):
    tong = xm_tong(y)
    t0 = tong - tong % xm.SHUO
    jing = t0 + m * xm.SHUO
    sun, moon = xm.sun_def(Fr(jing - tong)), xm.moon_def((t0 % xm.LI_ZHOU + m * xm.SHUO) % xm.LI_ZHOU)
    return jing, sun, moon, jing + sun + moon


def xuanming_rishi(y, m):
    """→ (蚀, 阴历, 食甚之比)。见 dingshuo.rs Dayan::xuanming_rishi。"""
    f = xm.F
    jing, sun, moon, ding = xm_month(y, m)
    _, qu, yin, _, _ = jiaohui("宣明", jing, sun, moon, f)
    frac = (ding % f) / f
    sr = xm_sunrise(y, ding)
    half = Fr(1, 2)
    h = (half - sr) * 100
    shicha = Fr(147) / h
    shen = frac - (half - frac) * shicha / 84 if frac <= half else frac + 2 * (frac - half) * shicha / 84
    ju_wu = abs(shen - half) * 100
    sui = Fr(xm.SUI, f)
    d = Fr(ding - xm_tong(y), f) % sui
    l_ = sui / 2
    x = d % l_
    dz = x if x <= l_ / 2 else l_ - x
    ys = 1 if yin else -1
    qi = max(2350 - 26 * dz, Fr(0))
    qi_d = qi - qi * ju_wu / h
    chun = sui / 4 <= d < sui * Fr(3, 4)
    s_qi = ys * (1 if chun else -1) * qi_d
    ke = min(Fr(21, 10) * dz, Fr(189, 2))
    am = shen < half
    dzh = d < l_
    s_ke = (-ys if dzh == am else ys) * ke * ju_wu
    k = floor(d / (sui / 24)) % 24
    jia = {22: 17, 2: 17, 23: 34, 1: 34, 0: 51}.get(k, 0)
    s_jia = 0 if am else -ys * jia * (shen - half) * 100
    ding_fen = qu + s_qi + s_ke + s_jia
    if ding_fen < 0:
        return (not yin), (not yin), shen
    return ding_fen < 8700, yin, shen


# ---------------------------------------------------------------- 宋诸历
SONG = {"崇天": sg.CHONGTIAN, "观天": sg.GUANTIAN, "纪元": sg.JIYUAN, "统元": sg.TONGYUAN, "乾道": sg.QIANDAO, "淳熙": sg.CHUNXI,
        "会元": sg.HUIYUAN, "统天": sg.TONGTIAN, "开禧": sg.KAIXI, "成天": sg.CHENGTIAN}
SONG_SPAN = {"崇天": (1024, 1074), "观天": (1094, 1102), "纪元": (1106, 1166), "统元": (1136, 1167), "乾道": (1168, 1176),
             "淳熙": (1177, 1190), "会元": (1191, 1198), "统天": (1199, 1207), "开禧": (1208, 1251), "成天": (1271, 1276)}


def song_month(li, y, m):
    tong, t0 = li.tong(y), li.first_shuo(y)
    base = (t0 - li.cha["zhuan"]) % li.zhuan if li.cha else t0 % li.zhuan
    jing = t0 + m * li.shuo
    sun, moon = li.sun_def(Fr(jing - tong)), li.moon_def((base + m * li.shuo) % li.zhuan)
    return jing, sun, moon, jing + sun + moon, base


def song_sunrise(li, y, t):
    f = li.f
    noon = (floor(Fr(t) / f) + Fr(1, 2)) * f
    d = next((noon - li.tong(yy) for yy in range(y - 1, y + 3) if 0 <= noon - li.tong(yy) < li.sui), (noon - li.tong(y)) % li.sui)
    return li.sunrise(Fr(d) / f + li.yingsuo_deg(Fr(d)))


def chongtian_rishi(y, m):
    """→ (蚀, 阴历, 食甚之比, 定用之比)。见 dingshuo.rs Song::chongtian_rishi。"""
    li = sg.CHONGTIAN
    f = li.f
    jing, sun, moon, ding, base = song_month(li, y, m)
    ru, qu, yin, qian, _ = jiaohui("崇天", jing, sun, moon, f)
    xiao = ding % f
    banfa = Fr(f, 2)
    if xiao <= banfa:
        p = banfa - xiao
        shicha, pm = (banfa - p) * p / 31770, False
    else:
        p = xiao - banfa
        shicha, pm = (banfa - p) * p / 13885, True
    shen = xiao + shicha if pm else xiao - shicha
    juwu = p + shicha
    sr = song_sunrise(li, y, ding) * f
    banzhou = banfa - sr
    zhongji = Fr((ding - li.tong(y)) % li.sui, f)
    erzhi = Fr(18262, 100)
    x = zhongji % erzhi
    u = x if x <= erzhi / 2 else erzhi - x
    jiaozhong = yin != qian

    def jz(v):
        return v if jiaozhong else -v
    qicha = 3533 - u * u * 100 / 236
    qi_d = qicha - qicha * juwu / banzhou
    sui = Fr(li.sui, f)
    d_qi = jz(qi_d) if sui / 4 <= zhongji < sui * Fr(3, 4) else -jz(qi_d)
    kecha = (erzhi - x) * x * 100 / 236
    ke_d = kecha * juwu * 4 / f
    d_ke = -jz(ke_d) if (zhongji < erzhi) != pm else jz(ke_d)
    d_shi = shicha if pm else -shicha
    zhong, xian = JIAO["崇天"][0], JIAO["崇天"][1]
    zhongri = zhong / 2
    xr = (ru + d_qi + d_ke + d_shi) % zhong
    fen = None
    if xr >= zhongri:
        w = xr - zhongri
        if w <= xian:
            fen = w
        elif w >= zhongri - xian:
            fen = zhongri - w
    if fen is not None and fen <= 4200:
        shi, yang, a = True, True, fen / 100
    elif fen is not None and fen < 11200:
        shi, yang, a = True, False, (11200 - fen) / 100
    else:
        shi, yang, a = False, False, Fr(0)
    fanyong = (84 - a) * a * 100 / 185 if yang else (140 - a) * a * 100 / 514
    x_r = ((base % li.zhuan) + m * li.shuo) % li.zhuan
    rate = li.moon_def((x_r + banfa) % li.zhuan) - li.moon_def((x_r - banfa + li.zhuan) % li.zhuan)
    zhuanding = 1337 - rate * Fr(1237) / f
    return shi, shi, shen / f, fanyong * 1337 / zhuanding / f


def song_rishi(name, y, m):
    """余宋诸历：→ (入食限, 阴历, 食甚之比, 定用之比)。见 dingshuo.rs Song::rishi。"""
    li = SONG[name]
    f = li.f
    jing, sun, moon, ding, _ = song_month(li, y, m)
    _, qu, yin, _, shixian = jiaohui(name, jing, sun, moon, f)
    frac = (ding % f) / f
    a, b = {"观天": (Fr(36090, 12030), Fr(18045, 12030))}.get(name, (Fr(10935, 7290), Fr(1)))
    half = Fr(1, 2)
    if frac <= half:
        x = half - frac
        shen = frac - (half - x) * x / a
    else:
        x = frac - half
        shen = frac + (half - x) * x / b
    if name == "观天":
        if qu <= 4900:
            aa, k, dd = qu / 100, 98, 250
        else:
            aa, k, dd = max(12800 - qu, Fr(0)) / 100, 158, 650
        dy_ = (k - aa) * aa / dd / 100
    else:
        uu = qu * 7290 / f
        dy_ = max((583 - uu * uu / 100 / (317 if yin else 198)) / 7290, Fr(0))
    return shixian, yin, shen, dy_


def frac(x):
    x = Fr(x)
    return f"{x.numerator}/{x.denominator}"


def rows():
    # 唐：(名, 日法, 年段, (y, m) → (经朔, 日躔, 月离))
    tang = []

    def dayan(y, m):
        ji = dy.JI_724 + (y - 724)
        zhong = dy.CE * ji
        t0 = zhong - zhong % dy.SHE
        jing = t0 + m * dy.SHE
        return jing, dy.sun_def(Fr(jing - zhong)), dy.moon_def(Fr((t0 * dy.MIAO + m * dy.SHE * dy.MIAO) % dy.ZHUAN, dy.MIAO))
    tang.append(("大衍", dy.F, (729, 761), dayan))
    for li, span in ((wz.WUJI, (762, 783)), (wz.ZHENGYUAN, (784, 806))):
        def f(y, m, li=li):
            tong = li.tong(y)
            t0 = tong - tong % li.she
            jing = t0 + m * li.she
            return jing, li.sun_def(Fr(jing - tong)), li.moon_def(Fr((t0 * li.miao + m * li.she * li.miao) % li.zhuan, li.miao))
        tang.append((li.name, li.f, span, f))
    tang.append(("宣明", xm.F, (822, 956), lambda y, m: xm_month(y, m)[:3]))
    for name, f, (y0, y1), fn in tang:
        for y in range(y0, y1 + 1, STEP):
            for m in range(14):
                jing, sun, moon = fn(y, m)
                ru = jiaohui(name, jing, sun, moon, f)[0]
                rs = xuanming_rishi(y, m) if name == "宣明" else None
                yield [name, y, m, frac(ru)] + ([int(rs[0]), int(rs[1]), frac(rs[2]), ""] if rs else ["", "", "", ""])
    for name, li in SONG.items():
        y0, y1 = SONG_SPAN[name]
        for y in range(y0, y1 + 1, STEP):
            for m in range(14):
                jing, sun, moon, _, _ = song_month(li, y, m)
                ru = jiaohui(name, jing, sun, moon, li.f)[0]
                rs = chongtian_rishi(y, m) if name == "崇天" else song_rishi(name, y, m)
                yield [name, y, m, frac(ru), int(rs[0]), int(rs[1]), frac(rs[2]), frac(rs[3])]


def main():
    with open(ROOT / "data/calendar/jiaohui_reference.csv", "w", encoding="utf-8", newline="") as fh:
        fh.write("# 唐宋诸历入交定与日食判定（scripts/calendar_data/jiaohui.py 生成，勿手改）：每历隔 3 岁取一冬至岁，自天正经朔起第 m 月；"
                 "入交定为日法之分（自交初起）；蚀、阴历、食甚（占一日之比）、定用（同）只宣明、宋诸历有。\n")
        w = csv.writer(fh)
        w.writerow(["历", "冬至岁", "m", "入交定分", "蚀", "阴历", "食甚", "定用"])
        n = 0
        for r in rows():
            w.writerow(r)
            n += 1
        print("jiaohui_reference.csv", n, "行")


if __name__ == "__main__":
    main()
