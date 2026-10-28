"""由逐条校验结果生成 doc/astronomy/phenomena-checks.md。

输入：
- data/phenomena/checks/*.csv：rust/core 各校验测试在 PHENOMENA_CHECKS=1 时写出（见 scripts/dev/phenomena_checks.sh）；
- data/phenomena/*_eclipses_check.csv：史书日月食检验（rust/core/examples/historical_eclipses.rs）。

每项一节：说明（取 CSV 的 # 注释行）、汇总（条数、指定列的最大绝对值/一致数）、逐条表格（全部行）。
用法（仓库根）：uv run --project scripts python scripts/phenomena/checks_to_md.py
"""

import csv
from pathlib import Path

CHECKS = Path("data/phenomena/checks")
OUT = Path("doc/astronomy/phenomena-checks.md")

# (文件, 标题, 汇总：[(列名, 汇总方式)]，汇总方式 "maxabs" 最大绝对值、"count:<值>" 计含该值的行数)
SECTIONS = [
    ("一、日月食（对现代星历表与日月食典）", None, None),
    ("nasa_solar_-0199_-0100", "NASA 五千年日食典，前 199—前 100", [("极大时刻差 s", "maxabs"), ("γ 差", "maxabs"), ("食分差", "maxabs")]),
    ("nasa_solar_2001_2100", "NASA 五千年日食典，2001—2100", [("极大时刻差 s", "maxabs"), ("γ 差", "maxabs"), ("食分差", "maxabs")]),
    ("nasa_lunar_-0199_-0100", "NASA 五千年月食典，前 199—前 100", [("极大时刻差 s", "maxabs"), ("γ 差", "maxabs"), ("半影食分差", "maxabs"), ("本影食分差", "maxabs")]),
    ("nasa_lunar_2001_2100", "NASA 五千年月食典，2001—2100", [("极大时刻差 s", "maxabs"), ("γ 差", "maxabs"), ("半影食分差", "maxabs"), ("本影食分差", "maxabs")]),
    ("二、行星", None, None),
    ("planet_events_vs_skyfield", "行星会合现象对 Skyfield", [("时刻差 s", "maxabs")]),
    ("ap_mag_v3_official", "行星视星等对 Ap_Mag V3 官方测试数据", [("差", "maxabs")]),
    ("planet_magnitude_vs_skyfield", "行星视星等对 Skyfield", [("差", "maxabs")]),
    ("三、近距事件与月掩星", None, None),
    ("close_approaches_vs_skyfield", "近距事件对 Skyfield", [("时刻差 s", "maxabs"), ("角距差″", "maxabs")]),
    ("close_approaches_vsop_elp_vs_de406", "近距事件：VSOP87 + ELPMPP02 对 DE406", [("时刻差 s", "maxabs"), ("角距差″", "maxabs")]),
    ("occultations_vs_skyfield", "月掩星对 Skyfield（北京）", [("入掩差 s", "maxabs"), ("出掩差 s", "maxabs")]),
    ("schaefer1992_telescope_model", "Schaefer 等 1992 望远镜目视月掩星极限星等", [("差", "maxabs")]),
    ("四、可见性模型（原文表格复现）", None, None),
    ("schaefer1993_table3_airmass", "Schaefer 1993 表 3 大气质量", None),
    ("ks1991_table2_moonlight", "Krisciunas & Schaefer 1991 表 2 月光亮度", [("差 nL", "maxabs")]),
    ("schaefer1990_eq18_zenith_limit", "Schaefer 1990 式 18 天顶极限星等", [("差", "maxabs")]),
    ("schaefer1998_worked_example", "Schaefer 1998 原文算例", None),
    ("schaefer1991_galilean", "Schaefer 1991 伽利略卫星（眩光）", None),
    ("schaefer1991_fig1_near_moon", "Schaefer 1991 图 1 紧靠月亮的极限星等", None),
    ("五、伏见（晨见夕伏）", None, None),
    ("schaefer1987_table1", "Schaefer 1987 表 1：45 次肉眼实测", "table1"),
    ("schaefer1987_table7_venus", "Schaefer 1987 表 7 条件下的金星伏见（尤卡坦 1977—1984）", None),
    ("schaefer2000_sirius", "Schaefer 2000 表 1：埃及天狼星偕日升", "sirius"),
    ("heliacal_babylon_2025_2026", "巴比伦 2025—2026 行星伏见（缺省模型）", None),
    ("六、恒星与二十八宿", None, None),
    ("star_apparent_vs_skyfield", "恒星视位置对 Skyfield", [("角距差″", "maxabs")]),
    ("pannai_table_9_1_4", "潘鼐表 9.1.4：《仪象考成》距星 1744 年位置", [("本模块−今测 RA″", "maxabs"), ("本模块−今测 Dec″", "maxabs")]),
    ("yuanshi_lodge_widths", "《元史》历代实测宿度", "widths"),
    ("yuanshi_winter_solstice", "《元史》历代冬至日躔", None),
    ("recorded_distances", "史书所记距离（尺、寸）与推算角距：一尺合多少度", None),
    ("lodge_degrees", "史书所记入宿度与推算入宿度（清时宪历为黄道 360°）", None),
    ("qingshigao_eclipses", "《清史稿》日食在北京：所记时辰、食分与推算", None),
    ("qingshigao_moon_fullday", "《清史稿》月与五星犯、距：全天地心最小角距（相距书为预推，不论昼夜）", None),
]


def read(name):
    p = Path(name[1:]) if name.startswith("@") else CHECKS / f"{name}.csv"
    if not p.exists():
        return None, [], []
    meta, body = [], []
    for line in p.read_text(encoding="utf-8").splitlines():
        (meta if line.startswith("#") else body).append(line)
    rows = list(csv.reader(body))
    return p, [m.lstrip("# ").strip() for m in meta], rows


def num(s):
    try:
        return float(s.replace("+", ""))
    except ValueError:
        return None


def summary(header, rows, spec):
    out = [f"共 {len(rows)} 条"]
    if spec == "table1":
        for i, h in enumerate(header):
            if h.endswith("当日可见"):
                ok = sum("✓" in r[i] for r in rows)
                clean = [r for r in rows if not any(x in r[8] for x in ("ZL", "MW", "??"))]
                okc = sum("✓" in r[i] for r in clean)
                out.append(f"{h[:-5]}：与实测一致 {ok}/{len(rows)}，去掉 ZL、MW、?? 后 {okc}/{len(clean)}")
        return out
    if spec == "sirius":
        for i, h in enumerate(header):
            if h.endswith("：差"):
                v = [num(r[i]) for r in rows if num(r[i]) is not None]
                out.append(f"{h[:-2]}：差 {min(v):+.1}—{max(v):+.1} 日，均值 {sum(v) / len(v):+.2}")
        return out
    if spec == "widths":
        for i, h in enumerate(header):
            if h == "差":
                v = [num(r[i]) for r in rows]
                rms = (sum(x * x for x in v) / len(v)) ** 0.5
                out.append(f"{header[i - 2].replace(' 史载', '')}：均方根 {rms:.2} 度，最大 {max(abs(x) for x in v):.2} 度")
        return out
    if spec == "records":
        def stat(name, vals):
            if vals:
                vals = sorted(vals)
                out.append(f"{name} {len(vals)} 条：≤1° {sum(x <= 1 for x in vals)}，≤2° {sum(x <= 2 for x in vals)}，≤5° {sum(x <= 5 for x in vals)}，"
                           f"中位 {vals[len(vals) // 2]:.2f}°")
        ai, pi, si = header.index("距星官最小角距°"), header.index("天体间最大角距°"), header.index("具体星角距°")
        stat("对星官（范围内最小角距）", [num(r[ai]) for r in rows if num(r[ai]) is not None])
        stat("天体之间（犯、合）", [num(r[pi]) for r in rows if num(r[pi]) is not None])
        stat("具体星", [num(r[si]) for r in rows if num(r[si]) is not None])
        out.append(f"干支不在所记之月 {sum(r[3] == '否' for r in rows)} 条")
        return out
    for col, how in spec or []:
        if col not in header:
            continue
        i = header.index(col)
        if how == "maxabs":
            v = [abs(num(r[i])) for r in rows if num(r[i]) is not None]
            if v:
                out.append(f"{col} 最大绝对值 {max(v):g}")
        elif how == "values":
            from collections import Counter

            c = Counter(r[i] for r in rows)
            out.append("；".join(f"{k} {n}" for k, n in c.most_common()))
    return out


# 史书天象记录：(文件, 史书, 类别, 年份列)。年份列为天文纪年整数列名，或「公历(儒略)」日期列（取年）。
RECORDS = [
    ("@data/phenomena/hanshu_solar_eclipses_check.csv", "《汉书·五行志》", "日食", "year"),
    ("@data/phenomena/houhanshu_solar_eclipses_check.csv", "《后汉书·五行志》", "日食", "year"),
    ("@data/phenomena/houhanshu_lunar_eclipses_check.csv", "《后汉书》", "月食", "year"),
    ("@data/phenomena/qingshigao_solar_eclipses_check.csv", "《清史稿·天文志》", "日食", "year"),
    ("@data/phenomena/jinshu_solar_eclipses_check.csv", "《晋书·天文志》", "日食", "year"),
    ("@data/phenomena/jinshu_lunar_eclipses_check.csv", "《晋书·天文志》", "月食", "year"),
    ("@data/phenomena/songshu_solar_eclipses_check.csv", "《宋书·五行志》", "日食", "year"),
    ("@data/phenomena/songshu_lunar_eclipses_check.csv", "《宋书·五行志》", "月食", "year"),
    ("@data/phenomena/nanqishu_solar_eclipses_check.csv", "《南齐书·天文志》", "日食", "year"),
    ("@data/phenomena/nanqishu_lunar_eclipses_check.csv", "《南齐书·天文志》", "月食", "year"),
    ("@data/phenomena/weishu_solar_eclipses_check.csv", "《魏书·天象志》", "日食", "year"),
    ("@data/phenomena/weishu_lunar_eclipses_check.csv", "《魏书·天象志》", "月食", "year"),
    ("@data/phenomena/suishu_solar_eclipses_check.csv", "《隋书·天文志》", "日食", "year"),
    ("@data/phenomena/suishu_lunar_eclipses_check.csv", "《隋书·天文志》", "月食", "year"),
    ("@data/phenomena/xintangshu_solar_eclipses_check.csv", "《新唐书·天文志》", "日食", "year"),
    ("@data/phenomena/xintangshu_lunar_eclipses_check.csv", "《新唐书·天文志》", "月食", "year"),
    ("@data/phenomena/jiutangshu_solar_eclipses_check.csv", "《旧唐书·天文志》", "日食", "year"),
    ("@data/phenomena/jiutangshu_lunar_eclipses_check.csv", "《旧唐书·天文志》", "月食", "year"),
    ("@data/phenomena/jiuwudaishi_solar_eclipses_check.csv", "《旧五代史·天文志》", "日食", "year"),
    ("@data/phenomena/jiuwudaishi_lunar_eclipses_check.csv", "《旧五代史·天文志》", "月食", "year"),
    ("@data/phenomena/xinwudaishi_solar_eclipses_check.csv", "《新五代史·司天考》", "日食", "year"),
    ("@data/phenomena/xinwudaishi_lunar_eclipses_check.csv", "《新五代史·司天考》", "月食", "year"),
    ("@data/phenomena/songshi_solar_eclipses_check.csv", "《宋史·天文志》", "日食", "year"),
    ("@data/phenomena/songshi_lunar_eclipses_check.csv", "《宋史·天文志》", "月食", "year"),
    ("@data/phenomena/jinshi_solar_eclipses_check.csv", "《金史·天文志》", "日食", "year"),
    ("@data/phenomena/jinshi_lunar_eclipses_check.csv", "《金史·天文志》", "月食", "year"),
    ("@data/phenomena/yuanshi_solar_eclipses_check.csv", "《元史·天文志》", "日食", "year"),
    ("@data/phenomena/yuanshi_lunar_eclipses_check.csv", "《元史·天文志》", "月食", "year"),
    ("hanshu_star_records", "《汉书·天文志》", "凌犯", "公历(儒略)"),
    ("houhanshu_star_records", "《后汉书·天文志》", "凌犯", "公历(儒略)"),
    ("jinshu_star_records", "《晋书·天文志》", "凌犯", "公历(儒略)"),
    ("songshu_star_records", "《宋书·天文志》", "凌犯", "公历(儒略)"),
    ("nanqishu_star_records", "《南齐书·天文志》", "凌犯", "公历(儒略)"),
    ("weishu_star_records", "《魏书·天象志》", "凌犯", "公历(儒略)"),
    ("suishu_star_records", "《隋书·天文志》", "凌犯", "公历(儒略)"),
    ("xintangshu_star_records", "《新唐书·天文志》", "凌犯", "公历(儒略)"),
    ("jiutangshu_star_records", "《旧唐书·天文志》", "凌犯", "公历(儒略)"),
    ("jiuwudaishi_star_records", "《旧五代史·天文志》", "凌犯", "公历(儒略)"),
    ("xinwudaishi_star_records", "《新五代史·司天考》", "凌犯", "公历(儒略)"),
    ("songshi_star_records", "《宋史·天文志》", "凌犯", "公历(儒略)"),
    ("jinshi_star_records", "《金史·天文志》", "凌犯", "公历(儒略)"),
    ("yuanshi_star_records", "《元史·天文志》", "凌犯", "公历(儒略)"),
    ("mingshi_star_records", "《明史·天文志》", "凌犯", "公历(儒略)"),
    ("qingshigao_star_records", "《清史稿·天文志》", "凌犯", "公历(儒略)"),
    ("hanshu_date_shift", "《汉书·天文志》", "日期偏移", "公历(儒略)"),
    ("houhanshu_date_shift", "《后汉书·天文志》", "日期偏移", "公历(儒略)"),
    ("jinshu_date_shift", "《晋书·天文志》", "日期偏移", "公历(儒略)"),
    ("songshu_date_shift", "《宋书·天文志》", "日期偏移", "公历(儒略)"),
    ("nanqishu_date_shift", "《南齐书·天文志》", "日期偏移", "公历(儒略)"),
    ("weishu_date_shift", "《魏书·天象志》", "日期偏移", "公历(儒略)"),
    ("suishu_date_shift", "《隋书·天文志》", "日期偏移", "公历(儒略)"),
    ("xintangshu_date_shift", "《新唐书·天文志》", "日期偏移", "公历(儒略)"),
    ("jiutangshu_date_shift", "《旧唐书·天文志》", "日期偏移", "公历(儒略)"),
    ("jiuwudaishi_date_shift", "《旧五代史·天文志》", "日期偏移", "公历(儒略)"),
    ("xinwudaishi_date_shift", "《新五代史·司天考》", "日期偏移", "公历(儒略)"),
    ("songshi_date_shift", "《宋史·天文志》", "日期偏移", "公历(儒略)"),
    ("jinshi_date_shift", "《金史·天文志》", "日期偏移", "公历(儒略)"),
    ("yuanshi_date_shift", "《元史·天文志》", "日期偏移", "公历(儒略)"),
    ("mingshi_date_shift", "《明史·天文志》", "日期偏移", "公历(儒略)"),
    ("qingshigao_date_shift", "《清史稿·天文志》", "日期偏移", "公历(儒略)"),
]


def shift_metrics(header, rows):
    """日期偏移：行星记录按距记载日最近的 <1° 时段分档。"""
    def near(r):
        v = r[header.index("最近一段（距记载日，日）")]
        if not v:
            return None
        a, b = (int(x) for x in v.split("…"))
        return 0 if a <= 0 <= b else min(abs(a), abs(b))

    sel = [near(r) for r in rows if r[header.index("记载日角距°")]]
    n = len(sel)
    return n, sum(v == 0 for v in sel), sum(v is not None and v <= 30 for v in sel), \
        sum(v is not None and v <= 400 for v in sel), sum(v is None for v in sel)

# 朝代分段（天文纪年，含端点）。按记录日期归段，一部史书跨段时拆开（如《晋书》所记魏事归三国魏）。
DYNASTIES = [
    ("秦", -220, -206, "秦"),
    ("西汉", -205, 8, "西汉（太初以前按复原历，以后按实历朔日表）"),
    ("新莽", 9, 24, "新莽（建丑，月名经月序改制表换算）与更始"),
    ("东汉", 25, 220, "东汉"),
    ("三国", 220, 265, "三国（魏；景初建丑）"),
    ("西晋", 266, 316, "西晋"),
    ("东晋十六国", 317, 420, "东晋"),
    ("南朝", 420, 589, "南朝（宋齐梁陈，实历朔日表）"),
    ("北朝", 386, 581, "北朝（北魏、东魏、北齐、北周；北朝推步历，calendar::push_calendar）"),
    ("隋", 581, 618, "隋（589 年平陈前按北周推步历，此后按实历朔日表）"),
    ("唐", 618, 907, "唐（含武周建子、肃宗末建子）"),
    ("五代十国", 907, 960, "五代十国"),
    ("宋辽金", 960, 1279, "宋辽金"),
    ("元", 1271, 1368, "元"),
    ("明", 1368, 1644, "明"),
    ("清", 1644, 1911, "清"),
]


def record_year(header, row, col):
    v = row[header.index(col)]
    if col == "公历(儒略)":
        neg = v.startswith("-")
        y = int(v.lstrip("-").split("-")[0])
        return -y if neg else y
    return int(v)


# 记录带政权（data/phenomena/<书>_star_records.csv 的 regime 列）者按政权归段：一书兼记南北（《隋书·天文志》梁陈齐周隋）
REGIME_DYNASTY = {"北魏": "北朝", "东魏": "北朝", "西魏": "北朝", "北齐": "北朝", "北周": "北朝",
                  "宋": "南朝", "齐": "南朝", "梁": "南朝", "陈": "南朝", "隋": "隋", "唐": "唐", "武周": "唐",
                  "后梁": "五代十国", "后唐": "五代十国", "后晋": "五代十国", "后汉": "五代十国", "后周": "五代十国",
                  "金": "宋辽金", "元": "元", "明": "明", "清": "清"}


def record_regimes(name):
    """检验表（序号同记录表行序）→ {序: 政权}；记录表无 regime 列者为空。"""
    src = Path("data/phenomena") / (name.rsplit("_", 2)[0] + "_star_records.csv")
    if not src.exists():
        return {}
    return {str(i): r.get("regime", "") for i, r in enumerate(csv.DictReader(open(src, encoding="utf-8")), 1)}


def dynasty_of(year, regime=""):
    """归段：有政权者按 REGIME_DYNASTY；否则按年，段有重叠时（220、420、581 等易代之年）取较早一段，「北朝」只由政权归入。"""
    if regime == "宋" and year >= 960:  # 赵宋；420—479 的「宋」为南朝刘宋
        return "宋辽金"
    if regime in REGIME_DYNASTY:
        return REGIME_DYNASTY[regime]
    for name, a, b, _ in DYNASTIES:
        if name == "北朝":
            continue
        if a <= year <= b:
            return name
    return None


def eclipse_summary(header, rows):
    from collections import Counter

    c = Counter(r[header.index("verdict")] for r in rows)
    return "；".join(f"{k} {n}" for k, n in c.most_common())


def record_metrics(header, rows):
    """凌犯记录：主角距 = 具体星角距 / 天体间最大角距 / 距星官最小角距（依次取有值者），只计干支在月的单日记录与只记到月者。"""
    def val(r, col):
        x = num(r[header.index(col)].split("（")[0]) if col in header else None
        return x

    out = {}
    for kind in ("月", "行星"):
        sel = [r for r in rows if ("月" in r[header.index("记载")]) == (kind == "月") and r[header.index("干支在月")] != "否"]
        v = sorted(x for x in (val(r, "具体星角距°") if val(r, "具体星角距°") is not None else
                               val(r, "天体间最大角距°") if val(r, "天体间最大角距°") is not None else
                               val(r, "距星官最小角距°") for r in sel) if x is not None)
        out[kind] = (len(v), sum(x <= 1 for x in v), sum(x <= 2 for x in v), v[len(v) // 2] if v else None)
    lodge = [r[header.index("推算所在宿")] for r in rows if r[header.index("推算所在宿")]]
    out["宿"] = (len(lodge), sum("同宿" in x for x in lodge), sum("邻宿" in x for x in lodge))
    out["不在月"] = sum(r[header.index("干支在月")] == "否" for r in rows)
    return out


def pct(a, n):
    return f"{a}（{100 * a // n}%）" if n else "—"


def dynasty_part():
    """史书天象记录按朝代：总表 + 各朝代小结 + 逐条表格。"""
    loaded = []
    for name, book, kind, ycol in RECORDS:
        path, meta, rows = read(name)
        if path is None:
            continue
        header, body = rows[0], rows[1:]
        if "regime" in header:  # 日月食检验表自带政权列
            regimes = [r[header.index("regime")] for r in body]
        else:
            reg = record_regimes(name)
            seq = header.index("序") if "序" in header else None
            regimes = [reg.get(r[seq], "") if seq is not None else "" for r in body]
        loaded.append((path, meta, book, kind, header, body, ycol, regimes))
    parts = ["## 七、史书天象记录（按朝代）", "",
             "按记录日期归入朝代；一部史书跨朝代时拆开（如《晋书·天文志》所记魏事归三国）。凌犯记录的「主角距」依次取具体星角距、",
             "天体间最大角距、距星官最小角距；≤1°、≤2° 只计日干支在所记之月的记录（只记到月者按整月逐夜取最小）。",
             "随机日期基线（同记录错开 ±60—240 日）行星 ≤1° 约 1%—2%，见各史书检验的说明行。", "",
             "### 总表", "",
             "「行星日期偏移」：地心角距 <1° 的时段距记载日最近者，依次计记载日当天、±30 日内、±400 日内、±4 年内全无的条数",
             "（`scripts/phenomena/record_date_shift.py`）。", "",
             "| 朝代 | 日食 | 月食 | 月凌犯：条数、≤1°、≤2°、中位 | 行星凌犯与相合：条数、≤1°、≤2°、中位 | 行星日期偏移：条数、当天、±30 日、±400 日、四年全无 | 推算所在宿：条数、同宿、邻宿 | 干支不在所记之月 | 来源 |",
             "|---|---|---|---|---|---|---|---|---|"]
    sections = []
    for dname, a, b, title in DYNASTIES:
        groups = []
        for path, meta, book, kind, header, body, ycol, regimes in loaded:
            sel = [r for r, rg in zip(body, regimes) if dynasty_of(record_year(header, r, ycol), rg) == dname]
            if sel:
                groups.append((path, meta, book, kind, header, sel))
        if not groups:
            parts.append(f"| {dname} | 未检验 | 未检验 | 未检验 | 未检验 | 未检验 | — | — | — |")
            continue
        solar = [eclipse_summary(h, r) + f"（{len(r)} 条）" for _, _, _, k, h, r in groups if k == "日食"]
        lunar = [eclipse_summary(h, r) + f"（{len(r)} 条）" for _, _, _, k, h, r in groups if k == "月食"]
        recs = [(h, r) for _, _, _, k, h, r in groups if k == "凌犯"]
        cells = {"月": "未检验", "行星": "未检验", "宿": "—", "不在月": "—"}
        if recs:
            # 各史书表头相同，合并统计
            h0 = recs[0][0]
            m = record_metrics(h0, [row for _, rr in recs for row in rr])
            for k in ("月", "行星"):
                n, a1, a2, med = m[k]
                cells[k] = f"{n}，{pct(a1, n)}，{pct(a2, n)}，{med:.2f}°" if n else "—"
            n, same, adj = m["宿"]
            cells["宿"] = f"{n}，{same}，{adj}" if n else "—"
            cells["不在月"] = str(m["不在月"])
        shifts = [(h, r) for _, _, _, k, h, r in groups if k == "日期偏移"]
        cells["偏移"] = "未检验"
        if shifts:
            n, d0, d30, d400, none = shift_metrics(shifts[0][0], [row for _, rr in shifts for row in rr])
            cells["偏移"] = f"{n}，{pct(d0, n)}，{pct(d30, n)}，{pct(d400, n)}，{pct(none, n)}" if n else "—"
        books = "、".join(dict.fromkeys(bk for _, _, bk, _, _, _ in groups))
        parts.append(f"| {dname} | {'；'.join(solar) or '未检验'} | {'；'.join(lunar) or '未检验'} | {cells['月']} | {cells['行星']} | {cells['偏移']} | "
                     f"{cells['宿']} | {cells['不在月']} | {books} |")
        sec = [f"### {title}（{a}—{b}）", ""]
        for path, meta, book, kind, header, sel in groups:
            sec += [f"#### {book}{kind}（{len(sel)} 条）", "", f"数据：`{path}`", ""]
            sec += [f"> {x}" for x in meta] + ([""] if meta else [])
            if kind == "凌犯":
                for line in summary(header, sel, "records")[1:]:
                    sec.append(f"- {line}")
            elif kind == "日期偏移":
                n, d0, d30, d400, none = shift_metrics(header, sel)
                sec.append(f"- 行星记录 {n} 条：<1° 时段落在记载日当天 {d0}、±30 日内 {d30}、±400 日内 {d400}、±4 年内全无 {none}")
                from collections import Counter
                gz = Counter(r[header.index("干支可落之月")] for r in sel if r[header.index("干支可落之月")])
                if gz:
                    sec.append("- 日干支不在所记之月者，同一干支可落之月：" + "；".join(f"{k} {v}" for k, v in gz.most_common()))
            else:
                sec.append(f"- {eclipse_summary(header, sel)}")
            sec += ["", md_table(header, sel), ""]
        sections += sec
    return parts + [""] + sections


def md_table(header, rows):
    esc = lambda s: s.replace("|", "\\|")
    lines = ["| " + " | ".join(esc(h) for h in header) + " |", "|" + "---|" * len(header)]
    for r in rows:
        lines.append("| " + " | ".join(esc(c) for c in r) + " |")
    return "\n".join(lines)


def main():
    parts = [
        "# 天象模块逐条校验结果",
        "",
        "由 `scripts/dev/phenomena_checks.sh` 重算、`scripts/phenomena/checks_to_md.py` 生成，**勿手改**。方法、模型与结论见",
        "[celestial-phenomena.md](./celestial-phenomena.md) §九；逐条数据见 `data/phenomena/checks/` 与 `data/phenomena/*_check.csv`。",
        "",
    ]
    for name, title, spec in SECTIONS:
        if title is None:
            parts += [f"## {name}", ""]
            continue
        path, meta, rows = read(name)
        parts.append(f"### {title}")
        parts.append("")
        if path is None:
            parts += ["（尚未生成：运行 `scripts/dev/phenomena_checks.sh`）", ""]
            continue
        header, body = rows[0], rows[1:]
        parts.append(f"数据：`{path}`")
        parts.append("")
        for m in meta:
            parts.append(f"> {m}")
        if meta:
            parts.append("")
        for s in summary(header, body, spec):
            parts.append(f"- {s}")
        parts.append("")
        parts.append(md_table(header, body))
        parts.append("")
    parts += dynasty_part()
    OUT.write_text("\n".join(parts), encoding="utf-8")
    print("写出", OUT)


if __name__ == "__main__":
    main()
