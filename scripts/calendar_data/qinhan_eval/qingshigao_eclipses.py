"""《清史稿·天文志》卷三十七日食（顺治元年至乾隆六十年，「某年某月某朔某時，日食某分，次於某宿」）→
data/phenomena/qingshigao_solar_eclipses.csv，供 rust/core/examples/historical_eclipses.rs 在北京计算所见食况。

列同 data/phenomena/houhanshu_solar_eclipses.csv，另加：
- shi：所记时辰（「午」）；
- fen：所记食分（分，直径的十分之一）：少、半、太为 1/4、1/2、3/4 分，强、弱按 ±1/12 分；「既」记 10。

这些食分、时辰多为钦天监按时宪历预推上报之数（卷首：「其見於本紀，無食分及所次宿，備以入誌」），比较所得为
时宪历推步的精度，不是观测精度。日期按实历朔日表（清时宪历）换算，并核对朔干支。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/qinhan_eval/qingshigao_eclipses.py
"""
import csv
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
sys.path.insert(0, str(HERE))
import gaiyuan_date as g  # noqa: E402
import houhanshu_star_records as hh  # noqa: E402

ROOT = HERE.parents[2]
OUT = ROOT / "data/phenomena/qingshigao_solar_eclipses.csv"
ERAS = {"順治": 1644, "康熙": 1662, "雍正": 1723, "乾隆": 1736}
NUM = "(?:元|[一二三四五六七八九十]+)"
MON = "正|十二|十一|十|[二三四五六七八九]"
REC = re.compile(rf"(?:(?P<era>順治|康熙|雍正|乾隆))?(?:(?P<yr>{NUM})年)?(?P<leap>閏)?(?P<mon>{MON})月(?P<gz>[甲乙丙丁戊己庚辛壬癸][子丑寅卯辰巳午未申酉戌亥])朔"
                 rf"(?P<shi>[子丑寅卯辰巳午未申酉戌亥])時，日食(?P<ext>[^，。]+)，次於(?P<lodge>[^。，]+)")
FRAC = {"少": 0.25, "半": 0.5, "太": 0.75}


def fen(ext):
    if ext == "既":
        return 10.0
    m = re.match(r"^([一二三四五六七八九十]+)分(半|少|太)?(強|弱)?$", ext)
    if not m:
        raise ValueError(ext)
    v = hh.cnnum(m.group(1)) + FRAC.get(m.group(2) or "", 0.0)
    return v + {"強": 1 / 12, "弱": -1 / 12}.get(m.group(3) or "", 0.0)


def main():
    t = hh.fetch("清史稿/卷37")
    i = t.index("其見於本紀")
    j = t.index("月五星淩犯掩距天官書")
    body = t[i:j]
    rows = []
    era = yr = None
    for m in REC.finditer(body):
        if m["era"]:
            era = m["era"]
        if m["yr"]:
            yr = hh.cnnum(m["yr"])
        year = ERAS[era] + yr - 1
        leap = bool(m["leap"])
        mon = 1 if m["mon"] == "正" else hh.cnnum(m["mon"])
        s0, _ = g.month_span(year, mon, leap)
        if g.ganzhi_of_jdn(s0) != m["gz"]:
            raise ValueError(f"{era}{yr}年{mon}月朔 {m['gz']} 与实历朔日表 {g.ganzhi_of_jdn(s0)} 不合")
        y0, m0, d0 = g.from_jdn(s0)
        rows.append(dict(regime="清", emp="", era=era, year_in_era=yr, year=year, month=mon, leap=int(leap), ganzhi=m["gz"], position="朔",
                         extent=m["ext"], lodge=m["lodge"], reported_by="", calendar="实历朔日表", jdn=s0,
                         julian=f"{y0}-{m0:02d}-{d0:02d}", lunar_day=1, note="", shi=m["shi"], fen=f"{fen(m['ext']):.2f}",
                         text=m.group(0)))
    with open(OUT, "w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)
    print(OUT, len(rows), "条（朔干支均与实历朔日表相合）")


if __name__ == "__main__":
    main()
