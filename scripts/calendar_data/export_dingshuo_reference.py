"""导出唐至明定朔诸历复原（本目录 *_dingshuo.py）的定朔，作 Rust 移植（rust/core/src/calendar/dingshuo.rs）的逐月比对基准。

每历在行用年内每隔 STEP 岁取一个冬至岁，导出自天正经朔起 14 个月的定朔（自上元甲子夜半起算的「分」，精确分数）、
日法、上元日序 → JDN 之差（与 dingshuo_check.compare 所用相同）。宋诸历另导出由维基文库底本解析、并经校改的表
（data/calendar/dingshuo_song_tables.csv），供 Rust 读入。

输出：data/calendar/dingshuo_reference.csv、data/calendar/dingshuo_song_tables.csv
用法（仓库根）：uv run --project scripts python scripts/calendar_data/export_dingshuo_reference.py
"""
import csv
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import chongxuan_dingshuo as cx  # noqa: E402
import dayan_dingshuo as dy  # noqa: E402
import dingshuo_check as c  # noqa: E402
import gaiyuan_date as g  # noqa: E402
import linde_dingshuo as ld  # noqa: E402
import shoushi_dingshuo as ss  # noqa: E402
import song_dingshuo as sg  # noqa: E402
import wuji_zhengyuan_dingshuo as wz  # noqa: E402
import wuyin_dingshuo as wy  # noqa: E402
import xuanming_dingshuo as xm  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
STEP = 3


def frac(x):
    x = Fr(x)
    return f"{x.numerator}/{x.denominator}"


def tang():
    """(历名, 日法, off, 起岁, 止岁, 冬至岁 → 14 个定朔分)"""
    out = []
    # 戊寅：脚本以「日」为单位，换成日法分
    z = Fr(wy.JI_626 * wy.SUI, wy.QIFA)
    off = c.align_offset(int(z), g.jdn(625, 12, 20))
    out.append(("戊寅定朔", wy.RI, off, 627, 645, lambda y: [d * wy.RI for _, d in wy.year_moons(y)]))
    out.append(("戊寅平朔", wy.RI, off, 646, 664, lambda y: [p * wy.RI for p, _ in wy.year_moons(y)]))
    out.append(("麟德", ld.F, ld.epoch_offset(), 665, 728, lambda y: [d for _, d in ld.year_moons(y)]))
    dz = dy.CE * dy.JI_724 // dy.F
    out.append(("大衍", dy.F, c.align_offset(dz, g.jdn(723, 12, 21)), 729, 761, lambda y: [d for _, d in dy.year_moons(y)]))
    for li, y0, y1 in ((wz.WUJI, 762, 783), (wz.ZHENGYUAN, 784, 806)):
        dz = li.tong(li.y_ji) // li.f
        li.off = c.align_offset(dz, g.jdn(*li.approx))

        def f(y, li=li):
            tong = li.tong(y)
            t0 = tong - tong % li.she
            return [t0 + m * li.she + li.sun_def(Fr(t0 + m * li.she - tong))
                    + li.moon_def(Fr((t0 * li.miao + m * li.she * li.miao) % li.zhuan, li.miao)) for m in range(14)]
        out.append((li.name, li.f, li.off, y0, y1, f))
    dz = xm.SUI * xm.JI_822 // xm.F
    out.append(("宣明", xm.F, c.align_offset(dz, g.jdn(821, 12, 19)), 822, 892, lambda y: [d for _, d in xm.year_moons(y)]))
    dz = cx.SUI * cx.JI_892 // cx.F
    out.append(("崇玄", cx.F, c.align_offset(dz, g.jdn(891, 12, 17)), 893, 955, lambda y: [d for _, d in cx.year_moons(y)]))
    return out


def song():
    out = []
    for li, y0, y1, approx in ((sg.CHONGTIAN, 1024, 1074, (1023, 12, 18)), (sg.MINGTIAN, 1065, 1067, (1063, 12, 16)),
                               (sg.GUANTIAN, 1094, 1102, (1091, 12, 16)), (sg.JIYUAN, 1106, 1166, (1105, 12, 16)),
                               (sg.TONGYUAN, 1136, 1167, (1134, 12, 15)), (sg.QIANDAO, 1168, 1176, (1166, 12, 15)),
                               (sg.CHUNXI, 1177, 1190, (1175, 12, 15)), (sg.HUIYUAN, 1191, 1198, (1190, 12, 15)),
                               (sg.TONGTIAN, 1199, 1207, (1198, 12, 15)), (sg.KAIXI, 1208, 1251, (1206, 12, 15)),
                               (sg.CHENGTIAN, 1271, 1276, (1270, 12, 14))):
        dz = li.tong(li.y_ji) // li.f
        li.off = c.align_offset(dz + li.day0, g.jdn(*approx)) + li.day0

        def f(y, li=li):
            if isinstance(li, sg.Mingtian):
                tong = li.tong(y)
                jy = tong // li.shuo
                return [(jy + m) * li.shuo + li.sun_def(Fr((jy + m) * li.shuo - tong)) + li.moon_sign * li.moon_of(jy + m) for m in range(14)]
            tong, t0 = li.tong(y), li.first_shuo(y)
            base = (t0 - li.cha["zhuan"]) % li.zhuan if li.cha else t0 % li.zhuan
            return [t0 + m * li.shuo + li.sun_def(Fr(t0 + m * li.shuo - tong)) + li.moon_def((base + m * li.shuo) % li.zhuan) for m in range(14)]
        out.append((li.name, li.f, li.off, y0, y1, f))
    return out


def shoushi():
    out = []
    dz = ss.QI_YING
    off = c.align_offset(dz // ss.F, g.jdn(1280, 12, 14))
    for name, li, y0, y1 in (("授时", ss.Shoushi("授时", 202050, 130205, False), 1281, 1367),
                             ("大统", ss.Shoushi("大统", 202050, 130205, True), 1368, 1644)):
        def f(y, li=li):
            zj = (y - 1281) * ss.SUI
            dzz = zj + ss.QI_YING
            run = (zj + li.run_ying) % ss.SHUO
            t0 = dzz - run
            ru0 = (zj + li.zhuan_ying - run) % ss.ZHUAN
            res = []
            for m in range(14):
                jing = t0 + m * ss.SHUO
                deg = ss.sun_diff(jing - dzz) - ss.moon_diff(ru0 + m * ss.SHUO)
                v = ss.moon_speed(ru0 + m * ss.SHUO) * Fr(82, 1000) - (Fr(82, 1000) if li.relative else 0)
                res.append(jing + deg / v * 820)
            return res
        out.append((name, ss.F, off, y0, y1, f))
    return out


def song_tables():
    rows = []
    for li in (sg.CHONGTIAN, sg.JIYUAN, sg.TONGYUAN, sg.QIANDAO, sg.CHUNXI, sg.HUIYUAN, sg.TONGTIAN, sg.KAIXI, sg.CHENGTIAN, sg.GUANTIAN):
        if li.sjp:
            for k, (ji, sign, a, cha) in enumerate(li.sjp):
                rows.append((li.name, "日躔", k, ji, sign, frac(a), frac(cha)))
        for i in range(28):
            if i in li.split:
                rows.append((li.name, "月离分段", i, li.mji[i], frac(li.inits[i]), li.split[i][0], li.split[i][1]))
            else:
                rows.append((li.name, "月离", i, li.mji[i], li.moon[i], "", ""))
    return rows


def main():
    with open(ROOT / "data/calendar/dingshuo_reference.csv", "w", encoding="utf-8", newline="") as fh:
        fh.write("# 唐至明定朔诸历复原的定朔（scripts/calendar_data/export_dingshuo_reference.py 生成，勿手改）：每历隔 3 岁取一冬至岁，"
                 "自天正经朔起第 m 月的定朔，单位为日法分，自上元甲子夜半起算；off 为上元日序 → JDN 之差。\n")
        w = csv.writer(fh)
        w.writerow(["历", "日法", "off", "冬至岁", "m", "定朔分"])
        n = 0
        for name, f, off, y0, y1, fn in tang() + song() + shoushi():
            for y in range(y0, y1 + 1, STEP):
                for m, v in enumerate(fn(y)):
                    w.writerow([name, f, off, y, m, frac(v)])
                    n += 1
        print("dingshuo_reference.csv", n, "行")
    with open(ROOT / "data/calendar/dingshuo_song_tables.csv", "w", encoding="utf-8", newline="") as fh:
        fh.write("# 宋诸历日躔、月离表（scripts/calendar_data/export_dingshuo_reference.py 由 song_dingshuo.py 解析维基文库底本并校改后导出，勿手改）。"
                 "日躔：气序 k、朏朒积、符号、定初率、日差；月离：日序、朏朒积、损益率（朒 +）；月离分段：日序、朏朒积、初数、初率、末率。\n")
        w = csv.writer(fh)
        w.writerow(["历", "类", "序", "a", "b", "c", "d"])
        rows = song_tables()
        w.writerows(rows)
        print("dingshuo_song_tables.csv", len(rows), "行")


if __name__ == "__main__":
    main()
