# 历法域内部结构（四层）

本文只讲**历法域**（`rust/core/src/calendar/`）内部的依赖层次与对应数据；三大域（天文/历法/民俗）之间的关系见
[doc/architecture/three-domains.md](../architecture/three-domains.md)，本文是该文档"历法域"一节的展开。

`rust/core/src/calendar/` 下每个文件属于哪一层，**改动该目录下代码前务必先对照下表确认**：

| 层 | 内容 | 文件 |
|----|------|------|
| 1 独立基础层（不依赖天文） | JD↔儒略历/格里高利历；各朝代古历分数推步（平朔平气，仅作验证）；干支纯循环命名；历法换算选项 | `gregorian.rs`、`ancient_restore.rs`、`push_calendar.rs`、`daming_li.rs`、`gan_zhi.rs`、`options.rs` |
| 2 聚合/实算层（依赖天文的朔、气） | 某一部历某农历年的具体历日（朔+中气），按来源分实物历日、按术推步、今人历表、现算 | `artifacts.rs`（实物历日）、`push_calendar.rs`（按术推步，兼第 1 层历谱生成）、`historical.rs`（今人历表：张培瑜）、`chinese_lunar.rs`（现算真朔真气）、`dunhuang.rs`（敦煌出土历日有据之月）；干支历"节气派"（`gan_zhi_options.rs` 一支） |
| 3 派生层（依赖第 2 层产出） | 政权·帝王·年号纪年；干支历"农历派" | `era_names.rs`、`era_gaiyuan.rs`、`month_labels.rs`；干支历"农历派"（`gan_zhi_options.rs` 另一支） |
| 4 仲裁/路由层 | 某农历年行用哪部历、历日取哪一级来源；敦煌有据之月与中原历逐月取舍 | `lifa.rs`（`ZhongyuanCalendarPeriods`、`resolve_year`/`RiliSource`、`resolve_dunhuang_day`、`LifaRegistry`）；`convert.rs` 调用之 |
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

**概念**：依赖天文域产出的真朔、真气（见 [三域文档](../architecture/three-domains.md) 天文域第 3 层），或依赖史书还原好的静态朔日表，得到某历法某农历年的岁数据（14 朔 + 13 中气：冬至至下一冬至）。

**四种实现，角色相同、来源不同**（按资料可靠性由高到低；取哪一种由第 4 层 `lifa::resolve_year` 决定，见 [rili-sources.md](rili-sources.md)）：
- **实物历日**（`artifacts.rs`）：当时历书原件，目前只有《宝祐四年会天历》（1256）。
- **按术推步**（`push_calendar.rs` 平朔诸历、`dingshuo.rs` 唐至明定朔诸历、`daming_li.rs` 金重修大明历）：按该历术文推算，已实现并核过术文的历才用；与张表之间按本纪检验逐历取舍。
- **今人历表**（`historical.rs`，`ShuoriTable`）：解析 `vendor/calendar/chinese_calendar/实历朔日表.csv`，即张培瑜《三千五百年历日天象》经 ChineseCalendar 转录。是复原结果，书中不载推步过程与逐条依据。
- **现算**（`chinese_lunar.rs`）：直接调用天文域算真朔真气，覆盖任意年份；历史区间只在前三种都不覆盖时用。

四种共用 `historical::year_data_from_months` 由逐月朔日构造岁数据。

**干支历"节气派"**（`gan_zhi_options.rs` 一支）：年界=立春交节、月界=十二节交节，只依赖气，不依赖朔、不依赖农历，因此和 `chinese_lunar.rs` 同属这一层而非下一层。

**对应数据**：
- 朔日表：`vendor/calendar/chinese_calendar/实历朔日表.csv` — 张培瑜历表每月朔的民用历日期（1582-10-15 前儒略历）。
- 实物历日：`data/calendar/会天历_1256.csv`。
- 敦煌朔日表：`data/calendar/敦煌朔日表.csv` — 吐蕃、归义军时期敦煌出土具注历日有据之月（邓文宽 1996 附录一，第 1 级证据）；`dunhuang.rs` 只供有据之月，
  无据之月退回中原历还是不给由第 4 层 `lifa::DunhuangFallback` 选，见 [dunhuang-calendar.md](dunhuang-calendar.md)。

---

## 第 3 层：派生层

**概念**：依赖第 2 层的产出（农历年数、朔日、正月初一），不直接依赖天文。

**包含**：
- **历史月序改制**（`month_labels.rs`）：第 2 层中原一线历日一律按建寅编月；新莽、魏景初建丑，武周、唐肃宗末建子，当时月名不同（武周有「腊月」「一月」，肃宗以「建子月」等斗建为名）。本模块按政权把建寅月换成当时月名、或反向，只做命名，不改朔日。数据 `data/calendar/month_label_reforms.csv`，史料与规则见 [historical-calendar.md](./historical-calendar.md)「月序改制」。
- **政权·帝王·年号纪年**（`era_names.rs`+`era_gaiyuan.rs`）：依附在历法之上的纪年体系，按政权划分。同一时刻在不同政权下可有不同「朝代·帝王·年号·年数」，同一年可能有多个并存政权（如三国期间魏/蜀/吴），[`EraNames::get_all`] 返回全部，[`EraNames::get_primary`] 返回"正统主线"一条。
- **干支历"农历派"**（`gan_zhi_options.rs` 另一支）：年界=正月初一零点、月界=每月初一零点，依赖第 2 层算出的朔/正月边界。

**对应数据**：
- 年号表：`data/calendar/汇总/era_names_merged.csv` — 跨来源交叉校对出的汇总表（主线来自 `vendor/calendar/chinese_calendar/年号表.csv`，并存政权补充自 `data/calendar/fang_shiming/` 转录），字段：`lunar_year, regime, emperor, era_name, era_year, is_primary, has_gaiyuan, gaiyuan_prev_era_name, gaiyuan_prev_era_year, gaiyuan_month, source, note`。表结构与汇总方法见 `data/calendar/汇总/README.md`。

---

## 第 4 层：仲裁/路由层

**概念**：面对"同一件事有多种实现"时决定选哪个，职责是**选择**不是**计算**。

**包含**：
- **`lifa.rs`（`ZhongyuanCalendarPeriods`、`resolve_year`）**：中原一线逐年行用之历（`data/calendar/汇总/zhongyuan_calendar_periods.csv`），历日按实物历日 → 按术推步 → 今人历表 → 现算取，结果带 `RiliSource`；选项 `ZhongyuanPreference`（缺省逐历按本纪检验，另有一律推步、张表优先）。见 [rili-sources.md](rili-sources.md)。`LifaRegistry` 解析历法总表 `lifa_summary.csv`。
- **敦煌历**（`lifa::dunhuang_months`、`resolve_dunhuang_day`、`resolve_dunhuang_month`）：逐月选有据之月（第 2 层 `DunhuangCalendar`）或中原历之月
  （`lifa::zhongyuan_months_in_jdn_range`，同上来源次序），选项 `DunhuangFallback`；只挪动无据之月的月首使月长合 29、30 日，不推算朔日。纪年（吐蕃地支、归义军奉中原年号）
  在第 3 层 `era_names::dunhuang_year_name`。见 [dunhuang-calendar.md](dunhuang-calendar.md)。
- **`convert.rs`**：对外的"公历↔农历"换算入口，岁数据一律经 `lifa::resolve_year_data` 取，不自行判断来源。

### 已解决：仲裁与数据供给混在一处

原先 `historical.rs`（`HistoricalCalendar`）既供数据又隐含"这年在不在实历覆盖范围"的判断，`convert.rs` 把"实历优先、缺省退回定气定朔"写死在换算函数里。2026-11-10 起：`historical.rs` 只供张培瑜历表的数据（`ShuoriTable`）；"取哪一级来源"集中在 `lifa::resolve_year`，结果带来源标注；`convert.rs`、民俗铺注等只调用它。

---

## 关系小结

| 层次 | 概念 | 锚定 | 对应数据 |
|------|------|------|----------|
| 1 独立基础层 | 儒略历/格里高利历、古历推步、干支循环命名 | JD / 儒略历+格里历 | `data/calendar/汇总/lifa_summary.csv`、`vendor/calendar/lifa_tongzhi/lifa_tongzhi.csv` |
| 2 聚合/实算层 | 岁数据（实物、推步、今人历表、现算） | JD / 儒略历+格里历 | `data/calendar/会天历_1256.csv`、`vendor/calendar/chinese_calendar/实历朔日表.csv` |
| 3 派生层 | 政权·帝王·年号、历史月名、干支历农历派 | 农历年 / JD / 儒略历+格里历 | `data/calendar/汇总/era_names_merged.csv`、`data/calendar/month_label_reforms.csv` |
| 4 仲裁/路由层 | 中原一线行用之历、历日来源次序 | — | `data/calendar/汇总/zhongyuan_calendar_periods.csv`、`data/calendar/汇总/lifa_summary.csv` |

第 3 层依赖第 2 层：某政权的「某年某月某日」由该政权所采用的历法（第 1/2 层）与朔日/推步决定，再落到儒略日或公历上。第 4 层横切前三层，不是被依赖，而是决定"该走第 2 层的哪种实现"。

**数据字段/来源细节**（各 CSV 的具体列、生成脚本、上游出处）见 [../../data/calendar/README_shuori.md](https://github.com/lhttjdr/calendar/blob/master/data/calendar/README_shuori.md) 与 [../../data/calendar/SOURCES_era_regime.md](https://github.com/lhttjdr/calendar/blob/master/data/calendar/SOURCES_era_regime.md)；[../../data/calendar/汇总/README.md](https://github.com/lhttjdr/calendar/blob/master/data/calendar/汇总/README.md) 另有 `era_names_merged.csv` 的汇总方法说明。
