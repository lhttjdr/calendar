# 三大域与域内四段结构

本仓库的计算逻辑分三个大域，依赖方向单向：

```
天文 (astronomy) → 历法 (calendar) → 民俗 (folk)
```

- **天文**：朔望、节气、月相、日月食等——由历表（VSOP87/ELPMPP02）、岁差、章动推出的日月视位置及其导出量。不知道、也不需要知道"农历""年号"这些概念。
- **历法**：干支历、农历/夏历（含历代各版本）、政权年号纪年等——把天文量组织成"年月日"这种人可读的历法结构。会用到天文的输出（真朔、真气），但天文不反向依赖历法。
- **民俗**：择日（建除、神煞、宜忌、婚嫁）、式占（奇门、六壬、太乙）、筮法（周易、六爻）、命理（七政四余、紫微斗数）、风水（玄空）等——依附在历法产出的干支/日期上，按原典规则给出结果，不做吉凶解读。多数规则只用历法的输出；需要「交节/中气时刻」「朔弦望」「太阳高度」「七曜宫度」的模块直接取天文域的产出（见下「民俗域」）。

这个方向性很重要：**下游可以依赖上游的产出，上游永远不知道下游的存在**。天文模块不应该出现"农历""年号"字样，历法模块不应该出现"八字""黄道"字样。

命名与归属也算在内。例：干支历的年界、月界、闰月、日界选项是历法本身的规则，放历法域（`calendar::gan_zhi_options`，中性预设为「节气干支历」「农历月干支」）；「子平取立春、节令、子正」「紫微取农历年月、闰月作下月」是民俗各派的取法，放民俗域（`folk::ganzhi_presets`），由民俗模块选用历法选项。注释中援引《命理探源》《紫微斗數全書》等书说明某分界法的出处，不算依赖。

## 域内的共同形状：四段结构

三个域各自内部都不是平的、并列的模块列表，而是同一种四段依赖形状：

1. **独立基础层**：互不依赖，只依赖更底层的通用设施（时间、数学）或纯配置/常量，谁也不用等谁。
2. **聚合/实算层**：依赖第 1 层，把多个基础量组合、现算出该域的"核心产出物"。
3. **派生层**：依赖第 2 层（或第 2 层的产出物），在其之上再加工出更具体的东西。
4. **仲裁/路由层**（横切，不是计算依赖）：面对"同一件事有多种实现/多种数据来源"时，决定选哪一个。这层的职责是**选择**，不是**计算**——一旦不小心把选择逻辑和计算逻辑混在同一个函数/文件里，选择逻辑就会跟着计算逻辑到处复制，后续想改选择策略（比如新增一种数据源、调整优先级）就要满仓库找。本仓库已经踩过这个坑两次（见下面两条案例），值得当成一条通用设计禁忌记下来。

四段里最容易被漏掉、也最容易和第 2/3 层混在一起写的是第 4 层。下面按域给出具体映射。

---

## 天文域

| 层 | 内容 | 对应代码 |
|----|------|----------|
| 1 独立基础层 | 时间尺度/儒略日、VSOP87/ELPMPP02 历表级数、岁差、章动——彼此独立，只依赖时间 T | `astronomy::time`、`astronomy::ephemeris`、`astronomy::frame::{precession,nutation}` |
| 2 聚合/实算层 | 岁差+章动组成参考架变换，配合历表+光行时/光行差改正，算出某天体在指定架下的视位置 | `astronomy::correction`（光行时/光行差/大气折射/BCRS）、`astronomy::pipeline`（State6/TransformGraph 内部实现）、对外 API：`astronomy::apparent` |
| 3 派生层 | 依赖第 2 层产出的视位置，做数值求根/极值搜索，得到具体天象时刻 | `astronomy::aspects`：`synodic`（朔望）、`solar_term`（节气）、`longitude.rs`（两者共用的视黄经工具）；月相是朔望/黄经差的直接推论。日月食、行星会合、伏见、月掩星等天象在 `astronomy::phenomena`（同属第 3 层，日月食以朔望时刻粗筛，对第 3 层自身再依赖；见 doc/astronomy/celestial-phenomena.md） |
| 4 仲裁/路由层 | 选历表数据格式（全二进制/二进制+文本混合/全文本）、选章动表来源（完整 IAU2000A 二进制/回落 77 项） | Web 侧集中在 [`web/src/ephemerisSourceRegistry.ts`](https://github.com/lhttjdr/calendar/blob/master/web/src/ephemerisSourceRegistry.ts)；Rust 侧章动的"取不到完整表就回落 77 项"逻辑在 `astronomy::frame::nutation::iau2000b::try_init_full_nutation_from_repo`/`_from_binary` |

**案例（本仓库真实踩过的坑）**：早期这层的决策逻辑分散在两处——章动表的回落判断写在 Rust 里，历表格式的三级降级判断写在 TypeScript 的 `useLunarWasm.ts` 一个大函数内部，且同一段 fetch+校验逻辑在主线程和 Worker（`yearDataWorker.ts`）各写了一份，两处会逐渐漂移。2026-09-23 提炼出 `ephemerisSourceRegistry.ts` 作为这层的唯一归属地后，两处调用方改为共享同一份决策函数。

## 历法域

| 层 | 内容 | 对应代码 |
|----|------|----------|
| 1 独立基础层 | 不依赖天文，纯算术/纯配置：JD↔儒略历/格里高利历；各朝代古历的分数推步（平朔平气，常数各异，仅作验证用）；干支纯循环命名（甲子…癸亥，只依赖日期整数）；历法换算选项配置 | `gregorian.rs`、`ancient_restore.rs`、`push_calendar.rs`、`gan_zhi.rs`、`options.rs` |
| 2 聚合/实算层 | 依赖天文域第 3 层（朔、气）算出该历法的核心产出——某农历年的岁数据（朔+中气）；或者不现算、改用史书还原好的静态表 | `chinese_lunar.rs`（现算真朔真气）、`historical.rs`（查表，见下方「发现的问题」）；干支历的"节气派"（年界立春、月界节气）只依赖气，也属于这层，混在 `gan_zhi_options.rs` 里 |
| 3 派生层 | 依赖第 2 层的产出（农历年数、朔日、正月初一） | 政权·帝王·年号纪年 `era_names.rs`+`era_gaiyuan.rs`（依赖农历年数）；干支历的"农历派"（年界正月初一、月界朔日，混在 `gan_zhi_options.rs` 里，依赖 `chinese_lunar.rs` 的朔/正月输出） |
| 4 仲裁/路由层 | 某农历年该用哪种历法实现（现算实历 / 查历史表 / 某朝代古历）；实历覆盖范围内优先用实历、缺省退回定气定朔现算 | `lifa.rs`（`LifaRegistry`，历法总表路由）；`convert.rs`（"优先实历、缺省退回定气定朔"的具体判断，见下方「发现的问题」） |

**契约层，不算在四段里**：`system.rs`（`CalendarSystem` trait）是 JD↔各历法日期互转的通用接口定义，被第 1/2 层的各历法系统共同实现，是横切的接口契约，不属于任一计算依赖层。

**发现的问题（2026-11-10 已改，见 [calendar-layers.md](../calendar/calendar-layers.md)「已解决」与 [rili-sources.md](../calendar/rili-sources.md)；以下为原记录）**：`historical.rs` 和 `convert.rs` 里，"仲裁"和"计算/数据供给"两个职责混在同一个文件甚至同一个函数里——

- `historical.rs` 的 `HistoricalCalendar` 既负责"给定农历年，返回该年朔日岁数据"（这是第 2 层的数据供给角色），也隐含着"这个农历年在不在实历覆盖范围内"的判断（这其实是第 4 层的路由职责），两者没有分开命名。
- `convert.rs` 里"实历覆盖范围内优先用实历，缺省退回定气定朔现算"这条规则，是不折不扣的第 4 层仲裁逻辑，但直接写死在换算函数内部，不是一个可以单独查阅、复用、修改优先级的具名决策点。

这和天文域早前发现的问题是同一种代码异味：**仲裁逻辑没有独立成第一等公民，就会散落进数据供给代码里**。是否要像天文域那样把它拆成一个显式的 `calendar::registry`（或扩展 `lifa::LifaRegistry` 承担这个职责），见本次"改实现"的落地记录。

## 民俗域

代码在 `rust/core/src/folk/`，各规则的原典出处与逐条核对见 `doc/folk/*.md`，网页接入见 [doc/engineering/web-folk-pages.md](../engineering/web-folk-pages.md)。只算到规则给出的结果（某日值何神、宜忌几项、盘上各宫何星），不做吉凶解读。

| 层 | 内容 | 对应代码 |
|----|------|----------|
| 1 独立基础层 | 只依赖日期整数、干支序数或调用方给的数（节气所在日、纬度与赤纬、生年月日时），互不依赖：纳音；二十八宿值日与七曜；年月紫白九星；建除十二值、黄道黑道、百忌日；月家日神煞（`data/folk/shensha_month.csv`）；协纪月表宜忌（`yiji`）与《董公選擇日要覽》宜忌（`dong_gong`，并列之另一说）；年神方位、日游神、鹤神；杂节气（三伏、社日、寒食、腊日等，输入为节气所在民用日）；更点（输入纬度与太阳赤纬）；周易大衍筮法与经文；六爻纳甲装卦、京房飞伏；紫微斗数两派（输入农历生年月日时，调用方由历法域取得） | `nayin`、`zhiri`、`jiuxing`、`jianchu`、`shensha`、`yiji`、`dong_gong`、`nianshen`、`zajie`、`gengdian`、`zhouyi`/`zhouyi_text`、`liuyao`、`ziwei`、`ziwei18` |
| 2 聚合/实算层 | 把第 1 层与历法、天文产出组合成该域的核心产出：**逐日铺注**（月表之上按当年节气、朔弦望、农历日加反支、上朔、四離四絕、土王用事、伏社、月忌、嫁娶周堂、交节等，得一日宜忌）；**排盘**（某时刻的奇门局与盘、六壬天地盘与四课三传、太乙四计与局、七政四余十一曜宫度宿度与命宫、玄空宅盘与流年流月日时紫白） | `puzhu`（农历日经 `calendar::lifa::resolve_year_data` 实历优先；节气、朔弦望取 `astronomy::aspects`）；`qimen::{pan,nianjia}`（定局用节气时刻：定气取 `astronomy::aspects::solar_term_jds_for_year_cached`，行用历平气取 `calendar::pingqi`）；`liuren`（月将按中气时刻换，取 `astronomy::aspects`；以日出入定昼夜取 `astronomy::apparent` 太阳视赤经赤纬与 GAST 求太阳高度）；`taiyi`（积年、章岁法纯算术）；`qizheng`（七政直接取 `astronomy::apparent::body_apparent_ecliptic` 视黄经黄纬，四余用 `astronomy::frame::{nutation,precession}`，宿度用 `astronomy::sky_culture::chinese` 的二十八宿；网页经星图 Worker 的 `SkyEngine::qizheng`）；`xuankong`（三元九运借 `taiyi`，日时紫白之二至取 `astronomy::aspects`） |
| 3 派生层 | 依赖第 2 层的产出再加工：婚嫁择吉（在铺注逐日宜忌、嫁娶周堂、值宿之上按新人之命定年月日与否决缘由）；奇门格局、逐宫标记；六壬课体课格、遁干与诸神煞；太乙诸神、定计、格局、八门、算和、十精、阳九百六；七政四余星命（量天尺命度、限法、宫位神煞）；六爻《黄金策》诸象（在装好的卦上）；玄空格局与逐宫标记 | `jiaqu`（用 `puzhu::{yiji_for_day,zhou_tang}`、`zhiri`）；`qimen::{geju,biaoji}`（月将贵人诸格借 `liuren`）；`liuren::{ke_ti,shen_sha}`；`taiyi::{zhushen,geju,bamen,suanshu,shijing}`；`qizheng::{xian_fa,shen_sha,guo_gong}`；`liuyao::huang_jin_marks`；`xuankong` 内格局诸函数 |
| 4 仲裁/流派层 | 同一件事诸家判法不同：各说照录并列，协纪（或各术主书）为缺省，其余作选项，不作取舍 | 无集中注册表：每模块以具名枚举/选项结构给出流派，并带缺省常量——神煞起例 `ShenshaOptions`（协纪/类编，`QiLiYi`）；宜忌之家（`yiji` 协纪月表 / `dong_gong`）；`JiaquOptions`（周堂、白虎、赤口、吉日表诸说）；铺注 `BuYiOptions`；奇门 `DingJu`/`RunRule`/`JieQiSource`/`PanOptions`；六壬 `JiangRule`/`GuiRule`/`ChuYu`/`YaoTie`/`SheHaiRule`；太乙 `JiNian` 与布门、门具、算和诸法；紫微 `ZiweiOptions`、`Ziwei18Options`；玄空 `XkOptions`、`RiShiOptions`；六爻 `LiuyaoOptions`；周易 `ZhanOptions`；七政 `QizhengOptions`（罗计、交点、命宫法等）。wasm 层（`rust/wasm-lib/src/lib_impl.rs` 各 `*_json` 的 `opts` 位域）把网页所选译成这些枚举。历法层面的选择复用历法域第 4 层：铺注的农历日走 `lifa::resolve_year_data`，奇门「行用历平气」走 `calendar::pingqi` |

**依赖方向的实情**：民俗域**直接依赖天文域**，不全经历法域转手——节气、中气的精确时刻（奇门定局、六壬换将、杂节气、铺注交节）与朔弦望（铺注）取天文域第 3 层 `astronomy::aspects`；六壬日出入昼夜、七政四余的七曜位置取天文域第 2 层视位置管线 `astronomy::apparent`，七政还用章动、岁差与 `sky_culture` 的二十八宿。日期、农历月日、干支、平气取历法域。方向仍是单向的：天文、历法模块不引用 `folk`。

**干支历四柱**（八字的年月日时柱，月柱以节为界、立春或冬至换年、夜子时诸说）属历法域：代码在 `calendar::gan_zhi_options`，原典与选项见 [doc/folk/ganzhi-calendar.md](../folk/ganzhi-calendar.md)。子平命理（十神、大运、流年）未做。

---

## 相关文档

- 历法域完整拆解（数据字段、CSV schema）：[doc/calendar/calendar-layers.md](../calendar/calendar-layers.md)
- 天文域各推导章节：[doc/index.md](../index.md) 「一、天文基础」
