"""由 verify_benji_rili.py、scripts/phenomena/benji_shuo_sky.py、daming_dingshuo.py、liao_calendar_check.py 等的输出生成 doc/calendar/benji-checks.md（逐条，勿手改）。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/benji_checks_to_md.py
"""
import csv
import re
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DYNS = ("汉", "三国", "晋", "南朝", "北朝", "隋", "唐", "五代", "宋", "辽", "金", "元", "明", "清")
BOOK = {"hanshu_benji": "汉书", "houhanshu_benji": "后汉书", "sanguozhi": "三国志", "sanguozhi_wushu": "三国志·吴书", "sanguozhi_shushu": "三国志·蜀书", "jinshu_benji": "晋书", "songshu_benji": "宋书", "nanqishu_benji": "南齐书", "liangshu_benji": "梁书",
        "chenshu_benji": "陈书", "weishu_benji": "魏书", "beiqishu_benji": "北齐书", "zhoushu_benji": "周书", "suishu_benji": "隋书",
        "jiutangshu_benji": "旧唐书", "xintangshu_benji": "新唐书",
        "jiuwudaishi_benji": "旧五代史", "xinwudaishi_benji": "新五代史", "songshi_benji": "宋史", "liaoshi_benji": "辽史",
        "jinshi_benji": "金史", "yuanshi_benji": "元史", "mingshi_benji": "明史", "qingshigao_benji": "清史稿"}


def rows(name):
    p = ROOT / "data/calendar" / name
    if not p.exists():
        return []
    with open(p, encoding="utf-8") as f:
        return list(csv.DictReader(line for line in f if not line.startswith("#")))


def table(head, body):
    out = ["| " + " | ".join(head) + " |", "|" + "---|" * len(head)]
    # 原文偶带维基模板残片（「{{Textquality|50」），留着 VitePress 会当 Vue 表达式解析
    out += ["| " + " | ".join(re.sub(r"\{\{[^}]*\}*", "", str(c)).replace("|", "／") for c in r) + " |" for r in body]
    return out


# 证据等级（高到低）：A 实物历日；B 当时文字；C 当朝或近时所修史书明书朔；D 当朝史书纪日；E 后人追修史书（多转录前史，非独立）。
# 复原性质材料（《通鉴目录》即刘羲叟长历、今人诸表）不作证据。
# 《新唐书》本纪多据实录，与《旧唐书》同级；《南史》《北史》《新五代史》节录前史，与前史同条者按前史计、不重计，前史无者才为 E。
# 同一事件各书互见只算一证：以 (政权, 年, 月, 本纪朔) 为一事，计数以事计，等级取诸书最高者。
E_BOOKS = ("houhanshu", "jinshu", "nanshi", "beishi", "xinwudaishi")


def level(b, shuo=True):
    return "E" if b.startswith(E_BOOKS) else ("C" if shuo else "D")


def pang_conflict(p):
    """旁证「a/b（共n）」：他书同月纪日 n 条中只有 b 条落在以本纪朔起算 30 日内，b < n 即他证与本纪朔冲突。"""
    m = re.match(r"(\d+)/(\d+)（共(\d+)）", p or "")
    return bool(m) and int(m[2]) < int(m[3])


def kaozheng_index():
    """已有考证：(年, 月) → [(本纪朔, 结论)]，取各 *_shuo_kaozheng.csv 与 beichao、liao 考证表。"""
    idx = defaultdict(list)
    for name in ("tang_shuo_kaozheng.csv", "tangsong_cunyi_kaozheng.csv", "wudai_shuo_kaozheng.csv", "song_shuo_kaozheng.csv",
                 "yuan_shuo_kaozheng.csv", "beichao_shuo_kaozheng.csv", "liao_shuo_kaozheng.csv"):
        for r in rows(name):
            idx[(r["年"], r["月"])].append((r.get("本纪朔") or r.get("本纪") or "", r.get("结论") or r.get("初判") or ""))
    return idx


def shishu_vs_zhang(r, errata, qita, kz, sky):
    """史载与张表不合之条：史书误（已有考证沿用）／张表误（推步合史载且无他证冲突）／待核。返回 (判, 依据)。"""
    key = (r["书"], r["年"], r["月"], r["本纪朔"])
    if key[:3] in errata:
        return "史书误", "已考讹（benji_shuo_errata.csv）"
    if r["类"] == "其他":
        j = qita.get(key, "")
        if j and not j.startswith("未明"):
            return "史书误", f"其他类考证：{j}"
        return "待核", "其他类考证：未明" if j else "其他类未考"
    for g, c in kz.get((r["年"], r["月"]), []):
        if r["本纪朔"] in g and ("讹" in c or "」误" in c):
            return "史书误", "已有考证：" + c[:40]
    if r["本项目推步"] == "合":
        if pang_conflict(r["旁证（张表/本纪朔）"]):
            return "待核", "推步合本纪，但他书同月纪日与本纪朔冲突"
        return "张表误", f"推步合本纪（{level(r['书'])} 级）"
    s = sky.get(key, {})
    if r["日食"] and r["朝代"] in ("汉", "三国", "晋", "南朝", "北朝", "隋") and s.get("合朔地方时日差（合朔日−本纪朔）") == "0":
        return "史书误", "平朔时代食日书朔（所书为合朔之日，非历日之朔）"
    return "待核", ""


def tuibu_vs_zhang(r):
    """本项目推步与张表互异之条（本纪只合其一）：①史载证张表误、②史载证推步误、③无据待核。返回 (分类, 等级)。"""
    hr, ht = r["本项目推步"] == "合", r["张表"] == "合"
    if not r["本项目推步"] or hr == ht:
        return None
    if r["类"] == "其他":
        return "③无据待核", "—（本纪自误）"
    if pang_conflict(r["旁证（张表/本纪朔）"]) and hr:
        return "③无据待核", "—（他证冲突）"
    return ("①史载证张表误" if hr else "②史载证推步误"), level(r["书"])


def book(b):
    return "、".join(BOOK.get(x, x) for x in b.split("、")) if b else ""


# 纪日抽取规则（zhi_star_records.py 纪日模式；2026-11-27 据同误成因逐条表 doc/calendar/tongwu-causes.md 所列抽取之误补订）
JIRI_METHOD = [
    "## 方法：纪日抽取", "",
    "本纪纪日表逐分句读维基文库录文（去 `<ref>` 校勘记、`{{*|…}}` 注文，点校本改字取改正后文字），维持当前纪年（年号、年数）与系月，"
    "带日干支的分句记一条。下列规则以原文为准，不以合张表或推步为目标；改订前后逐行比对，变动之行逐条可从原文解释。",
    "",
    "1. **引语、奏报中的日期不改系年系月**：引号「」内文字（诏令、奏状、牒文、点校者案语中引他书之日）解析时遮去，其中的年月日、年号都不作纪年"
    "（如《旧五代史·庄宗纪》「有司上言：『八月二十二日夜，熒惑犯星……』」、《宋史·仁宗纪》「去尊號『寶元』字」、《旧唐书·武宗纪》"
    "引状「大和七年嫁……開成三年……」）；句中以日序书日者（「徐州、鄴都上言，十月二十五日夜，地大震」「賽尚阿奏，六月初四日，馳抵桂林」）"
    "是所言之日，不改当前系月（句首者仍是纪事，如卷首传记「四年十一月二十日，秦王誅」）；前一分句以「奏」「言」「報」「稱」「云」收尾者，"
    "其后的年月日亦为所言之日（「四川制司言：二月甲寅，大元兵攻嘉定城」），不记、不改系月；「自……」后的年数是历时（「自遜位至薨，十有四年」），"
    "不是纪年；句中月首只认月序前进者，倒退的是追述（《元史·文宗纪》七月「……麥苗枯；六月壬子雨，至是日乃止」）。",
    "2. **月首**：句首「月+干支」其后正文紧接、不以「，」隔开者（「十月丙午謁陵」「五月甲辰至戊申，日赤如赭」「二月壬子望，月蝕既」）仍是月首；"
    "录文月首干支讹脱者（「十月辛居朔」「三月內辰朔」「十二月未朔」「十二月申午朔」「秋七月辛朔」「四年春正月已亥」）只换月不记日；"
    "月首「已」后接阴支者为「己」（「秋七月已丑」）；句首「年数+季」而其后讹脱（「二十五年秋七年」）仍取其年。",
    "3. **相对纪年**：句首「明年」「明歲」「翌年」「次年」「踰年」承当前年加一年（《后汉书·顺帝纪》卷首「延光三年……明年三月，安帝崩……"
    "十一月丁巳」为延光四年 125 年；《旧五代史·少帝纪》「明年正月朔，契丹主次東京城北」为 947 年）。只认卷首传记（编年纪事开始之前）、"
    "段首与同段书「崩」以后者（身后叙事），编年纪事段中者是预叙（《明史·英宗纪》「踰年，釋中敷為民」、诏文「明年，河南漕糧照數補還」），"
    "不改纪年；越出当前年号末年者改记该年所行年号。岁次纪年「歲戊辰七月庚午」取不早于当前年而岁次相合之最近一年；「某帝即位之年」"
    "不书年数，其年无从定，停记到下一个纪年。",
    "4. **改元与年号越界**：改元句年号前可冠国号（《旧唐书·高祖纪》「改隋義寧二年為唐武德元年」，原未识别，武德二年至九年之事误记作"
    "義寧三至九年、且早一年）；卷首传记中段中「年号+年」照常切换（《宋史·真宗纪》「淳化五年九月進封壽王……至道元年八月立為皇太子」，"
    "原未读，其后「三年三月，太宗崩」配作端拱三年 990，实为至道三年 997）；专名下划线标记 `<u>` 去掉后再读（《宋史·寧宗紀》"
    "「<u>淳熙</u>五年」）；不含「年」的节标题（另一帝纪起首，如《北史》卷七「廢帝」）其后重按卷首传记读；「初」起句中的改元句是追述，"
    "不改纪年。年数越出当前年号末年、年号表各政权皆无此年数、本政权其间已另有改元而又无可回退之年号者，不照写成越界之年："
    "编年段中者是追述（《三国志·文帝纪》「初，漢熹平五年……至四十五年，登尚在」），略去本分句；段首或卷首传记中者停记到下一个可识别的纪年。"
    "本政权其间无改元者是沿用旧年号（晋沿用唐天祐至二十年），照常。",
    "", "以上规则改订（2026-11-27）后，同误成因表原列「纪日抽取误」202 条中 168 条随规则改正（系月、系年与原文一致），余 34 条复核为原表判读之误"
    "（季名、年数前缀的月首未认出、录文「-{丑}-」标记、奏报中「去年十一月」被当作月首等）或本为史文之误（《旧唐书·玄宗纪》「上元二年四月甲寅，崩」"
    "实为宝应元年），纪日表本不误；逐条见 [tongwu-causes.md](./tongwu-causes.md)。", "",
]


def main():
    chk, leaps, sky, base = rows("benji_shuo_check.csv"), rows("benji_leap_check.csv"), rows("benji_shuo_sky.csv"), \
        rows("benji_shuo_baseline.csv")
    sky_by = {(s["书"], s["年"], s["月"], s["本纪朔"]): s for s in sky}
    errata = {(r["书"], r["年"], r["月"]) for r in rows("benji_shuo_errata.csv")}
    qita = {(r["书"], r["年"], r["月"], r["本纪朔"]): r["判"] for r in rows("benji_qita_kaozheng_notes.csv")}
    kz = kaozheng_index()
    pan = {id(r): shishu_vs_zhang(r, errata, qita, kz, sky_by) for r in chk if r["类"]}
    fen = {id(r): tuibu_vs_zhang(r) for r in chk}
    L = ["# 以本纪核验历日：逐条结果", "",
         "由 `scripts/calendar_data/verify_benji_rili.py`、`scripts/phenomena/benji_shuo_sky.py` 计算，"
         "`scripts/calendar_data/benji_checks_to_md.py` 生成，**勿手改**。方法与结论见 "
         "[historical-calendar.md](./historical-calendar.md)「以本纪核验历日」；逐条数据见 `data/calendar/benji_*_check.csv 等`，"
         "本纪纪日由 `scripts/calendar_data/qinhan_eval/zhi_star_records.py` 的纪日模式（`*_benji` 配置）抽取，见 `data/calendar/本纪纪日/`。", ""]
    L += JIRI_METHOD
    # 一、明书朔日
    st = defaultdict(lambda: defaultdict(int))
    for r in chk:
        s = st[r["朝代"]]
        s["n"] += 1
        s["ok"] += r["张表"] == "合"
        s[r["类"]] += 1
        if r["本项目推步"]:
            s["pn"] += 1
            s["pok"] += r["本项目推步"] == "合"
    # 以事计：同朝代同年月同朔干支之各书互见合为一事；判取史书误 > 张表误 > 待核（一书已考讹即同一记载之误），分类等级取最高
    ev_pan, ev_fen, ev_dyn = {}, {}, {}
    for r in chk:
        k = (r["政权"], r["年"], r["月"], r["本纪朔"])
        ev_dyn.setdefault(k, r["朝代"])
        if r["类"]:
            p0 = pan[id(r)][0]
            if k not in ev_pan or ["史书误", "张表误", "待核"].index(p0) < ["史书误", "张表误", "待核"].index(ev_pan[k]):
                ev_pan[k] = p0
        f = fen[id(r)]
        if f:
            old = ev_fen.get(k)
            if old is None or (old[0][0] == f[0][0] and f[1] < old[1]):
                ev_fen[k] = f
    # 逐条显示：同事之书有更高一级者（如《北史》条前史有同条），按前史之级注出
    for r in chk:
        f = fen[id(r)]
        if f:
            e = ev_fen[(r["政权"], r["年"], r["月"], r["本纪朔"])]
            if e[0] == f[0] and e[1] < f[1]:
                fen[id(r)] = (f[0], f"{e[1]}，前史同条，不重计")
    # 推步与张表相同（本纪合两家，或两家皆不合）之事：有证 / 同误（皆不合者，注判史书误与否）
    ev_same = {}
    for r in chk:
        if not r["本项目推步"] or fen[id(r)]:
            continue
        k = (r["政权"], r["年"], r["月"], r["本纪朔"])
        both = r["本项目推步"] == "合" and r["张表"] == "合"
        c = ("有证", level(r["书"])) if both else ("同误", pan.get(id(r), ("待核",))[0])
        old = ev_same.get(k)
        if old is None or (old[0] == c[0] == "有证" and c[1] < old[1]):
            ev_same[k] = c
    same_lv = defaultdict(int)
    for k, c in ev_same.items():
        st[ev_dyn[k]]["同" + c[0]] += 1
        same_lv[c] += 1
    for k, p0 in ev_pan.items():
        st[ev_dyn[k]]["判" + p0] += 1
    for k, f in ev_fen.items():
        st[ev_dyn[k]][f[0][0]] += 1
    L += ["## 一、明书朔日（本纪「某月某干支朔」）", "",
          "不合者分三类：**朔差一二日**（朔日推算之差，张表或本纪有一误）、**差一月**（所书干支为张表邻月或闰月之朔，闰位或月序不同）、"
          "**其他**（多为干支传写之误或解析误定年月，不作张表证据）。北朝、隋另列本项目推步表。", "",
          "不合张表者再按证据定谁误（各表「判」列）：**史书误**——已考为讹误（`benji_shuo_errata.csv`）、「其他」类已判致误之由"
          "（[benji-qita-kaozheng.md](./benji-qita-kaozheng.md)，「未明」除外）、已有考证表判为讹，或平朔时代日食之条所书即合朔之日（食日书朔）；"
          "**张表误**——本项目推步合本纪而他书同月纪日不与本纪朔冲突（北朝、隋、辽、金之推步为本政权之历，此时「张表误」指张表不能代表该政权历日）；"
          "**待核**——余者（史书、张表各有可能，无据不否定张表）。天文计算的合朔所在日只作旁证，不据以判。", "",
          "本项目推步与张表互异、本纪只合其一之条另按三类分（「分类」列）：**①史载证张表误**（本纪合推步）、**②史载证推步误**（本纪合张表）、"
          "**③无据待核**（本纪属「其他」类自误，或他书同月纪日与本纪朔冲突）；①②注证据等级：C 当朝或近时所修史书明书朔，"
          "E 后人追修史书（《后汉书》《晋书》；《南史》《北史》《新五代史》前史无同条者）。《新唐书》本纪多据实录，与《旧唐书》同级；"
          "各书互见同一记载只算一证（同政权同年月同朔干支合为一事，等级取诸书最高，前史有同条即按前史计）。"
          "单条明书朔与他证冲突者不判，归③。合张表不算证实张表。表中「不合中」「互异」各栏以事计，「条数」等栏以条计。", ""]
    L += table(["朝代", "条数", "与张表合", "朔差一二日", "差一月", "其他", "不合中（事）：史书误", "张表误", "待核", "本项目推步合",
                "推步与张表相同（事）：有证", "同误", "推步与张表互异（事）：①", "②", "③"],
               [[d, st[d]["n"], st[d]["ok"], st[d]["朔差一二日"], st[d]["差一月"], st[d]["其他"],
                 st[d]["判史书误"], st[d]["判张表误"], st[d]["判待核"],
                 f"{st[d]['pok']}/{st[d]['pn']}" if st[d]["pn"] else "",
                 *([st[d]["同有证"], st[d]["同同误"], st[d]["①"], st[d]["②"], st[d]["③"]] if st[d]["pn"] else ["", "", "", "", ""])] for d in DYNS if st[d]["n"]])
    tot = lambda k: sum(st[d][k] for d in DYNS)
    lv = defaultdict(int)
    for f in ev_fen.values():
        lv[(f[0][0], f[1])] += 1
    L += ["", f"合计（以事计，各书互见合为一事）：不合张表 {tot('判史书误') + tot('判张表误') + tot('判待核')} 事，史书误 {tot('判史书误')}、张表误 {tot('判张表误')}、待核 {tot('判待核')}；"
          f"推步与张表互异（本纪只合其一）{tot('①') + tot('②') + tot('③')} 事，①{tot('①')}（"
          + "、".join(f"{k} 级 {v}" for (c, k), v in sorted(lv.items()) if c == "①") + f"）、②{tot('②')}（"
          + "、".join(f"{k} 级 {v}" for (c, k), v in sorted(lv.items()) if c == "②") + f"）、③{tot('③')}；"
          f"有本项目推步之条中推步与张表相同 {tot('同有证') + tot('同同误')} 事：有证（本纪合两家）{tot('同有证')}（"
          + "、".join(f"{k} 级 {v}" for (c, k), v in sorted(same_lv.items()) if c == "有证") + f"），同误（本纪两家皆不合）{tot('同同误')}（"
          + "、".join(f"判{k} {v}" for (c, k), v in sorted(same_lv.items()) if c == "同误") + "）。逐月之相同、互异分类（含无证之月）见第三节与分政权检验。", ""]
    L += ["", "### 朔差一二日：实际合朔（第 6 级，只作旁证）", "",
          "合朔为 DE441 视合朔换算为都城地方平时；「日差」= 合朔所在日 − 本纪朔日。日食记录在平朔时代常把食日（即合朔日）书为朔，"
          "不作历日证据（见 historical-calendar.md）。", ""]
    body = []
    for r in chk:
        if r["类"] != "朔差一二日":
            continue
        s = sky_by.get((r["书"], r["年"], r["月"], r["本纪朔"]), {})
        f = fen[id(r)]
        body.append([BOOK.get(r["书"], r["书"]), r["政权"], r["年"], r["纪年"], r["月"], r["本纪朔"], r["张表"], r["本项目推步"], r["日食"],
                     s.get("合朔地方时日差（合朔日−本纪朔）", ""), s.get("合朔地方时", ""), s.get("判定", ""), book(r["他书同记"]),
                     r["旁证（张表/本纪朔）"], "：".join(x for x in pan[id(r)] if x), f"{f[0]}（{f[1]}）" if f else "", r["原文"]])
    L += table(["书", "政权", "年", "纪年", "月", "本纪朔", "张表", "本项目推步", "日食", "日差", "合朔地方时", "合朔所在（旁证）", "他书同记",
                "旁证（张表/本纪朔）", "判", "分类", "原文"], body)
    for kind in ("差一月", "其他"):
        L += ["", f"### {kind}", "",
              "「旁证」为他书同月记日干支落在张表该月内者 / 落在以本纪所书朔起算 30 日内者（共几条）。致误之由（一字误、「朔」字衍、年月误等）逐条见 [benji-qita-kaozheng.md](./benji-qita-kaozheng.md)。", ""]
        L += table(["书", "政权", "年", "纪年", "月", "本纪朔", "张表", "本项目推步", "日食", "他书同记", "旁证（张表/本纪朔）", "判", "分类", "原文"],
                   [[BOOK.get(r["书"], r["书"]), r["政权"], r["年"], r["纪年"], r["月"], r["本纪朔"], r["张表"], r["本项目推步"], r["日食"],
                     book(r["他书同记"]), r["旁证（张表/本纪朔）"], "：".join(x for x in pan[id(r)] if x),
                     f"{fen[id(r)][0]}（{fen[id(r)][1]}）" if fen[id(r)] else "", r["原文"]] for r in chk if r["类"] == kind])
    if base:
        L += ["", "### 相合者之朔日与实际合朔", "",
              "本纪明书朔日与张表相合者，合朔（都城地方平时）所在日 − 朔日之分布。−1 即朔在合朔次日；唐代此类合朔时刻几乎都在 16 时以后，为进朔。", ""]
        by = defaultdict(dict)
        for b in base:
            by[b["朝代"]][int(b["差"])] = int(b["次数"])
        L += table(["朝代", "−1", "0", "+1"], [[d, by[d].get(-1, 0), by[d].get(0, 0), by[d].get(1, 0)] for d in DYNS if d in by])
    # 二、闰位
    ls = defaultdict(lambda: defaultdict(int))
    for r in leaps:
        ls[r["朝代"]]["n"] += 1
        ls[r["朝代"]][r["张表"] if r["张表"] in ("合", "略书相容") else "矛盾"] += 1
    L += ["", "## 二、本纪「閏月」位置", "",
          "本纪「閏月」只保证在所记最后一月之后：张表闰位更晚为**略书相容**，更早或张表当年无闰为**矛盾**。"
          "北朝张表本非当时行用之历（北朝推步表闰位见 `data/calendar/leap_evidence.csv` 与 push-algorithms.md §3.4）。", ""]
    L += table(["朝代", "条数", "合", "略书相容", "矛盾"],
               [[d, ls[d]["n"], ls[d]["合"], ls[d]["略书相容"], ls[d]["矛盾"]] for d in DYNS if ls[d]["n"]])
    L += ["", "### 矛盾者", ""]
    L += table(["书", "朝代", "年", "纪年", "本纪闰位（不早于）", "张表", "原文"],
               [[BOOK.get(r["书"], r["书"]), r["朝代"], r["年"], r["纪年"], r["本纪闰位"], r["张表"], r["原文"]]
                for r in leaps if r["张表"] not in ("合", "略书相容")])
    # 三、中原一线按术推步与张表（examples/zhongyuan_reckoning_eval.rs）
    ev = rows("zhongyuan_reckoning_eval.csv")
    if ev:
        L += ["", "## 三、中原一线按术推步与张表：逐历", "",
              "本项目按当时行用之历的术文推步（平朔诸历与唐至明定朔诸历），与张表逐月比较，并以本纪明书朔日（不含「其他」类）逐条检验两家。"
              "「本纪合推步」「本纪合张表」按月名（含闰）取该月朔日比对；已考为讹误的本纪朔日不计入，另列「已考讹不计」，明细照列。推步合数含史载例外表的修正；其中只有本纪明书、推步临限而直接从本纪者另列「从本纪修正」，合推步减去此数即纯按术推算之合数。中原一线以实物历日为第 1 级来源之年（明大统、清时宪书实物）不在推步检验之列。逐历取法：本纪合推步不少于合张表之历缺省取推步，余缺省取张表"
              "（见 [rili-sources.md](./rili-sources.md)）。只取中原一线政权之条（明书朔与纪日同）：北魏、东魏、西魏、北齐、北周与隋平陈（590）以前，"
              "元 1280 年以前（行金重修大明历），明 1644 年后、清 1645 年前之条不检中原诸历——《北史》《隋书》所记北朝之事由分政权推步检验"
              "（[regime-benji-checks.md](./regime-benji-checks.md)）。由 `rust/core/examples/zhongyuan_reckoning_eval.rs` 计算。", ""]
        L += ["推步与张表互异之月（「月数 − 与张表同」，含闰位所致月名之异）逐月按证据三分：**①硬据证张表误**、**②硬据证推步误**、**③无据待核**"
              "（无证、证据冲突，或只一见、只一源而等级不足）。证据取实物历日（A：整年实物、出土具注历）、写本题记册文（B）、本纪与明清实录明书朔"
              "（C；追修史书 E）、本纪纪日与「閏月」（D；追修史书 E）、《汉书》秦汉初书朔晦（C）与纪日（D）；619—1911 年朔日之异沿用定朔诸历逐月考证之分类"
              "（[dingshuo-zhangbiao-diffs.md](./dingshuo-zhangbiao-diffs.md)，考证亦收实录书朔），考证按次序配月而月名不同之月（闰位、月名之异）"
              "另按月名收纪日与「閏月」；明实录校勘表判讹或存疑之实录书朔照其结论不计。有 A 级者从 A；余者诸证须一致，相冲突则归③；A、B、C 级一条即判，"
              "D、E 级须异源两条（同一部书——含同源诸书——同月诸事只算一源，2026-11-27 起）。张表与推步皆为今人复原，合张表不算证实。"
              "逐月见 `data/calendar/zhongyuan_reckoning_diffs.csv`。", ""]
        L += table(["历", "起", "止", "月数", "与张表同", "闰位不同之岁", "本纪条数", "本纪合推步", "本纪合张表", "只合推步", "只合张表", "已考讹不计", "从本纪修正",
                    "相同：有证", "同误", "无证", "互异之月", "①", "②", "③"],
                   [[r["历"], r["起"], r["止"], r["月数"], r["与张表同"], r["闰位不同之岁"] or "—", r["本纪条数"], r["本纪合推步"],
                     r["本纪合张表"], r["只合推步"], r["只合张表"], r.get("已考讹不计") or "0", r.get("从本纪修正") or "0",
                     r.get("相同有证", ""), r.get("相同同误", ""), r.get("相同无证", ""),
                     r.get("互异之月", ""), r.get("①硬据证张表误", ""), r.get("②硬据证推步误", ""), r.get("③无据待核", "")] for r in ev])
        zm = rows("zhongyuan_reckoning_months.csv")
        if zm:
            sc = defaultdict(int)
            for r in zm:
                if r["异同"] == "相同":
                    sc[(r["分类"], r["证据等级"])] += 1
            L += ["", "推步与张表相同之月（朔日、月名皆同）逐月分：**有证**——实物（A）、明书朔（C，追修史书 E）合两家，或两条以上独立纪日"
                  "（落在朔日、晦日或书日序者，D）合两家；**同误**——史载与两家皆不合（注已判史书误与否；纪日须两处以上独立错误）；**无证**——只是两家相同，"
                  "不算证实（纪日只一见者亦归此，注倾向）。不合两家之纪日按独立错误计：同书同月按原文行序，两家月外之条连续成串（中间无月内之条隔开）"
                  "多出一处脱月名、月名错位或系月之误，一串只算一处；月内而日序、晦不合者各算一处；同源之书（旧、新唐书等）取处数最多者，各史源相加。逐月分类在中原一线逐月表之「分类」列（`data/calendar/zhongyuan_reckoning_months.csv`）。", ""]
            L += table(["相同之月", "A", "B", "C", "D", "E", "计"],
                       [[c, *[sc[(c, k)] for k in "ABCDE"], sum(v for kk, v in sc.items() if kk[0] == c)] for c in ("有证", "同误")]
                       + [["无证", "—", "—", "—", "—", "—", sum(v for kk, v in sc.items() if kk[0] == "无证")]])
        zd = rows("zhongyuan_reckoning_diffs.csv")
        if zd:
            cnt = defaultdict(int)
            for r in zd:
                cnt[(r["分类"][0], r["证据等级"][0])] += 1
            L += ["", f"合计互异之月 {len(zd)}：①{sum(v for k, v in cnt.items() if k[0] == '①')}、②{sum(v for k, v in cnt.items() if k[0] == '②')}、"
                  f"③{sum(v for k, v in cnt.items() if k[0] == '③')}。三类 × 等级：", ""]
            L += table(["分类", "A", "B", "C", "D", "E", "计"],
                       [[c, *[cnt[(c[0], k)] for k in "ABCDE"], sum(v for kk, v in cnt.items() if kk[0] == c[0])] for c in ("①硬据证张表误", "②硬据证推步误")]
                       + [["③无据待核", "—", "—", "—", "—", "—", sum(v for k, v in cnt.items() if k[0] == "③")]])
            L += ["", "①② 之月：", ""]
            L += table(["历", "年", "月", "推步朔", "张表朔", "异之类", "分类", "等级", "证据"],
                       [[r["历"], r["年"], r["月"], r["推步朔"], r["张表朔"], r["异之类"], r["分类"], r["证据等级"], r["证据"]] for r in zd if not r["分类"].startswith("③")])
        dt = rows("zhongyuan_reckoning_benji.csv")
        L += ["", "### 推步与张表不同、或本纪只合其一之条", ""]
        L += table(["历", "书", "年", "月", "本纪朔", "推步朔", "张表朔", "本纪合推步", "本纪合张表", "类", "原文"],
                   [[r["历"], BOOK.get(r["书"], r["书"]), r["年"], r["月"], r["本纪朔"], r["推步朔"], r["张表朔"], r["本纪合推步"],
                     r["本纪合张表"], r["类"], r["原文"]] for r in dt])
    qh = rows("qinhan_benji_check.csv")
    if qh:
        L += ["", "### 秦汉初：《汉书》本纪纪日与实用历推步、李忠林复原历", "",
              "高帝至太初元年四月本纪「某月某干支」81 条（`data/calendar/hanshu_qinhan_benji.csv`）。月份为本项目夏正月序（十至十二月属上一夏正年，"
              "后九月记闰九月）；「推步」为秦汉初实用历推步（颛顼历置闰，合朔改正逐段由出土历日定出），"
              "「复原」为李忠林复原历（张表前 106 年起即同一复原）；「实物」为该年有出土历谱整年时对实物之合否，实物否定之条不计入推步、复原之比较。"
              "书朔者须为该月朔日，书晦者须为该月末日，余者只判在月内。书朔、晦 "
              f"{sum(1 for r in qh if r['朔晦'])} 条（实物否定 {sum(1 for r in qh if r['朔晦'] and r.get('实物合') == '0')} 条），推步 "
              f"{sum(int(r['推步合']) for r in qh if r['朔晦'] and r.get('实物合') != '0')}、复原 {sum(int(r['复原合']) for r in qh if r['朔晦'] and r.get('实物合') != '0')}；全部 {len(qh)} 条"
              f"推步合 {sum(int(r['推步合']) for r in qh)}、复原合 {sum(int(r['复原合']) for r in qh)}。"
              "说明见 [qin-han-calendar-evaluation.md](./qin-han-calendar-evaluation.md) 第七节。", ""]
        L += table(["卷", "纪年年", "夏正年", "月", "干支", "朔晦", "推步该月朔", "推步合", "复原该月朔", "复原合", "实物", "原文"],
                   [[r["卷"], r["纪年年"], r["夏正年"], r["月"], r["干支"], r["朔晦"] or "—", r["推步该月朔"], r["推步合"],
                     r["复原该月朔"], r["复原合"], {"1": "✓", "0": "✗"}.get(r.get("实物合", ""), ""), r["原文"]] for r in qh])
    # 四、敦煌历日（邓文宽 1996 附录一；check_dunhuang_comparison.py）
    head = L
    blocks = {}
    L = blocks["dunhuang"] = []
    dh = rows("dunhuang_vs_main.csv")
    if dh:
        L += ["", "## 三、敦煌历日与张表（邓文宽 1996 附录一）", "",
              "敦煌出土具注历日为当地自造之历（吐蕃、归义军时期），是独立历表，不据以改张表。转录 `data/calendar/敦煌历日_比较表.csv`"
              "（中原曆一栏据陈垣，与张表 447 格全合），逐月比较见 `data/calendar/dunhuang_vs_main.csv`。下表只计原文有据之月"
              "（不含原书括号推算之格）；「日差」= 敦煌朔 − 张表朔。", ""]
        per = defaultdict(lambda: defaultdict(int))
        for r in dh:
            if r["推算"]:
                continue
            k = (r["公元"], r["纪年"], r["文书"], r["时期"])
            per[k]["n"] += 1
            per[k][int(r["日差"])] += 1
        body = []
        for (y, era, doc, p), c in sorted(per.items(), key=lambda kv: int(kv[0][0])):
            body.append([y, era, doc, p, c["n"], c[0], c[-2] + c[-1], c[1] + c[2]])
        L += table(["公元", "纪年", "文书", "时期", "有据之月", "与张表同朔", "敦早（1—2 日）", "敦迟（1—2 日）"], body)
        L += ["", "### 敦煌朔与张表不同之月（有据者）", ""]
        L += table(["公元", "纪年", "文书", "张表月", "敦煌月", "敦煌朔", "张表朔", "日差", "书页"],
                   [[r["公元"], r["纪年"], r["文书"], r["张表月"], r["敦煌月"], r["敦煌朔"], r["张表朔"], r["日差"], r["书页"]]
                    for r in dh if not r["推算"] and r["日差"] != "0"])
    # 四、金大明历（daming_dingshuo.py）
    L = blocks["jin"] = []
    jn = rows("daming_dingshuo_check.csv")
    if jn:
        L += ["", "## 四、金大明历定朔与本纪明书朔日", "",
              "重修大明历定朔、进朔（`calendar::daming_li`）；1182 年起用术文原限，1182 年前杨级历以之代推，进朔限加经验改正 0.25 时"
              "（见 historical-calendar.md「金大明历」）。逐条见 `data/calendar/daming_dingshuo_check.csv`；下表列本纪不合张表、"
              "或不合金历、或临限（1182 年前超术文限 0—0.5 时）之条。", ""]
        n = len(jn)
        L += [f"共 {n} 条：合张表 {sum(int(r['本纪合张表']) for r in jn)}，合金历不进朔 {sum(int(r['本纪合大明不进朔']) for r in jn)}，"
              f"合术文进朔 {sum(int(r['本纪合术文进朔']) for r in jn)}，合缺省进朔 {sum(int(r['本纪合大明进朔']) for r in jn)}。", ""]
        L += table(["年", "纪年", "月", "本纪朔", "张表朔", "金定朔", "定朔小余（时）", "术文限（时）", "术文进朔", "缺省进朔", "日食", "原文"],
                   [[r["年"], r["纪年"], r["月"], r["本纪朔"], r["张表朔"], r["大明定朔"], r["定朔小余（时）"], r["术文进朔限（时）"],
                     r["术文进朔"], r["大明进朔（校准）"], r["日食"], r["原文"]]
                    for r in jn if r["临限"] or r["本纪合张表"] == "0" or r["本纪合大明进朔"] == "0"])
    # 五、辽历（liao_calendar_check.py）
    L = blocks["liao"] = []
    lc = rows("liao_calendar_check.csv")
    if lc:
        L += ["", "## 五、辽历：本纪明书朔日与宋历、祖冲之大明历、金大明历、宣明代推", "",
              "《辽史·历象志》所载「大明历」为祖冲之法（志自言「歷書法禁不可得……得祖沖之法於外史」），以本纪检之。"
              "祖冲之平朔与任何定朔历偶合约 68%（994—1125 年张表 1110/1632），辽本纪合平朔之率与之相当，合定朔则在 95% 以上："
              "辽行定朔之历，非祖冲之法。宋元丰元年司天监考辽历「遼己未歲氣朔與《宣明曆》合」，本项目分政权「辽」947—1087 年以宣明代推"
              "（1088 年起本纪闰位从宋，不推），见 calendar-compendium.md「辽」与 `data/calendar/liao_shuo_kaozheng.csv`。"
              "逐条见 `data/calendar/liao_calendar_check.csv`。", ""]
        per = defaultdict(lambda: [0, 0, 0, 0, 0])
        for r in lc:
            c = per[r["时期"]]
            c[0] += 1
            c[1] += int(r["合宋"])
            c[2] += int(r["合祖冲之"])
            c[3] += int(r["合金大明"])
            c[4] += int(r.get("合宣明推步") or 0)
        L += table(["时期", "明书朔日", "合宋历（张表）", "合祖冲之平朔", "合金大明历上推", "合宣明代推"],
                   [[p, *c[:4], c[4] if any(r.get("宣明推步") for r in lc if r["时期"] == p) else "—"] for p, c in per.items()])
        L += ["", "### 本纪不合宋历之条", ""]
        L += table(["年", "纪年", "月", "本纪朔", "宋历", "宣明代推", "祖冲之平朔", "金定朔", "定朔小余（时）", "进朔限（时）", "金进朔", "原文"],
                   [[r["年"], r["纪年"], r["月"], r["本纪朔"], r["宋历（张表）"], r.get("宣明推步", ""), r["祖冲之平朔"], r["金大明定朔"],
                     r["定朔小余（时）"], r["进朔限（时）"], r["金大明进朔"], r["原文"]] for r in lc if r["合宋"] == "0"])
    # 六、唐定朔诸历（linde_dingshuo.py、dayan_dingshuo.py、chongxuan_dingshuo.py）
    L = blocks["tang"] = []
    tang = [("戊寅历定朔 627—645", "wuyin_ding_check.csv", "戊寅定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("戊寅历平朔 646—664", "wuyin_ping_check.csv", "戊寅定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("麟德历 665—728", "linde_dingshuo_check.csv", "麟德定朔", "本纪合麟德进朔", "本纪合麟德无进朔", "麟德定朔小余（时）"),
            ("大衍历 729—761", "dayan_dingshuo_check.csv", "大衍定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("五纪历 762—783", "wuji_dingshuo_check.csv", "五纪定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("正元历 784—806", "zhengyuan_dingshuo_check.csv", "正元定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("宣明历 822—892", "xuanming_dingshuo_check.csv", "宣明定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("崇玄历 893—938", "chongxuan_dingshuo_check.csv", "崇玄定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("崇天历 1024—1064（《宋史》）", "song_崇天_check.csv", "崇天定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("纪元历 1106—1135（《宋史》）", "song_纪元_check.csv", "纪元定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("明天历 1065—1074（《宋史》）", "song_明天_check.csv", "明天定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("观天历 1094—1102（《宋史》）", "song_观天_check.csv", "观天定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("统元历 1136—1167（《宋史》）", "song_统元_check.csv", "统元定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("乾道历 1168—1176（《宋史》）", "song_乾道_check.csv", "乾道定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("淳熙历 1177—1190（《宋史》）", "song_淳熙_check.csv", "淳熙定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("会元历 1191—1198（《宋史》）", "song_会元_check.csv", "会元定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("统天历 1199—1207（《宋史》）", "song_统天_check.csv", "统天定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("开禧历 1208—1251（《宋史》）", "song_开禧_check.csv", "开禧定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("成天历 1271—1276（《宋史》）", "song_成天_check.csv", "成天定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("授时历 1281—1367（通轨应数）", "shoushi_tonggui_check.csv", "授时（通轨应数）定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("大统历 1368—1644", "datong_check.csv", "大统定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）")]
    L += ["", "## 六、唐至明定朔诸历与本纪明书朔日", "", "术数见 [calendar-compendium.md](./calendar-compendium.md)。下表列本纪不合张表、或不合进朔限之条（限：麟德试用 18 时，崇玄经验 18 时（术文限不合本纪），大衍 19 时，宣明、五纪、正元据志文，崇天日入分，纪元 3/4 日法并春分后减限）。",
          "",
          "此处「术推进朔」为纯按术文与进朔限之推（未经史载例外表）。术推与张表互异之条按本纪三分（「分类」列）：①本纪合术推（史载证张表误）、"
          "②本纪合张表（史载证术推误，多已据以入例外表）、③本纪两家皆不合或已考讹；①②注等级（C 明书朔；《新唐书》与《旧唐书》同级，《新五代史》与《旧五代史》同条者按旧史计）。"
          "单条与他证冲突与否、最终推步（经例外表）与张表之异，见定朔诸历逐月考证（[dingshuo-zhangbiao-diffs.md](./dingshuo-zhangbiao-diffs.md)，合纪日、写本等判）。"]
    tang_cnt = defaultdict(int)
    for title, name, col, jin_, bu, hcol in tang:
        rs = rows(name)
        # 术推进朔后之朔（「…进朔」列；无则定朔所在日）
        jc = next((k for k in (rs[0] if rs else {}) if "进朔" in k and "限" not in k and not k.startswith("本纪合")), col)

        def tfen(r):
            if r[jc] == r["张表朔"]:
                if r["本纪朔"] == r[jc]:
                    return f"相同·有证（{level(r['书'])}）"
                return "相同·同误（判史书误）" if (r["书"], r["年"], r["月"]) in errata else "相同·同误（未判）"
            if (r["书"], r["年"], r["月"]) in errata:
                return "③无据待核（本纪已考讹）"
            if r["本纪朔"] == r[jc]:
                return f"①史载证张表误（{level(r['书'])}）"
            if r["本纪朔"] == r["张表朔"]:
                return f"②史载证术推误（{level(r['书'])}）"
            return "③无据待核"
        n3 = defaultdict(int)
        seen_ev = set()  # 旧、新唐书等同条互见只算一事
        for r in rs:
            f = tfen(r)
            if f and (r["年"], r["月"], r["本纪朔"]) not in seen_ev:
                seen_ev.add((r["年"], r["月"], r["本纪朔"]))
                kk = f[:5] if f.startswith("相同") else f[0]
                n3[kk] += 1
                tang_cnt[kk] += 1
        L += ["", f"### {title}", "", f"共 {len(rs)} 条：合张表 {sum(int(r['本纪合张表']) for r in rs)}，合进朔 {sum(int(r[jin_]) for r in rs)}，"
              f"合不进朔 {sum(int(r[bu]) for r in rs)}；术推与张表相同 {n3['相同·有证'] + n3['相同·同误']} 事（有证 {n3['相同·有证']}、同误 {n3['相同·同误']}），互异 {n3['①'] + n3['②'] + n3['③']} 事（①{n3['①']}、②{n3['②']}、③{n3['③']}）。逐条见 `data/calendar/{name}`。", ""]
        L += table(["书", "年", "纪年", "月", "本纪朔", "张表朔", "术推朔", "术推进朔", "小余（时）", "日食", "分类", "原文"],
                   [[BOOK.get(r["书"], r["书"]), r["年"], r["纪年"], r["月"], r["本纪朔"], r["张表朔"], r[col], r[jc], r[hcol], r["日食"], tfen(r), r["原文"]]
                    for r in rs if r["本纪合张表"] == "0" or r[jin_] == "0" or (tfen(r) and not tfen(r).startswith("相同·有证"))])
    L += ["", f"诸历合计（各书互见合为一事）：术推与张表相同 {tang_cnt['相同·有证'] + tang_cnt['相同·同误']} 事（有证 {tang_cnt['相同·有证']}、同误 {tang_cnt['相同·同误']}），"
          f"互异 {tang_cnt['①'] + tang_cnt['②'] + tang_cnt['③']} 事（①{tang_cnt['①']}、②{tang_cnt['②']}、③{tang_cnt['③']}）。表中只列本纪不合张表、不合术推或两家互异之条，相同而有证之条不列。"]
    # 七、宝祐四年会天历（check_huitian_1256.py）
    L = blocks["huitian"] = []
    ht = rows("huitian_1256_check.csv")
    if ht:
        L += ["", "## 七、《宝祐四年会天历》（1256）", "",
              "第 1 级资料（当时历日的清钞本，转录 `data/calendar/会天历_1256.csv`）。会天历术不传，开禧为上一部历，上推只作对照。", ""]
        L += table(["月", "会天历一日", "张表", "开禧定朔（上推）", "小余（时）", "进朔限（时）", "开禧进朔"],
                   [[r["月"], r["会天历一日"], r["张表"], r["开禧定朔（上推）"], r["小余（时）"], r["进朔限（时）"], r["开禧进朔"]] for r in ht])
    # 八、《春秋》经文纪日（chunqiu_check.py、chunqiu_jianzheng.py）
    L = blocks["chunqiu"] = []
    cq = rows("chunqiu_check.csv")
    if cq:
        models = [k for k in cq[0] if "·" in k or k == "春秋历"]
        early = {"隱公", "桓公", "莊公", "閔公"}
        L += ["", "## 八、《春秋》经文纪日与春秋历、古六历", "",
              "经文取自维基文库《春秋左氏傳》經节（未对校点校本）。任意干支落入一个月的机会约 49%，这是「在月内」的基线；书朔恰合的偶合机会约 1/60。"
              "春秋历的闰年表是今人排定的，合经文不是独立证据。说明见 [calendar-compendium.md](./calendar-compendium.md) §12。", "",
              "### 汇总", ""]
        L += table(["历", "隐—闵在月内", "僖—哀在月内", "全部", "书朔恰合"],
                   [[m, f"{sum(r[m] != '外' for r in cq if r['公'] in early)}/{sum(r['公'] in early for r in cq)}",
                     f"{sum(r[m] != '外' for r in cq if r['公'] not in early)}/{sum(r['公'] not in early for r in cq)}",
                     f"{sum(r[m] != '外' for r in cq)}/{len(cq)}", f"{sum(r[m] == '朔' for r in cq if r['朔'] == '1')}/{sum(r['朔'] == '1' for r in cq)}"] for m in models])
        jz = rows("chunqiu_jianzheng.csv")
        L += ["", "### 春秋历正月之建（真冬至定子月）", ""]
        dukes = list(dict.fromkeys(r["公"] for r in jz))
        L += table(["公", "年数", "亥", "子", "丑", "寅"],
                   [[d, sum(r["公"] == d for r in jz)] + [sum(r["公"] == d and r["建"] == k for r in jz) for k in "亥子丑寅"] for d in dukes])
        with open(ROOT / "data/phenomena/chunqiu_solar_eclipses_check.csv", encoding="utf-8") as f:
            ec = list(csv.DictReader(f))
        with open(ROOT / "data/phenomena/chunqiu_solar_eclipses.csv", encoding="utf-8") as f:
            for r, src in zip(ec, csv.DictReader(f)):
                r["note"] = src["note"]
        L += ["", "### 日食（曲阜，东经 116.98°、北纬 35.6°）", "", "记录日由春秋历换算，未书干支者取该月朔，这几条不独立。「无日食」指记录日前后无日食。", ""]
        L += table(["公", "年", "公历年", "月", "干支", "书朔", "记录日（儒略历）", "食型", "γ", "曲阜食分", "判定", "附注"],
                   [[r["emp"], r["year_in_era"], r["year"], r["month"], r["ganzhi"], r["position"], r["julian"], r["kind"], r["gamma"], r["site_magnitude_0"], r["verdict"], r.get("note", "")] for r in ec])
        with open(ROOT / "data/phenomena/chunqiu_zhang_3_1.csv", encoding="utf-8") as f:
            z31 = list(csv.DictReader(f))
        L += ["", "### 与张培瑜表 3-1 对照（在书给的日子重算）", ""]
        L += table(["公", "年", "经月", "经干支", "书儒略日", "书干支", "书食分", "本项目判定", "食型", "本项目食分", "差", "春秋历换算同日", "书注"],
                   [[r["公"], r["年"], r["经月"], r["经干支"], r["书儒略日"], r["书干支"], r["书食分"], r["本项目判定（书之日）"], r["食型"], r["本项目食分"], r["差"], "同" if r["同日"] == "1" else "异", r["书注"]] for r in z31])
        L += ["", "### 逐条（朔：恰为朔日；内：在月内；外：不在月内）", ""]
        pick = ["春秋历", "周·建子", "周·建丑", "鲁·建子", "鲁·建丑", "殷·建子"]
        L += table(["公", "年", "月", "干支", "书朔", "日食"] + pick + ["原文"],
                   [[r["公"], r["公年"], r["月"], r["干支"], "朔" if r["朔"] == "1" else "", "食" if r["日食"] == "1" else ""] + [r[m] for m in pick] + [r["原文"]] for r in cq])
    L = blocks["zuozhuan"] = []
    zz = rows("zuozhuan_check.csv")
    if zz:
        L += ["", "## 九、《左传》传文纪日与春秋历", "", "偏 k：传文 m 月当春秋历 m+k 月。说明见 [calendar-compendium.md](./calendar-compendium.md) §12。偏 0 与偏 +2 几乎同时合或不合，不能分辨岁首差。", ""]
        L += table(["公", "年", "月", "干支", "书朔", "同经", "偏-1", "偏0", "偏+1", "偏+2", "原文"],
                   [[r["公"], r["公年"], r["月"], r["干支"], "朔" if r["朔"] == "1" else "", "同" if r["同经"] == "1" else "", r["偏-1"], r["偏+0"], r["偏+1"], r["偏+2"], r["原文"]] for r in zz])
    L = blocks["wangtao"] = []
    wc = rows("wangtao_compare.csv")
    if wc:
        L += ["", "## 十、经传纪日在张培瑜春秋历与王韬谱间判定不同者", "", "王韬逐月朔由正朔均分取整，可差 ±1 日。说明见 [calendar-compendium.md](./calendar-compendium.md) §12。", ""]
        L += table(["文", "公", "年", "月", "干支", "只合", "原文"], [[r["文"], r["公"], r["公年"], r["月"], r["干支"], r["合"], r["原文"]] for r in wc])
    # 专题各节按所涉时代先后排列（以起始年代为序）：先秦经传 → 唐至明定朔诸历（627 起）→ 敦煌历日 → 辽 → 金 → 宝祐会天历
    L = head
    for name in ["chunqiu", "zuozhuan", "wangtao", "tang", "dunhuang", "liao", "jin", "huitian"]:
        L += blocks.get(name, [])
    num = "一二三四五六七八九十"
    n = iter(range(100))
    cn = lambda k: num[k] if k < 10 else "十" + num[k - 10]
    L = [re.sub(r"^## [一二三四五六七八九十]+、", lambda m: f"## {cn(next(n))}、", x) if x.startswith("## ") else x for x in L]
    (ROOT / "doc/calendar/benji-checks.md").write_text("\n".join(L) + "\n", encoding="utf-8")
    print("doc/calendar/benji-checks.md", len(L), "行")


if __name__ == "__main__":
    main()
