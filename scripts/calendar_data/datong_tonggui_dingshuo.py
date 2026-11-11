"""《大统暦日通轨》立成查表定朔：照《通轨》步气朔术文与卷上、卷下立成推大统历 1368—1644 年各月定朔，与公式推步（shoushi_dingshuo.py）、张表比较。

底本：明抄本《大统暦日通轨》扫描（hanshu_cache/netdisk_tianwen/10642342_大统暦日通轨.pdf，不入库），
pymupdf 页序 2—5 步气朔术文，6—15 太阳盈缩立成，15—38 太阴迟疾度立成。转录见
data/calendar/datong_tonggui_taiyang_licheng.csv、data/calendar/datong_tonggui_taiyin_licheng.csv。

术文（页序）：
- 求各年前十一月中积（2 左）：置岁实 3652425 分，以至元辛巳积年减一乘之（抄本「辛」讹作「年」）；
- 求闰余（3 右）：中积加闰应 202050，满朔策 295305.93 去之；求冬至：中积加气应 550600；求经朔：冬至减闰余，累加朔策；
- 求盈缩历（3 左）：置半岁周 1826212.5 减闰余，为天正经朔入缩历；累加朔策，满半岁周去之，盈缩互易。
  冬至后为盈初、春分后为盈末、夏至后为缩初、秋分后为缩末；盈初缩末限 88.909225 日，缩初盈末限 93.712025 日。
- 求迟疾历（4 右）：中积加转应 130205，减闰余，满转终 275546 去之；不满小转中 137773 为疾历，满之去之为迟历。
- 求盈缩差（4 左）：盈初缩末用冬至前后二象，缩初盈末用夏至前后二象；去大余，以小余全分乘其下盈缩加分，以万约为分，
  加其下盈缩积，以万约为度。
- 求迟疾差（5 右）：置迟疾历日及分，用迟疾历日率减之，余以其下损益分乘之，「得数以八百二十而一」，益加损减其下迟疾度。
- 求加减差（5 右）：盈迟同名、缩疾同名，二度相并；异名相消；「余以八百二十分乘之，得数又以其下迟疾[限行]度分而一」。
  盈迟为加，缩疾为减。
- 求定朔（5 左）：经朔以加减差加减之，命甲子算外；弦望小余在日出分以下者退一日（朔不进不退）。

立成与术文的对应（逐格见两个 CSV 的「比較」列）：
- 太阴立成「损益分」已是每日分（1/10000 日）之秒数（初限 1.3514116 秒 × 820 = 1108.1575 秒 = 一限积度），
  「疾/迟历限行度」已是 820 ÷ 月每限实行度（初限 1.2071 度 → 679.3140，抄本以 6.793140 记）。故术文「以八百二十而一」
  「以八百二十分乘之、以其下迟疾度分而一」两步已并入立成，查表时只须：迟疾差（秒）＝积度 ± 损益分 × 余分；
  加减差（日分）＝（盈缩差 ± 迟疾差）（度）× 限行度 × 100。若再照字面除 820，迟疾差将小三个数量级，显然不合。
- 迟疾历日率按 转终 ÷ 336 ＝ 820.0774 分一限累计（一十三限 1.0661、一百六十八限 13.7773＝转中），
  不是术文「以十二限二十乘之」的 1/12.2 日；公式推步用 x＝入历日×12.2。
- 迟疾积度八十二至八十五限不依三次式：损益分取八十一限之 2/3、1/3 递减，极值在八十四限（5.42934424 度），
  前后对称；三次式在 82.5 限前后即转降。
- 限行度：月每限行度取 1.0963 ± 该限积度差（四位），或 1.0962375 ± 积度差 四舍五入至四位，再以 820 除之；两法抄本互见，末位常差一二。

截分（术文未言者为本脚本假定）：
- exact：全用分数，不截。
- 截分：盈缩差、迟疾差保留至表的末位（盈缩 1/10000 分、迟疾 1/10000 秒）；加减差截至秒（1/100 日分）；
  经朔、入转、盈缩历用分秒原数（朔策 295305.93 本有秒）。
- 盈初缩末的「末」限、缩初盈末的「末」限，查表日数取 半岁周 − 盈缩历（术文只言「交」，未明言反减；立成按「前后二象」
  对称排列，只能如此用）。
- 抄本明显抄误之格（CSV「校改」列：衍文、字形不辨、与对称格互校可定者）缺省用校改值，--raw 用原读数（不辨之格仍取公式值）。

结果（截分与不截同；立成取校改值）：
- 1368—1644 年与张表同日 3423/3426（公式推步 3421/3426）；--raw 3422（1460 年五月因一百四十七限遲曆抄误 8.680345 移后一时）。
- 弘治十年十月：疾历 47 限，立成日率（820.0774 分一限）与限行度阶梯使加减差比公式少减 3.2 分，定朔 己巳 00:00.3（公式 戊辰 23:59.9）。
- 万历三十八年二月：迟历 84 限，立成极值区（积度 5.42934 度、每限行 1.0965 度；公式 5.42384、1.09154）使加减差少 21.8 分，
  定朔 丁未 23:59.2（公式 戊申 00:02.4）。二者皆合《明实录》与张表。
- 洪武三年二月、十一年八月：立成与公式同日（辛酉 00:03.6、庚子 23:35.5），合实录，张表误。
- 弘治八年七月：经朔 壬午 13:54.4，缩初 39.0 日、疾历 3.70 日，加减差 −4018 分（−9.64 时），定朔 壬午 04:15.8，与公式同；
  得辛巳须再减 1776 分（4.26 时），而此入转下大统加减差最大只 13.27 时（需 13.92 时），任何立成、截分都到不了辛巳；
  1368—1644 年定朔在 0—4:48 时之月 738 个，张表退一日者只此月与洪武三年二月，非系统规则。差在经朔或颁历本身，非立成。
- 立成与公式定朔逐月差：中位 1.9 分，最大 29 分（多在近极值八十二至八十五限与限行度阶梯处）。

用法（仓库根）：
  uv run --project scripts python scripts/calendar_data/datong_tonggui_dingshuo.py            # 五月比较、全段统计、1495 年七月中间量
  uv run --project scripts python scripts/calendar_data/datong_tonggui_dingshuo.py --tables   # 重算两表的「公式」「比較」列
  uv run --project scripts python scripts/calendar_data/datong_tonggui_dingshuo.py --raw      # 立成用原读数
  uv run --project scripts python scripts/calendar_data/datong_tonggui_dingshuo.py --export   # 写推步用表 datong_tonggui_licheng_used.csv（Rust 读）
"""
import csv
import math
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import discriminating_records as d  # noqa: E402
import gaiyuan_date as g  # noqa: E402
import shoushi_dingshuo as ss  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
MOON_CSV = ROOT / "data/calendar/datong_tonggui_taiyin_licheng.csv"
SUN_CSV = ROOT / "data/calendar/datong_tonggui_taiyang_licheng.csv"

F = 10000
SUI = 3652425
HALF = Fr(SUI, 2)                      # 半岁周 1826212.5 分
SHUO = Fr(29530593, 100)               # 朔策 295305.93 分
ZHUAN = 275546                         # 转终
ZHUAN_ZHONG = 137773                   # 小转中
QI_YING, RUN_YING, ZHUAN_YING = 550600, 202050, 130205
YING_XIAN = Fr(88909225, 100)
SUO_XIAN = Fr(93712025, 100)           # 缩初盈末限 93.712025 日（分）
PING_XIAN = Fr(1336875, 100000) * Fr(82, 1000)   # 月每限平行 1.0962375 度


# ------------------------------------------------------------------ 公式（三次式）对照值
def moon_cubic(x):
    """入迟疾历第 x 限之迟疾积度（度）：授时三次式，过 84 限按 168 − x 对称（同 shoushi_dingshuo.moon_diff）。"""
    if x > 84:
        x = 168 - x
    return Fr(11110000 - (28100 + 325 * x) * x) * x / 10 ** 8


def sun_cubic(x, winter):
    a, b, c = (5133200, 24600, 31) if winter else (4870600, 22100, 27)
    return (a - (b + c * x) * x) * x / Fr(10 ** 4)   # 分（1 度 = 10000 分）


# ------------------------------------------------------------------ 读表
def _num(s):
    return None if s in ("", None) or "?" in s or not any(ch.isdigit() for ch in s) else Fr(s)


def _fixes(text):
    out = {}
    for part in filter(None, text.split("；")):
        k, v = part.split("=", 1)
        out[k] = v.split("（")[0]
    return out


def read_rows(path):
    with open(path, encoding="utf-8") as fh:
        return list(csv.DictReader(line for line in fh if not line.startswith("#")))


def load_tables(raw=False):
    """返回 (moon, sun)。moon[n] = (日率分, 损益秒每分（带号）, 积度秒, 疾历列, 迟历列)；sun[(winter, 日)] = (加分, 积分)。"""
    moon = []
    for r in read_rows(MOON_CSV):
        n = int(r["限"])
        fx = {} if raw else _fixes(r["校改"])
        get = lambda col: _num(fx.get(col.split("（")[0].replace("遲疾曆", ""), r[col]))
        rl = get("遲疾曆日率（日）")
        if n == 168:
            moon.append((rl * F, None, None, None, None))
            continue
        sy, jd = get("損益分（秒/日分）"), get("遲疾積度（度）")
        dx = moon_cubic(n + 1) - moon_cubic(n)
        if sy is None:   # 不辨之格取公式
            sy = abs(dx) * F / 820
        if jd is None:
            jd = moon_cubic(n)
        sign = -1 if r["損益"] == "損" else 1
        ji, ch = get("疾曆限行度"), get("遲曆限行度")
        moon.append(((rl if rl is not None else Fr(n * ZHUAN, 336 * F)) * F, sign * sy, jd * F, ji, ch))
    sun = {}
    for r in read_rows(SUN_CSV):
        winter = r["表"].startswith("冬")
        fx = {} if raw else _fixes(r["校改"])
        day = int(r["積日"])
        a, j = _num(fx.get("盈縮加分", r["盈縮加分（分）"])), _num(fx.get("盈縮積", r["盈縮積（分）"]))
        if j is None:
            j = sun_cubic(day, winter)
        if a is None:
            a = sun_cubic(day + 1, winter) - sun_cubic(day, winter)
        sun[(winter, day)] = (a, j)
    return moon, sun


# ------------------------------------------------------------------ 逐格比较（写回 CSV 的公式、比较列）
def update_tables():
    rows = read_rows(MOON_CSV)
    for r in rows:
        n = int(r["限"])
        r["公式日率"] = f"{n * ZHUAN / 336 / F:.6f}"
        if n == 168:
            r["比較"] = "合（轉中）" if abs(float(r["遲疾曆日率（日）"]) - 13.7773) < 1e-9 else "異"
            continue
        dx = moon_cubic(n + 1) - moon_cubic(n)
        r["公式損益分"] = f"{float(abs(dx) * F / 820):.8f}"
        r["公式積度"] = f"{float(moon_cubic(n)):.8f}"
        r["公式疾曆限行度"] = f"{float(820 / (PING_XIAN + dx) / 100):.6f}"
        r["公式遲曆限行度"] = f"{float(820 / (PING_XIAN - dx) / 100):.6f}"
        notes = []
        rl = _num(r["遲疾曆日率（日）"])
        if rl is None or abs(rl - Fr(n * ZHUAN, 336 * F)) >= Fr(1, F):
            notes.append(f"日率{'不辨' if rl is None else f'差 {float(rl - Fr(n * ZHUAN, 336 * F)) * F:+.2f} 分'}")
        sgn = "損" if dx < 0 else "益"
        if r["損益"] and r["損益"] != sgn:
            notes.append(f"損益號作{r['損益']}（三次式{sgn}）")
        sy = _num(r["損益分（秒/日分）"])
        if sy is None:
            notes.append("損益分不辨")
        elif abs(sy - abs(dx) * F / 820) > Fr(2, 10 ** 6):
            notes.append(f"損益分差 {float(sy - abs(dx) * F / 820):+.7f} 秒")
        jd = _num(r["遲疾積度（度）"])
        if abs(jd - moon_cubic(n)) > Fr(1, 10 ** 8):
            notes.append(f"積度差 {float(jd - moon_cubic(n)) * F:+.4f} 秒")
        for col, fcol in (("疾曆限行度", "公式疾曆限行度"), ("遲曆限行度", "公式遲曆限行度")):
            v = _num(r[col])
            rel = abs(v / Fr(r[fcol]) - 1)
            if rel > Fr(1, 10 ** 4):
                notes.append(f"{col[:2]}差 {float(v - Fr(r[fcol])):+.6f}")
        if 82 <= n <= 85:
            notes.append("近極值處立成不依三次式：損益取八十一限之 2/3、1/3")
        r["比較"] = "；".join(notes) if notes else "合"
    _write(MOON_CSV, rows, [
        "# 《大统暦日通轨》卷下「太陰遲疾度立成」（明抄本扫描页序 15—38）逐限转录。日率：日（1 日 = 10000 分）；",
        "# 損益分：每日分之秒（1 度 = 100 分 = 10000 秒），抄本「秒」多作「秋」；積度：度；疾/遲曆限行度：抄本记数，×100 = 820 ÷ 月每限行度（度）。",
        "# 校改：抄误可定之格（缺省用于推步）；公式：三次式 (11110000 − (28100 + 325x)x)x/1e8 度（过 84 限对称），",
        "# 日率 = 限 × 轉終/336，限行度 = 820 ÷ (1.0962375 ± 該限積度差)；比較：損益分容差 2e-6 秒、積度 1e-8 度、限行度相對 1e-4。",
        "# 由 scripts/calendar_data/datong_tonggui_dingshuo.py --tables 重算公式、比較列。",
    ])
    rows = read_rows(SUN_CSV)
    for r in rows:
        winter = r["表"].startswith("冬")
        day = int(r["積日"])
        fa, fj = sun_cubic(day + 1, winter) - sun_cubic(day, winter), sun_cubic(day, winter)
        r["公式加分"], r["公式積"] = f"{float(fa):.4f}", f"{float(fj):.4f}"
        notes = []
        a, j = _num(r["盈縮加分（分）"]), _num(r["盈縮積（分）"])
        if r["盈縮加分（分）"] == "空":
            notes.append("加分空")
        elif a is None:
            notes.append("加分不辨")
        elif abs(a - fa) > Fr(1, F):
            notes.append(f"加分差 {float(a - fa):+.4f}")
        if j is None:
            notes.append("積不辨")
        elif abs(j - fj) > Fr(1, F):
            notes.append(f"積差 {float(j - fj):+.4f}")
        r["比較"] = "；".join(notes) if notes else "合"
    _write(SUN_CSV, rows, [
        "# 《大统暦日通轨》卷上太陽「冬至前後二象盈初縮末限」「夏至前後二象縮初盈末限」（页序 6—15）逐日转录。",
        "# 單位：分（1 度 = 10000 分，即術文「以萬約為度」）。末行（冬 89 日、夏 94 日）只列積。",
        "# 公式：盈初縮末 (5133200 − (24600 + 31x)x)x/1e4，縮初盈末 (4870600 − (22100 + 27x)x)x/1e4；比較容差 1e-4 分。",
        "# 由 scripts/calendar_data/datong_tonggui_dingshuo.py --tables 重算公式、比較列。",
    ])
    for path in (MOON_CSV, SUN_CSV):
        rs = read_rows(path)
        bad = [r for r in rs if r["比較"] not in ("合", "合（轉中）")]
        print(f"{path.name}：{len(rs)} 行，合 {len(rs) - len(bad)}，異 {len(bad)}")
        for r in bad:
            print("  ", r.get("限") or (r["表"][:2] + r["積日"]), r["比較"])


def _write(path, rows, comments):
    with open(path, "w", encoding="utf-8", newline="") as fh:
        for c in comments:
            fh.write(c + "\n")
        w = csv.DictWriter(fh, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)


# ------------------------------------------------------------------ 推步
def trunc(x, unit):
    """向零截至 unit 的整数倍。"""
    q = x / unit
    return (math.floor(q) if q >= 0 else -math.floor(-q)) * unit


class Tonggui:
    def __init__(self, moon, sun, cut=True):
        self.moon, self.sun, self.cut = moon, sun, cut
        self.rilv = [m[0] for m in moon]

    def ying_suo(self, li, ying):
        """盈缩历 li（分，0—半岁周），ying：盈历否。返回 (盈缩差度（盈正缩负）, 说明)。"""
        if ying:
            first, x = li < YING_XIAN, li
            winter = first                    # 盈初：冬至前后；盈末：夏至前后
        else:
            first, x = li < SUO_XIAN, li
            winter = not first                # 缩初：夏至前后；缩末：冬至前后
        if not first:
            x = HALF - li
        day = int(x // F)
        frac = x - day * F                    # 小余全分
        a, j = self.sun[(winter, day)]
        part = a * frac / F
        if self.cut:
            part = trunc(part, Fr(1, F))
        deg = (j + part) / F
        name = ("盈" if ying else "缩") + ("初" if first else "末")
        return (deg if ying else -deg), f"{name} 查{'冬' if winter else '夏'}至表 {float(x / F):.4f} 日（{day} 日 + {float(frac):.2f} 分）"

    def chi_ji(self, li, fast):
        """迟疾历 li（分，0—小转中）。返回 (迟疾差度（疾正迟负）, 限行度列值, 说明)。"""
        n = max(k for k in range(168) if self.rilv[k] <= li)
        rl, sy, jd, ji, ch = self.moon[n]
        yu = li - rl
        sec = jd + sy * yu                    # 秒
        if self.cut:
            sec = trunc(sec, Fr(1, F))
        col = ji if fast else ch
        deg = sec / F
        return (deg if fast else -deg), col, f"{'疾' if fast else '迟'}历 {float(li / F):.4f} 日 → {n} 限（日率 {float(rl / F):.4f}，余 {float(yu):.2f} 分），限行度 {float(col):.6f}"

    def month(self, zj, m, verbose=False):
        run = (zj + RUN_YING) % SHUO
        dz = zj + QI_YING
        jing = dz - run + m * SHUO
        li_ys = HALF - run + m * SHUO        # 天正经朔入缩历，累加朔策
        ying = False
        while li_ys >= HALF:
            li_ys -= HALF
            ying = not ying
        zz = (zj + ZHUAN_YING - run + m * SHUO) % ZHUAN
        fast = zz < ZHUAN_ZHONG
        li_cj = zz if fast else zz - ZHUAN_ZHONG
        ys, s1 = self.ying_suo(li_ys, ying)
        cj, col, s2 = self.chi_ji(li_cj, fast)
        diff = ys - cj                        # 盈、迟为加；缩、疾为减
        jjc = diff * col * 100
        if self.cut:
            jjc = trunc(jjc, Fr(1, 100))
        ding = jing + jjc
        info = dict(经朔=jing, 入转=zz, 盈缩历=li_ys, 盈缩=s1, 盈缩差=ys, 迟疾=s2, 迟疾差=cj, 限行度=col, 加减差=jjc, 定朔=ding)
        return ding, info

    def moons(self, y0, y1):
        out = []
        for y in range(y0, y1):
            zj = (y - 1281) * SUI
            for m in range(14):
                out.append(self.month(zj, m)[0])
        return sorted(set(out))


# ------------------------------------------------------------------ 比较
def fmt(t, off):
    j = int(t // F) + off
    h = float((t % F) / F * 24)
    return j, f"{g.ganzhi_of_jdn(j)} {int(h):02d}:{(h - int(h)) * 60:04.1f}"


def zhang_stats(moons, off, y0=1368, y1=1644):
    by = {v: k for k, v in d.tables()["张表"].items()}
    same = n = 0
    diff = []
    for t in moons:
        j = int(t // F) + off
        near_ = [k for k in range(j - 2, j + 3) if k in by and y0 <= by[k][0] <= y1]
        if not near_:
            continue
        k = min(near_, key=lambda x: abs(x - j))
        n += 1
        if k == j:
            same += 1
        else:
            diff.append((by[k], k, t))
    return same, n, diff


CASES = [(1370, 2, "辛酉"), (1378, 8, "庚子"), (1495, 7, "辛巳"), (1497, 10, "己巳"), (1610, 2, "丁未")]


USED_CSV = ROOT / "data/calendar/datong_tonggui_licheng_used.csv"


def export_used():
    """推步实用之表（校改后、不辨之格取公式），分数 p/q 记，供 rust/core/src/calendar/dingshuo.rs 内嵌。"""
    moon, sun = load_tables(False)
    fr = lambda x: "" if x is None else (str(x.numerator) if x.denominator == 1 else f"{x.numerator}/{x.denominator}")
    with USED_CSV.open("w", encoding="utf-8", newline="") as fo:
        fo.write("# 《大统暦日通轨》立成推步实用值（scripts/calendar_data/datong_tonggui_dingshuo.py --export 生成，勿手改）：\n")
        fo.write("# 太阴：限,日率（分）,损益（秒/日分，损为负）,积度（秒）,疾历列,迟历列；太阳：表（冬/夏）,积日,加分（分）,积（分）。\n")
        w = csv.writer(fo)
        w.writerow(["类", "a", "b", "c", "d", "e", "f"])
        for n, (rl, sy, jd, ji, ch) in enumerate(moon):
            w.writerow(["太阴", n, fr(rl), fr(sy), fr(jd), fr(ji), fr(ch)])
        for (winter, day), (a, j) in sorted(sun.items(), key=lambda kv: (not kv[0][0], kv[0][1])):
            w.writerow(["太阳", "冬" if winter else "夏", day, fr(a), fr(j), "", ""])
    print(USED_CSV)


def main():
    if "--export" in sys.argv:
        export_used()
        return
    raw = "--raw" in sys.argv
    if "--tables" in sys.argv:
        update_tables()
        return
    moon, sun = load_tables(raw)
    off = ss.c.align_offset(QI_YING // F, g.jdn(1280, 12, 14))
    zhang = d.tables()["张表"]
    formula = ss.Shoushi("大统", RUN_YING, ZHUAN_YING).moons(1367, 1646)
    variants = {"立成·截分": Tonggui(moon, sun, True).moons(1367, 1646),
                "立成·不截": Tonggui(moon, sun, False).moons(1367, 1646)}
    print("立成取" + ("原读数" if raw else "校改后之数"))
    print("== 五月（《明实录》月朔为准）")
    for y, mo, shilu in CASES:
        mt = zhang[(y, mo, False)] if (y, mo, False) in zhang else None
        row = [f"{y} 年 {mo} 月 实录 {shilu} 张表 {g.ganzhi_of_jdn(mt)}"]
        for name, ms in [("公式", formula)] + list(variants.items()):
            t = min(ms, key=lambda x: abs(int(x // F) + off - mt))
            row.append(f"{name} {fmt(t, off)[1]}")
        print("  " + "；".join(row))
    print("== 1368—1644 年逐月朔日与张表")
    for name, ms in [("公式", formula)] + list(variants.items()):
        same, n, diff = zhang_stats(ms, off)
        print(f"  {name}：同日 {same}/{n}")
        for (ly, lm, lp), k, t in diff:
            print(f"     {ly} 年{'闰' if lp else ''}{lm} 月 张表 {g.ganzhi_of_jdn(k)} {name} {fmt(t, off)[1]}")
    print("== 1495 年七月逐步中间量（经朔、入转、入历、盈缩差、迟疾差、加减差）")
    tg = Tonggui(moon, sun, True)
    zj = (1495 - 1281) * SUI
    mt = zhang[(1495, 7, False)]
    m = min(range(14), key=lambda k: abs(int(tg.month(zj, k)[0] // F) + off - mt))
    _, info = tg.month(zj, m)
    jing = info["经朔"]
    print(f"  中积 {zj} 分，天正经朔后第 {m} 月")
    print(f"  经朔 {float(jing):.2f} 分 → {fmt(jing, off)[1]}")
    print(f"  入转 {float(info['入转']):.2f} 分（{float(info['入转'] / F):.4f} 日）；{info['迟疾']}")
    print(f"  盈缩历 {float(info['盈缩历']):.2f} 分；{info['盈缩']}")
    print(f"  盈缩差 {float(info['盈缩差']):+.8f} 度；迟疾差（疾正迟负）{float(info['迟疾差']):+.8f} 度")
    print(f"  加减差 {float(info['加减差']):+.2f} 分（{float(info['加减差']) / F * 24:+.3f} 时）；定朔 {fmt(info['定朔'], off)[1]}")
    t = jing - (zj + QI_YING)
    sd, md = ss.sun_diff(t), ss.moon_diff(info["入转"])
    v = ss.moon_speed(info["入转"]) * Fr(82, 1000)
    fdeg = sd - md
    print(f"  公式：盈缩差 {float(sd):+.8f} 度；迟疾差 {float(md):+.8f} 度；月每限行 {float(v):.6f} 度（立成列 → {float(820 / (info['限行度'] * 100)):.6f}）；"
          f"加减差 {float(fdeg / v * 820):+.2f} 分；定朔 {fmt(jing + fdeg / v * 820, off)[1]}")


if __name__ == "__main__":
    main()
