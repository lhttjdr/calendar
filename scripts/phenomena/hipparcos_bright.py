"""从 Hipparcos 原始星表提取肉眼亮星表 data/stars/hipparcos_bright.csv。

- 天体测量（ICRS 位置、历元 J1991.25；视差；自行）取新归算 I/311（van Leeuwen 2007），vendor/hipparcos/I_311/hip2.dat.gz；
- Johnson V 与 B−V 取原版主表 I/239（ESA 1997），vendor/hipparcos/I_239/hip_main.dat（新归算只有 Hp 星等）。
- 收录 V ≤ 6.5（I/239 Vmag）且新归算有解的星；另收 vendor/stellarium_skycultures/ 中国星官引用的全部 HIP 星（含暗于 6.5 等者），
  使星官数据都能定位。

用法（仓库根）：uv run --project scripts python scripts/phenomena/hipparcos_bright.py
"""

import glob
import gzip
import json
import math

V_LIMIT = 6.5
HIP2 = "vendor/hipparcos/I_311/hip2.dat.gz"
MAIN = "vendor/hipparcos/I_239/hip_main.dat"
OUT = "data/stars/hipparcos_bright.csv"


def fnum(s):
    s = s.strip()
    return float(s) if s else None


def referenced_hips():
    """中国星官（Stellarium 各体系）连线中引用的 HIP。"""
    out = set()
    for f in glob.glob("vendor/stellarium_skycultures/*/index.json"):
        d = json.load(open(f, encoding="utf-8"))
        for c in d["constellations"]:
            for line in c["lines"]:
                out.update(h for h in line if isinstance(h, int) and h < 10**7)
    return out


def main():
    extra = referenced_hips()
    photo = {}
    with open(MAIN, encoding="ascii") as f:
        for line in f:
            hip = int(line[8:14])
            v = fnum(line[41:46])
            if v is None or (v > V_LIMIT and hip not in extra):
                continue
            photo[hip] = (v, fnum(line[245:251]), line)
    rows = []
    with gzip.open(HIP2, "rt", encoding="ascii") as f:
        for line in f:
            hip = int(line[0:6])
            if hip not in photo:
                continue
            v, bv, _ = photo.pop(hip)
            ra, de = float(line[15:28]), float(line[29:42])
            plx, pmra, pmde = float(line[43:50]), float(line[51:59]), float(line[60:68])
            rows.append((hip, math.degrees(ra), math.degrees(de), plx, pmra, pmde, v, bv, "I/311"))
    # 新归算无解而被星官引用者：用原版 I/239 的天体测量（历元同为 J1991.25，ICRS）
    for hip in sorted(set(photo) & extra):
        v, bv, line = photo.pop(hip)
        ra, de = fnum(line[51:63]), fnum(line[64:76])
        plx, pmra, pmde = fnum(line[79:86]) or 0.0, fnum(line[87:95]) or 0.0, fnum(line[96:104]) or 0.0
        src = "I/239"
        if ra is None or de is None:
            # 无天体测量解（双星、聚星）：用 H3、H4 的历元 J1991.25 位置（时分秒、度分秒），自行、视差记 0
            h, m, sec = (float(x) for x in line[17:28].split())
            sign = -1.0 if line[29] == "-" else 1.0
            d, dm, ds = (float(x) for x in line[30:40].split())
            ra, de = 15.0 * (h + m / 60 + sec / 3600), sign * (d + dm / 60 + ds / 3600)
            src = "I/239-H3H4"
        rows.append((hip, ra, de, plx, pmra, pmde, v, bv, src))
    rows.sort()
    with open(OUT, "w", encoding="utf-8") as f:
        f.write("# Hipparcos 亮星（V <= %.1f，另含中国星官引用的较暗星）。位置 ICRS 历元 J1991.25、视差、自行：I/311（van Leeuwen 2007）；V、B-V：I/239（ESA 1997）。\n" % V_LIMIT)
        f.write("# 生成：scripts/phenomena/hipparcos_bright.py。pmra 为 mu_alpha*cos(delta)。B-V 缺失为空。astrometry：I/311 新归算；无解者用 I/239 原版，原版亦无解者用其 H3、H4 位置（无自行，I/239-H3H4）。\n")
        f.write("hip,ra_deg,dec_deg,plx_mas,pmra_mas_yr,pmdec_mas_yr,vmag,bv,astrometry\n")
        for r in rows:
            bv = "" if r[7] is None else "%.3f" % r[7]
            f.write("%d,%.9f,%.9f,%.2f,%.2f,%.2f,%.2f,%s,%s\n" % (r[:7] + (bv, r[8])))
    print(f"{len(rows)} 颗；I/311 中缺解 {len(photo)} 颗：{sorted(photo)[:20]}；星官引用而缺失：{sorted(extra - {r[0] for r in rows})}")


if __name__ == "__main__":
    main()
