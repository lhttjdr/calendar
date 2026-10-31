# 实历朔日数据（历史朔日）与年号表

本文档说明的数据文件实际存放在 `vendor/calendar/chinese_calendar/`（原样提取自 ytliu0/ChineseCalendar，不重排/不改动），包含两类数据：

1. **朔日**：`vendor/calendar/chinese_calendar/实历朔日表.csv` / `.json` — 实历每月初一对应的儒略历/格里高利历日期。  
2. **年号**（单独成表）：`vendor/calendar/chinese_calendar/年号表.csv` / `.json` — 按农历年一行，政权·帝王·年号（属历法域第 3 层「派生层」，见下）。

**历法域四层结构**：朔日属第 2 层「聚合/实算层」，年号属第 3 层「派生层」，详见 [doc/calendar/calendar-layers.md](../../doc/calendar/calendar-layers.md)。上述朔日与年号对应**主表**（见下）。

## 主表来源与含义

（本仓库其余文档提到"据历还原"/"中原正朔"时均指向本节，不再重复展开）

《三千五百年历日天象》**并无一种名为「中原正朔」的历法**。该书采用**「据历还原」**：按各朝代实际颁行历法（太初历、四分历、景初历、元嘉历、大明历、授时历、大统历、时宪历等），用该历法的**推步公式**逐段推算历日，拼接成一条连续历日表；本仓库**主表**即指这条连续表（ytliu0/ChineseCalendar 的 `ChineseToGregorian()` 等），习惯上可称其对应「中原政权更替下的颁历序列」，年号表按年体现汉→魏→晋→…→清等政权。

**重要**：各朝代/政权历法不同（如三国魏蜀吴、南北朝多政权、宋辽金元、南明与清等），**有历史朔也不能唯一「算出」农历月**，必须明确是**哪一套历法**——主表只覆盖上述"中原政权更替下的颁历序列"这一条线，蜀历、吴历、北朝各历、南明历等**未包含**在内。**配合各朝代/政权的推步算法**（见 [doc/calendar/historical-calendar.md](../../doc/calendar/historical-calendar.md)、[doc/calendar/push-algorithms.md](../../doc/calendar/push-algorithms.md)）即可按政权计算：主表用本实历，其它政权用对应历术推步。上游库具体是如何支持"各朝代不同历法"的、以及三国魏蜀吴历法差异的详细说明，见下「各朝代历法差异」一节。

## 数据来源

- 项目：[ytliu0/ChineseCalendar](https://github.com/ytliu0/ChineseCalendar)（公历与农历日期对照，722 BCE–2200 CE）
- 朔日：提取自 `src/calendarData.js` 的 `ChineseToGregorian()` 表；该书历表按各朝代颁行历法**据历还原**推步编制（见上「主表来源」）。
- 年号：来自同仓库 `src/eras.js` 的 `eraName`（繁体）/ `eraNameSim`（简体），当前仅 **default** 区域（按年对应汉/魏/晋/…/清等政权）。

**三国 220–280 年**：原站魏蜀吴历法不同，各有朔闰表；该段主表朔日与年号对应**魏**颁历（据历还原），蜀、吴历与年号未导出。整体主表按年对应中原各政权（汉、魏、晋、…）。详见下「各朝代历法差异」中「三国魏蜀吴历法差异」小节。

## 各朝代历法差异

是的，[ytliu0/ChineseCalendar](https://github.com/ytliu0/ChineseCalendar) **在不同朝代/政权下会用到多套历法**，但实现方式分为两类：**预存实历表** 与 **按历术推步计算**。

### 两种实现方式

**方式一：一张主表（实历考证）**
- **来源**：`calendarData.js` 的 `ChineseToGregorian()`；《三千五百年历日天象》按各朝颁历**据历还原**、推步编制历表。
- **覆盖**：约 -105 ～ 2200 年，**一套**朔闰序列（每月初一、闰月、年日数等）。
- **含义**：对每个时代取「一种」主流或正朔历（例如东汉至曹魏用中原颁历，明清用大统历/清历），**不区分同一时代并存的多个政权历法**。
- 本仓库的**实历朔日表**即仅从此表导出，故只有这一套。

**方式二：按政权/历术用不同算法（推步）**
- **来源**：`split.js`、`ancientCalendars.js` 等，根据 **region**（政权/视角）或 **历术名 (li)** 选不同参数或不同函数，**现场算**朔日。
- **含义**：同一公历年内，魏/蜀/吴、南朝/北朝、宋/辽/金/元、清/南明等可以**各用一套历**，朔日、闰月可能不同。
- 网页上通过「选择政权/历法」切换（如魏/蜀/吴、南北朝各朝、南明等），显示的就是不同历法的结果。

### 库中涉及的历法/政权（按时期）

| 时期 | 历法/政权 | 实现方式 | 说明 |
|------|-----------|----------|------|
| 春秋 722–481 BCE | 周历、鲁(春秋)历 | 推步 | `chunqiu_cmonth`, 古六历等 |
| 战国 480–222 BCE | 周、鲁、颛顼(秦) | 推步 | `guliuli_calendar_cmonth(li, y, jdc)` |
| 秦～汉初 -221～-104 | 颛顼历/秦与汉初历 | 推步 | `HanZhuanXu`、秦汉历复原 |
| -105～2200 | **主表（各朝据历还原推步）** | **预存表** | `ChineseToGregorian()`，本仓库已导出；"据历还原"含义见上「主表来源与含义」 |
| 三国 220–280 | **魏** | 主表 | 同上；三国三方历法差异详见下「三国魏蜀吴历法差异」 |
| 三国 221–263 | **蜀** | 推步 | `cmonthDate_Shu`，**四分历** (Sifen) |
| 三国 222–280 | **吴** | 推步 | `cmonthDate_Wu`，**乾象历** (Qianxiang) 等 |
| 晋～隋 384–580 | 后秦、北凉、北魏/东魏/西魏/北齐/北周 | 推步 | 三纪、宣时、景初、玄始、正光、明克让、天和、大象、开皇等 |
| 宋辽金元 947–1367 | 宋 vs 辽金元 | 推步/表 | `cmonthDate_LiaoJinYuan` 等，历法差异 |
| 南明/明郑 1645–1683 | 南明 vs 清 | 推步/表 | `cmonthDate_southernMing`，大统历 vs 清历 |
| 1912 以后 | 现代农历 | 推步 | 按现行规则计算 |

（上表只列代表性时期，中间未写的年份一般用主表或同一逻辑。）

### 年号与历法的对应

`eras.js` 里按 **region** 切换同一年的年号，与上述历法对应，例如：

- `region='default'` → 主表年号（汉/魏/晋/宋/…/清 等中原颁历政权）
- `region='Shu'` → 蜀年号
- `region='Wu'` → 吴年号
- `region='LaterQin'`、`'NorthernLiang'`、`'WeiZhouSui'`、`'WeiQi'`、`'LiaoJinYuan'`、`'SouthernMing'` 等 → 各政权年号

所以：**同一公历/农历年，不同政权可以对应不同历法 + 不同年号**。

### 小结

- **是的，该库实现了各朝代/政权不同的历法**：一部分用**一张主表**（实历考证，本仓库已导出）；另一部分按**政权/历术**用**不同算法或参数**推步（蜀、吴、南北朝、辽金元、南明等）。
- 本仓库当前数据 = **主表 + default 年号**（主表 = 各朝颁历据历还原推步的连续表）；未导出蜀历、吴历、南北朝多政权历、南明历等。
- 若需某政权历的朔日或年号，需在原库中切到对应 **region**（及必要时 **li**）再计算或导出。

### 三国魏蜀吴历法差异

220–280 年（魏蜀吴时期）三国**历法不同**：朔日、闰月可能不一致，同一天在魏是初一在蜀可能是初二或三十。本节说明原站数据源如何体现这种差异，以及当前本仓库导出数据对应哪一套历——是上表"三国"三行的展开。

**在 ChineseCalendar 原项目中的体现**：原站「魏蜀吴朔闰表」页 ([table_period.html?period=weishuwu](https://ytliu0.github.io/ChineseCalendar/table_period.html?period=weishuwu)) 上，**分三张表**分别给出三种朔日：

| 政权 | 时间范围 | 朔日来源 | 历法 |
|------|----------|----------|------|
| **魏** | 220–265 | `ChineseToGregorian()` 主表（预存实历表） | 魏颁历（据历还原推步，见上「主表来源与含义」） |
| **蜀** | 221–263 | **推步计算** `cmonthDate_Shu()` | **四分历** (Sifen) |
| **吴** | 222–280 | **推步计算** `cmonthDate_Wu()` | 222 年前四分历，222 年起 **乾象历** (Qianxiang)，含 244、247 年两处人工修正 |

- 魏：朔日来自 `calendarData.js` 的**预存实历表**，与史书/历日考证一致。
- 蜀、吴：朔日**不来自**同一张表，而是 `split.js` 里用**历法参数 + 平朔/无中气规则**算出来的，因此同一农历年在三张表里朔日、闰月可能不同。
- 年号：`eras.js` 中按 `region`（`'default'` / `'Shu'` / `'Wu'`）分别列出魏、蜀、吴年号，与三套历法对应。

网页上通过「魏 / 蜀 / 吴」切换，显示的就是上述三套不同的朔闰表；另有「[魏蜀吴朔闰异同表](https://ytliu0.github.io/ChineseCalendar/ThreeKingdoms_calendars_simp.html)」对比三历差异。

**当前本仓库导出数据的含义**：实历朔日表与年号表仅从以下两处提取——朔日：`ChineseToGregorian()`（即**魏历/主表**）；年号：`eraName(..., 'default', ...)`（即**魏**年号）。因此：

- **朔日**：220–280 年对应的是**魏历**朔日，**不包含**蜀历、吴历的朔日。
- **年号**：为**魏**的年号（如「魏文帝黃初」「魏明帝景初」等），**不包含**蜀、吴的纪年（如「蜀後主建興」「吳大帝黃龍」等）。

若要得到蜀、吴的朔日或年号，需要在原仓库中按 `region='Shu'` / `region='Wu'` 使用相应逻辑（蜀/吴朔日用 `cmonthDate_Shu` / `cmonthDate_Wu` 计算，年号用 `eraName(..., 'Shu')` / `eraName(..., 'Wu')`）另行导出。

**小结（三国）**：

| 项目 | 魏 | 蜀 | 吴 |
|------|----|----|-----|
| 原站朔闰表 | 有（主表） | 有（四分历推步） | 有（乾象历推步） |
| 本仓库朔日数据 | ✅ 有（即当前 shuori） | ❌ 未导出 | ❌ 未导出 |
| 本仓库年号数据 | ✅ 有（即当前年号表） | ❌ 未导出 | ❌ 未导出 |

**三国历法不同**在原站是通过「魏 / 蜀 / 吴」三套朔闰表 + 三套年号（按 region）体现的；本仓库目前只导出了**魏历朔日 + 魏年号**，未单独导出蜀、吴历的朔日与年号。

## 时间范围

- **农历年**：约 -105（前106年）至 2200 年  
- **儒略历/格里高利历**：1582 年前为儒略历，1582 年及后为格里高利历；公元前年份以负数表示（如 -105 即 106 BCE）。

## 文件格式

### 朔日：`vendor/calendar/chinese_calendar/实历朔日表.csv` / `.json`

| 列名 | 含义 |
|------|------|
| gregorian_year | 儒略历/格里高利历年（负数=公元前） |
| gregorian_month | 月 (1–12) |
| gregorian_day | 日 |
| lunar_year | 农历年（按建寅岁首） |
| lunar_month | 农历月 (1–12)，**一律按建寅（夏正）编号** |
| is_leap | 是否闰月 (0=否, 1=是) |

每行一条**朔日**记录，表示该农历月（或闰月）的初一落在的儒略历/格里高利历日期。1582 年前为儒略历，之后为格里高利历。**不含年号**，年号见年号表 `vendor/calendar/chinese_calendar/年号表.csv`。

新莽、魏景初（建丑）与武周、唐肃宗末（建子）当时所用月名与本表的建寅月序不同，换算见下节 `month_label_reforms.csv`。

### 上游推步政权朔日与年号：`vendor/calendar/chinese_calendar/分政权朔日表.csv`、`分政权年号表.csv`

由 `scripts/calendar_data/extract_region_calendars_from_chinese_calendar.js` 运行上游 ytliu0/ChineseCalendar（commit `719e02ff`，
2026-10-16；GPL-3.0，只运行导出数据、不复制代码）的 `setup_region_calendar`、`eraName`/`eraNameSim` 导出，2026-10-27 取。
- `分政权朔日表.csv`：region（上游名：Shu、Wu、LaterQin、NorthernLiang、WeiZhouSui、WeiQi）, gregorian_year/month/day（民用历）,
  lunar_year, lunar_month, is_leap。含上游人工修正（430 年二月；吴 244 年十二月、247 年九月），照《三千五百年历日天象》附表 2、3，该书不注逐条依据（见 `doc/calendar/push-algorithms.md` 3.1）。
- `分政权年号表.csv`：region, lunar_year, era_trad, era_simp。
用途：与本项目推步结果（下节）逐月比对，见 `push_calendar::tests::push_matches_upstream_export_except_known_corrections`。

### 出土与石刻纪日：`data/calendar/出土纪日.csv`

买地券、石塔题记、墓志、砖铭等当时人所记的历日，是独立于各复原表的一手证据。`scripts/calendar_data/check_dated_inscriptions.py`
逐条对主表、本项目推步、上游推步核对（不合即非零退出）。

| 列名 | 含义 |
|------|------|
| regime | 政权 |
| table | 该政权应对照的历表：主表、或推步政权名 |
| text | 纪日原文 |
| lunar_year, month, is_leap, day | 岁、月、闰否、日序 |
| day_ganzhi, shuo_ganzhi | 日干支、朔干支（铭文未记朔者空） |
| object, source, checked | 器物、出处（转录来源）、核验状态（如「转录未核拓片」） |

### 推步政权朔日：`data/calendar/推步政权朔日表.csv`

（「推步」「实历」等术语与证据层级见 `doc/calendar/calendar-compendium.md`。）

主表以外、按推步得出的各政权逐月朔日：蜀（四分历 221–263）、吴（乾象历 222–280）、后秦（三纪历 384–416）、北凉（玄始历 398–439）、
北魏—西魏—北周—隋（景初、玄始、正光、明克让、天和、大象、开皇历 398–590）、东魏—北齐（正光、兴和、天保历 534–577）、金（重修大明历定朔、进朔 1137–1234，1182 年前杨级历以之代推）。
由 `rust/core/examples/export_regime_calendars.rs` 从 `calendar::push_calendar`（平朔、平气、无中气置闰，见
`doc/calendar/push-algorithms.md`；金为定朔、恒气置闰，`calendar::daming_li`，见 `doc/calendar/historical-calendar.md`「金大明历」）
导出；本项目推算所得，不是上游原表。

| 列名 | 含义 |
|------|------|
| regime | 政权（如「北魏/西魏/北周/隋」「东魏/北齐」） |
| calendar | 该岁所用历术 |
| gregorian_year, gregorian_month, gregorian_day | 朔日（民用历：1582 年前儒略历） |
| lunar_year, lunar_month, is_leap | 该政权的岁、月（建寅）、闰否 |

### 月序改制：`data/calendar/month_label_reforms.csv`

由 `scripts/calendar_data/build_month_label_reforms.py` 按史志规则从实历朔日表生成；只列当时月名与建寅月名不同的月份。
Rust 侧 `calendar::month_labels`，Python 侧 `scripts/calendar_data/gaiyuan_date.py` 的 `table_month`。规则与史料见
`doc/calendar/historical-calendar.md`「月序改制」。

| 列名 | 含义 |
|------|------|
| regime | 政权（新、魏、武周、唐） |
| period | 时期名，如「魏景初建丑」 |
| table_lunar_year, table_lunar_month, table_is_leap | 实历朔日表中的月（建寅） |
| label_year | 当时名义岁（天文纪年，按该政权岁首） |
| label_month | 当时月名：正月…十二月、腊月、一月、后十二月、建子月…建巳月 |
| label_is_leap | 是否闰月 |

### 年号单独表：`vendor/calendar/chinese_calendar/年号表.csv`（6 列基础 + 改元 2 列）

| 列名 | 含义 |
|------|------|
| lunar_year | 农历年 |
| has_gaiyuan | 是否改元年（1=该年多个年号，0=单一年号） |
| regime_simp | 政权（简体），如 汉、魏、五代、后梁、宋 |
| emperor_simp | 帝号/庙号（简体），如 武帝、后梁太祖、太祖（不含政权前缀） |
| era_name_simp | 年号名（简体），如 元封、元凤、开平 |
| era_year_simp | 年数（简体），如 五年、元年、四年 |
| gaiyuan_prev_era_name_simp | 改元前年号名（仅改元年有值），如 始元 |
| gaiyuan_prev_era_year_simp | 改元前年数（仅改元年有值），如 七年 |

**无繁体、无合并字段**。改元年时主年号取当年新号（如 元凤元年），改元前列存旧号（如 始元七年）；展示格式：**政权 · 帝号 改元前年号年数/当年年号年数 月日**（如 汉 · 昭帝 始元七年/元凤元年 正月…）。非改元年仅用主年号。  
**说明**：各政权朔闰与历法由**推步**计算（见 [doc/calendar/push-algorithms.md](../../doc/calendar/push-algorithms.md)）；主表年号按年体现中原政权更替。从带 era_simp 的源表生成本表见 `scripts/calendar_data/refactor_era_names_csv.mjs`；裁成 6/8 列见 `scripts/calendar_data/era_names_to_base_csv.mjs`。**补全与校勘**见 [SOURCES_era_regime.md](SOURCES_era_regime.md)。历法名、治历者、岁实及成历之年与公元对照见 [历法通志_README.md](../../vendor/calendar/lifa_tongzhi/历法通志_README.md)（摘自朱文鑫《历法通志》）。

### 改元候选表：`vendor/calendar/chinese_calendar/改元候选表.csv` / `.json`

仅包含 `年号表.csv` 中 `has_gaiyuan=1` 的年份（一年内发生改元、存在多个年号），便于直接查"一年多号"的年份，列为：

| 列名 | 含义 |
|------|------|
| lunar_year | 农历年 |
| era_trad | 年号（繁体），改元前/改元后年号以 `/` 连接，如"(漢昭帝始元七年/元鳳元年)" |
| era_simp | 年号（简体），格式同上 |

**注意**：本表（以及 `年号表.csv`）仅包含**某农历年对应哪个/哪些年号**，**不包含改元发生的具体日期**——即无法从本数据得知"哪天"改元。若要精确到「哪天改元」，需另查：

1. **历日文献**：《三千五百年历日天象》（张培瑜等）等朔闰表 + 史书改元记载；历日简牍、大统历日汇编等原始历书。
2. **纪年/年号工具**：[中研院两千年中西历转换](http://sinocal.sinica.edu.tw/)（可查某日对应的年号）、[上海图书馆·中国历史纪年表](https://data.library.sh.cn/dynasty/main)、李崇智《中国历代年号考》、邓洪波《东亚历史年表》等。
3. **史书与考证**：改元多记载在正史本纪、实录中，具体日期需按儒略历/格里高利历与农历对照后逐条核对。

### JSON 文件

朔日、年号均为同结构对象数组，便于程序读取。

## 重新生成

需先克隆 ChineseCalendar 仓库，再执行：

```bash
# 克隆源仓库（若尚未克隆）
git clone https://github.com/ytliu0/ChineseCalendar.git /tmp/ChineseCalendar

# 指定数据文件路径并运行提取脚本
CALENDAR_DATA_PATH=/tmp/ChineseCalendar/src/calendarData.js node scripts/calendar_data/extract_shuori_from_chinese_calendar.js
```

也可将 ChineseCalendar 放在本项目旁目录 `ChineseCalendar/`，不设置环境变量直接运行上述 `node` 命令（脚本会默认查找 `../ChineseCalendar/src/calendarData.js`）。

## 相关代码实现

本仓库 Rust 核心库中解析、使用这两张表的**实历模块**与**年号模块**，其接口、设计原则与用法见
[doc/calendar/historical-calendar.md](../../doc/calendar/historical-calendar.md)，不在本文重复。


## 本纪纪日（`本纪纪日/<书>.csv`）

`scripts/calendar_data/qinhan_eval/zhi_star_records.py` 的纪日模式（`date_sections`，每卷指定政权）从本纪抽取带日干支的记事日期：
《三国志》（`sanguozhi`）、《晋书》（`jinshu_benji`）、《宋书》（`songshu_benji`）、《南齐书》（`nanqishu_benji`）、《梁书》（`liangshu_benji`）、
《陈书》（`chenshu_benji`）、《魏书》（`weishu_benji`）、《北齐书》（`beiqishu_benji`）、《周书》（`zhoushu_benji`）、《隋书》（`suishu_benji`）、
《旧唐书》（`jiutangshu_benji`）、《新唐书》（`xintangshu_benji`）、《旧五代史》（`jiuwudaishi_benji`）、《新五代史》（`xinwudaishi_benji`）、
《宋史》（`songshi_benji`）、《辽史》（`liaoshi_benji`）、《金史》（`jinshi_benji`）、《元史》（`yuanshi_benji`）、《明史》（`mingshi_benji`）。
列：regime、era、year_in_era、year（天文纪年）、month（所记月序，闰月前加「闰」）、month_name、leap、ganzhi、calendar（该政权所用历表）、
in_month（日干支在该历所记之月）、jdn、julian、position（原文明书「朔」者为「朔」）、text。`<书>_閏月.csv` 记本纪「閏月」及其前所记之月
（leap_after）。用于 `scripts/calendar_data/discriminating_records.py` 以一手日期检验主表与各推步表（结论见 `doc/calendar/push-algorithms.md`
§3.3—3.6），及 `scripts/calendar_data/verify_main_table.py` 以明书朔日与閏月位置核验主表（见下）。本纪月份更替常不书，in_month = 0 者多为此，
非历表之误（在各表下同样不在月）；《旧唐书》本纪略书月份尤多。

## 主表核验（`main_table_check.csv`、`main_table_leaps.csv`、`main_table_shuo_sky.csv`、`main_table_shuo_baseline.csv`）

- `main_table_check.csv`：本纪明书朔日逐条（书、朝代、政权、年、纪年、月、本纪朔、主表〔合或差几日〕、主表朔/本纪朔 JDN、类〔朔差一二日、
  差一月、其他〕、日食、本项目推步、上游推步、原文、旁证〔他书同月记日落在主表该月/本纪朔起 30 日内者〕、他书同记）。
- `main_table_leaps.csv`：本纪「閏月」位置与主表闰位（合、略书相容、矛盾）。
- `main_table_shuo_sky.csv`：朔差一二日者的实际合朔（DE441，都城地方平时）与判定；`main_table_shuo_baseline.csv`：相合者朔日与合朔日之差的分布
  （唐代朔在合朔次日者为进朔）。
- 由 `verify_main_table.py`、`scripts/phenomena/main_table_shuo_sky.py` 生成，`main_table_checks_to_md.py` 汇为
  `doc/calendar/main-table-checks.md`；方法与结论见 `doc/calendar/historical-calendar.md`「以本纪核验主表」。


## 敦煌历日（`敦煌历日.csv`）

敦煌出土北魏太平真君十一年、十二年（450、451）历日逐月（月大小、朔干支、建除、历注节气），据邓文宽《敦煌天文历法文献辑校》（1996）
书页 101—103 录文转录（原卷敦煌研究院藏；451 年十至十二月原卷残缺，据苏、刘两钞本补）。实物历日，为当时行用之历的一手证据。
比对见 `scripts/calendar_data/check_dunhuang_calendar.py` 与 `doc/calendar/push-algorithms.md` §3.5。

## 敦煌历日比较表（`敦煌历日_比较表.csv`、`dunhuang_vs_main.csv`）

邓文宽《敦煌天文历法文献辑校》（1996）附录一「中原曆、敦煌具注曆日比較表」逐格转录：北魏 450—451 与唐元和三年（808）至宋淳化四年
（993）共 35 年、447 行（书页 701—735，PDF 页 = 书页 + 36）。列：文书、纪年、公元、行月序（中原曆月序）、中原大小、中原朔、敦煌月序
（与中原不同时）、敦煌大小、敦煌朔、二历相差、备注、书页；原书括号格（据残历条件推算）照录括号，「?」为月大小不明。中原曆一栏据陈垣
《廿史朔闰表》（复原），与主表逐格全合；敦煌曆一栏为出土历日（当地自造之历）。`scripts/calendar_data/check_dunhuang_comparison.py`
自校（月大小与相邻朔差全合）并写 `dunhuang_vs_main.csv`（敦煌有据之月与主表逐月日差），汇入 `doc/calendar/main-table-checks.md` 第三节。
