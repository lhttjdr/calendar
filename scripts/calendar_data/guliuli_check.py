"""古六历：以《开元占经》卷一百五所载积年核验上游参数（push-algorithms.md 5.1、ancient_restore.rs）。

古六历皆四分术：岁实 365¼，朔策 29 + 499/940；上元为甲子日夜半朔旦冬至。积年 N（至开元二年 714 算外）之年，
天正冬至 = 上元日 + ⌊N × 1461/4⌋ 日，天正朔 = 上元日 + ⌊k × 27759/940⌋。上元日序 0 为甲子，JDN 由干支取最近真冬至者。
与上游的冬至历元 jdEpoch、合朔历元 jdEpoch_lunar 比较，看两者是否同一推步（差为整日则同、差为分数则冬至或朔的相位不同）。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/guliuli_check.py
"""
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gaiyuan_date as g  # noqa: E402

SOLAR, LUNAR = Fr(1461, 4), Fr(27759, 940)
# 历：(占经积年至 714 算外, 占经上元干支, 上游 yEpoch, 上游 jdEpoch, 上游 jdEpoch_lunar)
U = {  # 积年读自《开元占经》卷一百五（页 1029）
    "黄帝": (2760863, "辛卯", 170, Fr(17835105, 10), None),
    "颛顼": (2761019, "乙卯", 14, None, Fr(17265755, 10)),
    "夏": (2760589, "乙丑", 444, Fr(18835905, 10), None),
    "殷": (2761080, "甲寅", -47, Fr(17042505, 10), None),
    "周": (2761130, "丁巳", -104, Fr(16834305, 10), None),
    "鲁": (2761334, "庚子", -481, Fr(15457305, 10), None),
}


def upstream(name):
    n, gz, ye, je, jl = U[name]
    if name == "颛顼":
        je = jl - SOLAR / 8
    if jl is None:
        jl = je - LUNAR / 19 if name == "鲁" else je
    return ye, je, jl


def check(name, n714):
    _, gz, *_ = U[name]
    ye, je, jl = upstream(name)
    y_yuan = 714 - n714
    gz_ok = (y_yuan - 4) % 60 == g.GANZHI.index(gz)
    ev = SOLAR / 8 if name == "颛顼" else 0          # 颛顼历元首为立春，余五历为冬至
    # 上游：年 y 的「天正冬至」= je + (y − ye − 1)·solar 记在 y−1 年末；取 y = ye + 1 的那一个
    w_up = je + ev
    n = n714 - (714 - (ye + 1))                      # 该冬至（或立春）所属之岁的积年
    d = n * SOLAR                                    # 占经：自上元日之日数
    jd0 = w_up - d                                   # 隐含的上元日 JD
    jd0_int = round(jd0 - Fr(1, 2)) + Fr(1, 2)       # 取最近之夜半
    phase = float(jd0 - jd0_int)
    jiazi = (int(jd0_int + Fr(1, 2)) + 49) % 60 == 0
    k = (jl - jd0_int) / LUNAR                       # 上游朔历元距上元的朔数
    shuo = float((k - round(k)) * LUNAR)
    return gz_ok, phase, jiazi, shuo


def main():
    rows = []
    for name, (n714, gz, *_r) in U.items():
        cands = [n714] if name != "周" else [n714, n714 + 7]
        for n in cands:
            gz_ok, phase, jiazi, shuo = check(name, n)
            rows.append((name, n, gz, gz_ok, phase, jiazi, shuo))
            print(f"{name} 积年 {n}：上元干支{'合' if gz_ok else '不合'}{gz}；冬至/立春相位差 {phase:+.4f} 日；上元日{'是' if jiazi else '非'}甲子；朔相位差 {shuo:+.4f} 日")


if __name__ == "__main__":
    main()
