"""《新法算书》月离「五轮图」（卷二十九月离历指卷二）作图与卷三十四二三均数表逐格比较。

原文（卷二十九，四库本 CADAL06054903 第 21 页目验）：「如圖甲為地心，取甲乙線為半徑，乙為心，甲為界，作甲丁丙圈。
從圈周任取丁為心，作戊己癸圈，其半徑丁戊是為月與地之平距……若丁戊為全數十萬，即甲乙為二千一百七十分。
又於戊己癸周任取癸點為心，取癸辛線五千八百分為半徑，作午辛辰本輪；又取辛庚線二千九百分為半徑，作庚壬子均輪……
凡月之定朔定望時，丁心與地心甲合為一點，丁心右旋，循甲丙丁圈，半月而周，則甲丙丁周上之弧為月距太陽之倍數。
本輪之癸心循戊癸未圈右旋二十七日有竒而周，均輪庚子之心辛循本輪周左旋，亦二十七日有竒而周……
月體則循均輪周右旋，十三日有竒而周。」卷三十：二均、三均合為「實均數」之外加減；卷三十四表以月距日、實自行
（「初自行以初均加減訖」）為引數。

原书目验（2026-11-11，CADAL06054903 第 21 页，Commons 缩略图 960px）：「若丁戊為全數十萬，即甲乙為二千一百七十分，
右為二三一」——原文即 2170，维基文库录文之缺系录入所缺；图中乙在地心甲之上，朝最高戊。

模型（十万为单位，角度以度）：月 = 甲丁 + 1e5·e(L) + 5800·e(L−α) + 2900·e(L+180°+α)，α 为平自行（自最高），
由表引数实自行 s 反求（s = α + 第一均数）；甲丁 = R1·(e(A+180°+2η) − e(A+180°))，A = L − α 为平最高，
丁绕乙右旋月距日倍度、朔望时与地心重合；二三均数 = 有、无甲丁两视黄经之差，另加月角差 V·sin2η。
- 原文参数：R1 = 2170，V = 0。
- 造表实用（反推）：R1 = 2174（第谷 Progymnasmata 值），V = 40′30″ = 2430″（第谷月角差 variatio；新法算书各卷未见明文，
  卷二十八言「又次輪……微眇難分……無暇及也」）。

用法（仓库根）：python3 scripts/calendar_data/xinfa_wulun.py
"""
import csv
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
R = 1e5
B1, B2 = 5800.0, 2900.0


def e(deg):
    return complex(math.cos(math.radians(deg)), math.sin(math.radians(deg)))


def arg(z):
    return math.degrees(math.atan2(z.imag, z.real))


def wrap(d):
    return (d + 180) % 360 - 180


def first(alpha):
    """第一均数（度，加为正）：atan(−8700 sinα / (R + 2900 cosα))"""
    z = R + B1 * e(-alpha) + B2 * e(180 + alpha)
    return arg(z)


def alpha_from_shi(s):
    """实自行 s = α + 第一均数 → 反求平自行 α"""
    a = s
    for _ in range(50):
        a = s - first(a)
    return a


def ersan(eta, shi, r1, phi_kind, sense):
    a = alpha_from_shi(shi)
    L = 0.0
    A = L - a
    phi0 = {"A": A, "A+180": A + 180, "L": L, "L+180": L + 180}[phi_kind]
    d = r1 * (e(phi0 + sense * 2 * eta) - e(phi0))
    m0 = R * e(L) + B1 * e(L - a) + B2 * e(L + 180 + a)
    return wrap(arg(d + m0) - arg(m0))


def load():
    out = []
    for f in ("er_san_jun_part1.csv", "er_san_jun_part2.csv"):
        p = ROOT / "data/calendar/xinfa_yueli_tables" / f
        if not p.exists():
            continue
        for r in csv.DictReader(l for l in open(p, encoding="utf-8") if not l.startswith("#")):
            if r["存疑"] == "1" or r["分"] == "":
                continue
            eta = int(r["月距日_宫"]) * 30 + int(r["月距日_度"])
            shi = int(r["实自行_宫"]) * 30 + int(r["实自行_度"])
            has_sec = r["秒"] != ""
            v = int(r["分"]) * 60 + (int(r["秒"]) if has_sec else 0)
            v = v if r["号"] == "加" else -v
            out.append((eta, shi, v, has_sec))
    return out


def stats(cells, fn):
    res = []
    for eta, shi, v, has_sec in cells:
        c = fn(eta, shi) * 3600
        tol = 0.5 if has_sec else 30
        res.append((eta, shi, v, c, abs(c - v) - tol))
    rms = math.sqrt(sum((r[3] - r[2]) ** 2 for r in res) / len(res))
    bad = [r for r in res if r[4] > 1.5]
    return rms, max(abs(r[3] - r[2]) for r in res), bad


def model(eta, shi, r1, vamp):
    a = alpha_from_shi(shi)
    A = -a
    m0 = R * e(0) + B1 * e(-a) + B2 * e(180 + a)
    d = r1 * (e(A + 180 + 2 * eta) - e(A + 180))
    return wrap(arg(m0 + d) - arg(m0)) * 3600 + vamp * math.sin(math.radians(2 * eta))


def report(cells, r1, vamp, label):
    sec = [c for c in cells if c[3]]
    res = [(eta, shi, v, v - model(eta, shi, r1, vamp)) for eta, shi, v, h in sec]
    rms = math.sqrt(sum(r[3] ** 2 for r in res) / len(res))
    mx = max(abs(r[3]) for r in res)
    big = [r for r in res if abs(r[3]) > 2]
    # 分格（月距日 16 度起，只到分）：容差 ±30″
    mn = [c for c in cells if not c[3]]
    mres = [abs(v - model(eta, shi, r1, vamp)) for eta, shi, v, h in mn]
    over = sum(x > 30 for x in mres)
    print(f"{label}：R1={r1} V={vamp}″ 带秒 {len(res)} 格 均方根 {rms:.2f}″ 最大 {mx:.2f}″ 超 2″ {len(big)}；"
          f"度分格 {len(mn)} 格 超 ±30″ {over}")
    return big


if __name__ == "__main__":
    cells = load()
    report(cells, 2170.0, 0.0, "原文参数（无月角差）")
    report(cells, 2170.0, 2430.0, "原文 R1 + 月角差")
    big = report(cells, 2174.0, 2430.0, "第谷 R1 + 月角差")
    print("超 2″ 之格（月距日, 实自行, 表, 残差）：", [(b[0], b[1], b[2], round(b[3], 1)) for b in big])
