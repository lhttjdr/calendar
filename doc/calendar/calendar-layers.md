# 历法域内部结构（四层）

本文只讲**历法域**（`rust/core/src/calendar/`）内部的依赖层次与对应数据；三大域（天文/历法/民俗）之间的关系见
[doc/architecture/three-domains.md](../architecture/three-domains.md)，本文是该文档"历法域"一节的展开。

`rust/core/src/calendar/` 下每个文件属于哪一层，**改动该目录下代码前务必先对照下表确认**：

| 层 | 内容 | 文件 |
|----|------|------|
| 1 独立基础层（不依赖天文） | JD↔儒略历/格里高利历；各朝代古历分数推步（平朔平气，仅作验证）；干支纯循环命名；历法换算选项 | `gregorian.rs`、`ancient_restore.rs`、`push_calendar.rs`、`daming_li.rs`、`gan_zhi.rs`、`options.rs` |
| 2 聚合/实算层（依赖天文的朔、气） | 现算或查表得到某历法某农历年的岁数据（朔+中气） | `chinese_lunar.rs`（现算真朔真气）、`historical.rs`（查历史朔日表）；干支历"节气派"（`gan_zhi_options.rs` 一支） |
| 3 派生层（依赖第 2 层产出） | 政权·帝王·年号纪年；干支历"农历派" | `era_names.rs`、`era_gaiyuan.rs`、`month_labels.rs`；干支历"农历派"（`gan_zhi_options.rs` 另一支） |
| 4 仲裁/路由层 | 某农历年该用哪种历法实现；实历覆盖范围内是否优先用实历 | `lifa.rs`（`LifaRegistry`）；`convert.rs`（"优先实历、缺省退回定气定朔"判断，见下方「已知问题」） |
| 契约层（不计入四层） | `CalendarSystem` trait，JD↔各历法日期互转的通用接口，被第 1/2 层各历法系统实现 | `system.rs` |

---

## 第 1 层：独立基础层

**概念**：不查天文、纯算术或纯配置，输入只有日期整数/儒略日和历法自身的常数。

**包含**：
- **格里高利历**（及此前之儒略历）：公历体系，1582 年前儒略历、其后格里高利历。
- **古历复原**（`ancient_restore.rs`）：分数推步（`BigRational`），平朔平气、历谱生成；**仅作验证古历用**，不作为实际换算依据（与下面"实历优先"的换算路径是两条不同路径，见 [ancient-calendar-restore.md](ancient-calendar-restore.md)）。
- **推步历**（`push_calendar.rs`）：各政权历法（蜀四分、吴乾象等）的平朔+无中气置闰实现，算法与参数见 [push-algorithms.md](push-algorithms.md)。
  定朔历金重修大明历（`daming_li.rs`，有理数推步定朔、进朔）经同一政权表（`regime_calendars`）导出，见 [historical-calendar.md](historical-calendar.md)「金大明历」。
- **干支纯循环命名**（`gan_zhi.rs`）：甲子…癸亥 60 日循环，只依赖儒略日整数，不依赖任何历法边界规则（边界规则在第 2/3 层的 `gan_zhi_options.rs`）。

**对应数据**：
- **历法总表**：`data/calendar/汇总/lifa_summary.csv` — 历法 id、历名（name_simp）、起止年（start_ce/end_ce，公历年份，负为公元前）、制作者（maker_regime, maker_person）、锚定说明（anchor_note）、来源（source）。含格里高利历、儒略历、现代农历及《历法通志》所载太初历、四分历、乾象历、景初历、大明历、授时历、大统历、时宪历等主要历法。命名为 `_summary` 以区别于下面的原书细表（本项目整理的精选清单，非某一来源的原样转录）。
- **历法通志细表**：`vendor/calendar/lifa_tongzhi/lifa_tongzhi.csv` — 历名·治历者·岁实·朔策·成历之年·公元·备注（摘自朱文鑫《历法通志》）；与 `lifa_summary.csv` 可互参，细目以通志为准。

---

## 第 2 层：聚合/实算层

**概念**：依赖天文域产出的真朔、真气（见 [三域文档](../architecture/three-domains.md) 天文域第 3 层），或依赖史书还原好的静态朔日表，得到某历法某农历年的岁数据（14 朔 + 12 中气）。

**两种实现方式，角色相同、来源不同**：
- **现算**（`chinese_lunar.rs`）：直接调用天文域算真朔真气，精度最高，覆盖任意年份（含未来）。
- **查表**（`historical.rs`）：解析 `vendor/calendar/chinese_calendar/实历朔日表.csv`，为**主表**（历法 id 见 `lifa::LIFA_ID_ZHONGYUAN_SHILI`）提供岁数据。主表 = 《三千五百年历日天象》按各朝代颁行历法据历还原、推步得出的连续历日，书中并无名为「中原正朔」的单一历法。**历史区间应优先使用此查表结果**而非现算——无论天文计算多精准，历史日期已成事实，只有实历表才是历史区间的唯一准确来源。

**干支历"节气派"**（`gan_zhi_options.rs` 一支）：年界=立春交节、月界=十二节交节，只依赖气，不依赖朔、不依赖农历，因此和 `chinese_lunar.rs` 同属这一层而非下一层。

**对应数据**：
- 朔日表：`vendor/calendar/chinese_calendar/实历朔日表.csv` — 主表每月朔的儒略历/格里高利历日期（按各朝颁历推步得出）。

---

## 第 3 层：派生层

**概念**：依赖第 2 层的产出（农历年数、朔日、正月初一），不直接依赖天文。

**包含**：
- **历史月序改制**（`month_labels.rs`）：主表（第 2 层）一律按建寅编月；新莽、魏景初建丑，武周、唐肃宗末建子，当时月名不同（武周有「腊月」「一月」，肃宗以「建子月」等斗建为名）。本模块按政权把主表月换成当时月名、或反向，只做命名，不改朔日。数据 `data/calendar/month_label_reforms.csv`，史料与规则见 [historical-calendar.md](./historical-calendar.md)「月序改制」。
- **政权·帝王·年号纪年**（`era_names.rs`+`era_gaiyuan.rs`）：依附在历法之上的纪年体系，按政权划分。同一时刻在不同政权下可有不同「朝代·帝王·年号·年数」，同一年可能有多个并存政权（如三国期间魏/蜀/吴），[`EraNames::get_all`] 返回全部，[`EraNames::get_primary`] 返回"正统主线"一条。
- **干支历"农历派"**（`gan_zhi_options.rs` 另一支）：年界=正月初一零点、月界=每月初一零点，依赖第 2 层算出的朔/正月边界。

**对应数据**：
- 年号表：`data/calendar/汇总/era_names_merged.csv` — 跨来源交叉校对出的汇总表（主线来自 `vendor/calendar/chinese_calendar/年号表.csv`，并存政权补充自 `data/calendar/fang_shiming/` 转录），字段：`lunar_year, regime, emperor, era_name, era_year, is_primary, has_gaiyuan, gaiyuan_prev_era_name, gaiyuan_prev_era_year, gaiyuan_month, source, note`；主表对应历法 id `lifa::LIFA_ID_ZHONGYUAN_SHILI`。表结构与汇总方法见 `data/calendar/汇总/README.md`。

---

## 第 4 层：仲裁/路由层

**概念**：面对"同一件事有多种实现"时决定选哪个，职责是**选择**不是**计算**。

**包含**：
- **`lifa.rs`（`LifaRegistry`）**：解析历法总表，按时代/政权判断某个历史时期该用哪部历法。
- **`convert.rs`**：对外的"公历↔农历"换算入口，内含"实历覆盖范围内优先用实历（查 `historical.rs`），缺省退回定气定朔现算（`chinese_lunar.rs`）"的判断。

### 已知问题（写本文档时定位到，尚未修复）

`historical.rs` 和 `convert.rs` 把"仲裁"和"数据供给/计算"两个职责混在同一个文件甚至同一个函数里：

- `HistoricalCalendar`（`historical.rs`）既负责"给定农历年，返回该年朔日岁数据"（第 2 层的数据供给角色），也隐含"这个农历年在不在实历覆盖范围内"的判断（第 4 层的路由职责），两者没有分开命名——调用方拿到 `None` 时无法区分"这年没有实历数据"和"这年本来就不该用实历"。
- `convert.rs` 里"实历优先、缺省退回定气定朔"这条规则直接写死在换算函数内部，不是一个可以单独查阅、复用、调整优先级的具名决策点。

这与天文域早前发现的问题同构（见 [三域文档](../architecture/three-domains.md) 天文域案例）：仲裁逻辑不独立成第一等公民，就会散落进数据供给代码里，后续想调整"哪年该用哪种实现"的策略就要满文件找。是否已修复、修复方式见对应 commit / `lifa.rs` 当前实现。

---

## 关系小结

| 层次 | 概念 | 锚定 | 对应数据 |
|------|------|------|----------|
| 1 独立基础层 | 儒略历/格里高利历、古历推步、干支循环命名 | JD / 儒略历+格里历 | `data/calendar/汇总/lifa_summary.csv`、`vendor/calendar/lifa_tongzhi/lifa_tongzhi.csv` |
| 2 聚合/实算层 | 岁数据（现算或查表） | JD / 儒略历+格里历 | `vendor/calendar/chinese_calendar/实历朔日表.csv` |
| 3 派生层 | 政权·帝王·年号、历史月名、干支历农历派 | 农历年 / JD / 儒略历+格里历 | `data/calendar/汇总/era_names_merged.csv`、`data/calendar/month_label_reforms.csv` |
| 4 仲裁/路由层 | 历法总表路由、实历优先判断 | — | `data/calendar/汇总/lifa_summary.csv` |

第 3 层依赖第 2 层：某政权的「某年某月某日」由该政权所采用的历法（第 1/2 层）与朔日/推步决定，再落到儒略日或公历上。第 4 层横切前三层，不是被依赖，而是决定"该走第 2 层的哪种实现"。

**数据字段/来源细节**（各 CSV 的具体列、生成脚本、上游出处）见 [../../data/calendar/README_shuori.md](https://github.com/lhttjdr/calendar/blob/master/data/calendar/README_shuori.md) 与 [../../data/calendar/SOURCES_era_regime.md](https://github.com/lhttjdr/calendar/blob/master/data/calendar/SOURCES_era_regime.md)；[../../data/calendar/汇总/README.md](https://github.com/lhttjdr/calendar/blob/master/data/calendar/汇总/README.md) 另有 `era_names_merged.csv` 的汇总方法说明。
