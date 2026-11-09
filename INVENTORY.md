# 项目大清单

本仓库「历法 / 纪年 / 天文 / 数据 / 文档 / 脚本」总览，便于检索与补全。更新时请同步本表。

> 本文件供 AI 辅助开发时快速定位使用，不是面向读者的文档，也不发布（读者文档站在 `site/`）。

---

## 一、概念与架构

| 项 | 说明 | 文档 |
|----|------|------|
| 三大域 | 天文（astronomy）→ 历法（calendar）→ 民俗（folk），依赖单向（民俗直接取天文的节气朔望时刻、视位置，也取历法的日期干支农历） | [doc/architecture/three-domains.md](https://github.com/lhttjdr/calendar/blob/master/doc/architecture/three-domains.md) |
| 历法域四层 | 1 独立基础层（儒略历/格里历/古历推步）→ 2 聚合实算层（现算或查表得岁数据）→ 3 派生层（政权年号）→ 4 仲裁路由层（该用哪种历法实现） | [doc/calendar/calendar-layers.md](https://github.com/lhttjdr/calendar/blob/master/doc/calendar/calendar-layers.md) |
| 民俗域四层 | 1 独立基础层（纳音、值宿、九星、建除、神煞、月表宜忌、年神、杂节气、更点、周易、六爻、紫微）→ 2 聚合实算层（逐日铺注 `puzhu`；奇门、六壬、太乙、七政四余、玄空排盘）→ 3 派生层（婚嫁择吉、各盘格局标记、星命、黄金策诸象）→ 4 流派层（各模块具名选项，协纪或主书为缺省） | [doc/architecture/three-domains.md](doc/architecture/three-domains.md)「民俗域」、[doc/folk/yuanliu.md](doc/folk/yuanliu.md) |
| 主表 | 《三千五百年历日天象》按各朝颁历**据历还原**推步得到的连续朔闰表；**非**书中某一历法名，属历法域第 2 层 | [data/calendar/README_shuori.md](https://github.com/lhttjdr/calendar/blob/master/data/calendar/README_shuori.md)、[historical-calendar.md](doc/calendar/historical-calendar.md) |
| 历法总表 id | `zhongyuan_shili` → 主表（据历还原）；`lifa_summary.csv` 另有格里高利、儒略、现代农历、历法通志各历 | [data/calendar/汇总/lifa_summary.csv](https://github.com/lhttjdr/calendar/blob/master/data/calendar/汇总/lifa_summary.csv) |

---

## 二、历术与算法（ytliu0/ChineseCalendar 对应）

| 历术 li | 中文名 | 朔策 | 岁实 | 备注 |
|---------|--------|------|------|------|
| Sifen | 东汉四分 | 29+499/940 | 365.25 | 蜀、吴前期等 |
| Qianxiang | 乾象 | 29+773/1457 | 365+145/589 | 吴 |
| Jingchu | 景初 | 29+2419/4559 | 365+455/1843 | 北魏等 |
| Sanji | 三纪 | 29+3217/6063 | 365+605/2451 | 后秦 |
| Xuanshi | 玄始 | 29+47251/89052 | 365+1759/7200 | 北凉、北魏 |
| Zhengguang | 正光 | 29+39769/74952 | 365+1477/6060 | 北魏/东魏 |
| fakeMingKeRang | 明克让 | 同正光 | 节气 JDw+0.4 | 北周 |
| Xinghe | 兴和 | 29+110647/208530 | 365+4117/16860 | 东魏 |
| Tianbao | 天保 | 29+155272/292635 | 365+5787/23660 | 北齐 |
| Tianhe | 天和 | 29+153991/290160 | 365+5731/23460 | 北周 |
| Daxiang | 大象 | 29+28422/53563 | 365+3167/12992 | 北周 |
| Kaihuang | 开皇 | 29+96529/181920 | 365+25063/102960 | 隋 |
| Xuanming | 宣明 | — | 365+137/560 | 仅平气 |
| RevisedDaming | 重修大明 | — | 365+637/2615 | 仅平气 |
| Datong | 大统 | — | 365.2425 | 仅平气 |

**古六历**：周、黄帝、殷、鲁、颛顼、夏(春秋)/夏(战国) — 朔策岁实同上四分，差在历元与 ziOffset。  
**春秋历**：朔策 30328/1027，闰年表。  
**秦与汉初**：HanZhuanXu，三段历元 + 19 年章。  

详表与政权分段见 [push-algorithms.md](doc/calendar/push-algorithms.md)。

---

## 三、Rust `calendar` 模块

| 模块 | 作用 |
|------|------|
| `ancient_restore` | 古历复原：分数推步，平朔平气+历谱；古六历预设；三千五百年仅作验证 |
| `chinese_lunar` | 农历岁数据、定气定朔换算 |
| `convert` | 公历↔农历（优先实历） |
| `era_names` | 第 3 层（派生层）年号表解析 |
| `gan_zhi` / `gan_zhi_options` | 干支 |
| `gregorian` | 公历与 JD |
| `historical` | 主表朔日 CSV → 岁数据 |
| `lifa` | 第 4 层（仲裁/路由层）历法表解析，`LIFA_ID_ZHONGYUAN_SHILI` |
| `month_labels` | 第 3 层：历史月序改制（新莽/景初建丑、武周/肃宗末建子）当时月名 ↔ 主表建寅月，`data/calendar/month_label_reforms.csv` |
| `push_calendar` | 平朔+无中气置闰（首个无中气月为闰），各历术 `LiParams` 预设；`regime_year_months` 导出 `data/calendar/推步政权朔日表.csv` |
| `options` / `system` | 选项与历法系统 |

---

## 四、历法通志与历法名单数据

| 文件 | 内容 |
|------|------|
| [vendor/calendar/lifa_tongzhi/lifa_tongzhi.csv](https://github.com/lhttjdr/calendar/blob/master/vendor/calendar/lifa_tongzhi/lifa_tongzhi.csv) | 历名·治历者·岁实·朔策·成历之年·公元·备注（两页合一整理） |
| [vendor/calendar/lifa_tongzhi/历法通志_README.md](https://github.com/lhttjdr/calendar/blob/master/vendor/calendar/lifa_tongzhi/历法通志_README.md) | 来源朱文鑫《历法通志》、列说明 |
| [data/calendar/汇总/lifa_summary.csv](https://github.com/lhttjdr/calendar/blob/master/data/calendar/汇总/lifa_summary.csv) | 历法 id、历名、起止 CE、制作者、锚定、来源（本项目整理的精选清单，非来源原样转录） |
| [data/calendar/汇总/regime_era_calendar_matrix.csv](https://github.com/lhttjdr/calendar/blob/master/data/calendar/汇总/regime_era_calendar_matrix.csv) | 年号表.csv 的宽表重排（一政权一列），供人工浏览/校验；`build_regime_era_calendar_matrix.mjs` 从 年号表.csv 生成，`.xlsx` 为同内容的 Excel 版本 |

---

## 五、纪年与朔日（主表）

| 文件 | 内容 |
|------|------|
| [vendor/calendar/chinese_calendar/年号表.csv](https://github.com/lhttjdr/calendar/blob/master/vendor/calendar/chinese_calendar/年号表.csv) | lunar_year, has_gaiyuan, regime, emperor, era_name, era_year, gaiyuan_prev_* |
| [vendor/calendar/chinese_calendar/年号表.json](https://github.com/lhttjdr/calendar/blob/master/vendor/calendar/chinese_calendar/年号表.json) | 同上 JSON |
| [vendor/calendar/chinese_calendar/改元候选表.csv](https://github.com/lhttjdr/calendar/blob/master/vendor/calendar/chinese_calendar/改元候选表.csv) | 改元年子集 |
| [vendor/calendar/chinese_calendar/实历朔日表.csv](https://github.com/lhttjdr/calendar/blob/master/vendor/calendar/chinese_calendar/实历朔日表.csv) | 朔日：公历↔农历月 |
| [vendor/calendar/chinese_calendar/实历朔日表.json](https://github.com/lhttjdr/calendar/blob/master/vendor/calendar/chinese_calendar/实历朔日表.json) | 同上 JSON |

说明类：[README_shuori.md](https://github.com/lhttjdr/calendar/blob/master/data/calendar/README_shuori.md)（含朔日表、年号表、改元候选表三份数据说明，以及各朝代历法差异、三国魏蜀吴专节）、[SOURCES_era_regime.md](https://github.com/lhttjdr/calendar/blob/master/data/calendar/SOURCES_era_regime.md)；历法域四层结构说明见 [doc/calendar/calendar-layers.md](https://github.com/lhttjdr/calendar/blob/master/doc/calendar/calendar-layers.md)。

---

## 六、天文与历表数据（节选）

| 路径 | 用途 |
|------|------|
| `vendor/vsop87/VSOP87B.ear` 等 | VSOP87 地球轨道（定气等），原样保留的上游数据 |
| `vendor/elpmpp02/` | ELPMPP02 月球（定朔等），原样保留的上游数据 |
| `vendor/IAU2000/` | 章动表，原样保留的上游数据 |
| `vendor/jpl/de406/de406.bsp` | JPL DE406 星历（测试/比对用），原样保留的上游数据 |
| `data/fit/` | 本项目自己生成的历表拟合：VSOP87 日心改正（DE431 缺省、DE406、DE441，`vsop87-*`）；ELPMPP02 拟合 DE431（缺省）、DE441 的常数集（`elpmpp02-de4*.txt`，`examples/elpmpp02_de_fit.rs`）与三套目标的残差改正（`elpmpp02-de4*-patch.txt`，`examples/elpmpp02_residual_fit.rs`），连同逐世纪残差 CSV；常数集与改正都编译时嵌入。配套规则见 doc/astronomy/model-pairing.md |
| `vendor/月相和二十四节气的计算/` | TDB 时刻参考，原样保留的上游数据 |

详见 [data-paths-summary.md](doc/engineering/data-paths-summary.md)。

---

## 六之二、史书天象检验（`data/phenomena/`）

| 项 | 内容 |
|----|------|
| `<书>_star_records.csv` | 正史天文志月五星凌犯、合聚解析结果（汉书至清史稿 16 部均由 `scripts/calendar_data/qinhan_eval/zhi_star_records.py <书>` 生成）；列见 `data/phenomena/README.md` |
| `<书>_{solar,lunar}_eclipses.csv` | 同一解析器提取的日月食，逐行带都城经纬度；`《清史稿》` 日食由 `qingshigao_eclipses.py` 生成 |
| `capitals.csv` | 各政权都城（观测地），Rust 凌犯检验（`tests_records.rs`）与 Python 日月食表共用 |
| `checks/` | 逐条检验结果：`<书>_star_records`（Rust `<书>_star_records_check`，`PHENOMENA_CHECKS=1` 时写出）、`<书>_date_shift`、`recorded_distances`（尺寸与角距）、`lodge_degrees`（入宿度）、`qingshigao_eclipses`、`qingshigao_moon_fullday` |
| `<书>_{solar,lunar}_eclipses_check.csv` | `rust/core/examples/historical_eclipses.rs` 都城所见食况（f64） |
| 总报告 | `doc/astronomy/phenomena-checks.md`（`scripts/phenomena/checks_to_md.py` 生成，勿手改），按朝代汇总见 `doc/astronomy/celestial-phenomena.md` §十 |
| 一键重算 | `scripts/dev/phenomena_checks.sh`（缺省 f64；`compare` 两种构建对照） |

---

## 七、文档索引（doc）

| 章 | 文件 | 主题 |
|----|------|------|
| 天文基础 | [index.md](doc/index.md) 所列 `astronomy/` | 时间尺度、标架、岁差章动、历表、光行差、视位置、月相节气 |
| 农历编算 | [index.md](doc/index.md) 所列 `calendar/` | 农历编算、实历模块、古历复原、各朝推步与历术表 |
| 民俗 | [index.md](doc/index.md) 所列 `folk/`（总论 [yuanliu.md](doc/folk/yuanliu.md)） | 干支历四柱、纳音值宿、建除神煞宜忌、通书对照、婚嫁、奇门、六壬、太乙、七政四余、紫微、六爻、周易、玄空 |
| 工程实现 | [index.md](doc/index.md) 所列 `engineering/` | 项目实现、WASM、二进制、数据路径、管线图 |
| 案例分析 | [de406-solar-term-failure-analysis.md](doc/case-studies/de406-solar-term-failure-analysis.md) | DE406 定气节气分析 |

---

## 八、脚本（scripts）

按用途分子目录：`dev/`（构建/测试/覆盖率）、`ephemeris/`（历表拟合、章动表抓取、二进制生成与压缩）、`calendar_data/`（纪年/朔日表处理）、`manim/`（文档配图/动画）。

| 脚本 | 作用 |
|------|------|
| `calendar_data/extract_shuori_from_chinese_calendar.js` | 从 ChineseCalendar 提取朔日 |
| `calendar_data/refactor_era_names_csv.mjs` | 年号 CSV 重构（含改元列） |
| `calendar_data/era_names_to_base_csv.mjs` | 裁成 6/8 列基础表 |
| `ephemeris/compress_ephemeris_brotli.mjs` | 历表压缩 |
| `calendar_data/qinhan_eval/zhi_star_records.py` | 正史天文志通用解析（汉书至清史稿 16 部；`BOOKS` 按书配置节、年号政权、帝号纪年、历表（含太初前复原历）、默认主语、日月食节）；`houhanshu_star_records.py` 为其共用工具库；年号繁体用 OpenCC（scripts 工程依赖 `opencc-python-reimplemented`） |
| `calendar_data/qinhan_eval/qingshigao_eclipses.py` | 《清史稿》日食（时辰、食分、所次宿） |
| `calendar_data/build_era_names_merged.py` | 合并年号表 `data/calendar/汇总/era_names_merged.csv`（主线 + 方诗铭并存政权） |
| `calendar_data/check_dated_inscriptions.py` | 出土纪日对各历表核对 |
| `calendar_data/build_dunhuang_shuori.py`、`calendar_data/dunhuang_checks_to_md.py`、`phenomena/dunhuang_fallback_check.py` | 敦煌朔日表生成（自邓文宽附录一比较表）、换算逐条检验写入 `doc/calendar/dunhuang-calendar.md`、原件之月对主表与实际合朔（定无据之月退回何表）；Rust 侧 `examples/dunhuang_check.rs` |
| `calendar_data/check_dunhuang_calendar.py` | 敦煌北魏太平真君十一、十二年历日（`data/calendar/敦煌历日.csv`）对各历表与景初、玄始历重推（朔闰、节气） |
| `calendar_data/check_lvli_epochs.py` | 《隋书·律历志》天和、大象历常数复核与天和年间闰位诸算法 |
| `calendar_data/discriminating_records.py`、`phenomena/discriminating_records_sky.py` | 主表与同系推步表判断不一的史书天象记录与本纪纪日（`data/calendar/本纪纪日/`，`zhi_star_records.py` 的 `date_sections` 纪日模式），及以天象判定哪一历为真 |
| `phenomena/record_date_shift.py` | 行星记录日期偏移、干支可落之月（skyfield + DE441/DE406） |
| `phenomena/recorded_distances.py`、`lodge_degrees.py` | 所记尺寸、入宿度与推算对照 |
| `phenomena/qingshigao_eclipse_compare.py`、`qingshigao_moon_fullday.py` | 清代日食时辰食分、相距书全天地心角距 |
| `phenomena/checks_to_md.py` | 生成 `doc/astronomy/phenomena-checks.md` |

---

## 九、前端 / WASM（要点）

| 项 | 说明 |
|----|------|
| `rust/wasm-lib` | 年号、实历、天文接口等 |
| `web/` | React 应用；年号展示含改元 `gaiyuan_prev_*` |
| 民俗页 | 择日页（宜忌、神煞起例、董公、婚嫁择吉、年神罗盘）、排盘页（奇门、六壬、太乙、七政四余、紫微、六爻·周易、玄空）；wasm `*_json`，七政经星图 Worker `SkyEngine::qizheng`；见 [doc/engineering/web-folk-pages.md](doc/engineering/web-folk-pages.md) |

---

## 十、待办（按优先级，2026-11-09 整理）

**2026-10-29 以后已完成（据 git log 归纳）**
- [x] 天文：以 DE431 为日、月历表与 ΔT 配套枢纽，ΔT 改 MS2021；ELPMPP02 常数集照 Fortran 导出、按论文两步法拟合 DE441 并加 Chapront 式残差改正；章动基本角两处符号订正；库缺省改 f64；管线补站心到视地平、折射缺省光线追迹；变换图改网络；视位置加太阳引力光线偏折（对 SOFA 18 例差 0）
- [x] 天象与星图：日月食年表对 NASA 目录批量检验、日食图与影子轮廓、月食图与月面实况、计算提速 3—4 倍；3D 星空（地形、4K 银河、日月光晕）；石氏星表与三家星官检验
- [x] 史书天象：先秦日躔中星按岁差检验（月令、尧典）、《左传》卜偃与《国语》伐纣天象；汉代日食宿度溯源（太初前三统、光武后四分）及诸假设排除；春秋日食刘歆分野
- [x] 历法：唐宋金元明诸历定朔复原与《历代历法总录》（含表逐格对点校本）；平气按当时行用历推算（太初至两宋、五代钦天、元初重修大明历，`pingqi_check`）；古六历书朔逐历分析；清代本纪核验
- [x] 干支历四柱：授时大统平气与当地太阳时时柱，冬至换年、夜子时、立春换年据原典
- [x] 日历显示：民用历（1582-10-15 前儒略历）、罗马实历九种、按时代显示行用之历、年号附君主本名、时区与古代日期
- [x] 民俗择日：纳音、值宿七曜、紫白九星、杂节气、更点、建除黄道百忌；月家日神煞；协纪月表宜忌 720 日逐页核毕（存疑十九日以殿本校）；铺注层与乾隆三十年时宪书 384 日核对、他年复核；年神方位
- [x] 民俗通书对照：类编、董公（144 条复核、逐日判定表）、鳌头（十二星断、婚姻四表）、宗镜通读、类编卷十一 720 日逐日表；神煞起例可选协纪、类编或逐神
- [x] 民俗婚嫁：协纪为纲、类编并列，制化三态、周堂白虎、赤口、类编逐月吉日表、横天朱雀；神煞清点待实现清单 23 项（納采問名重推、铺注逐项不看、协纪改删旧说与类编鳌头董公诸神逐神取舍、董公与鳌头婚姻表、各书之说目录）
- [x] 民俗式占命理筮法风水：奇门（定局、飞盘、格局、方位格、年家、逐宫标记）、六壬（天将、课体课格、日墓、出狱、昼夜）、太乙（诸神、定计、格局、八门、算和、十精、阳九百六、章岁法）、紫微两派、七政四余（宫度宿度、星命、限法、神煞、时宪书比对）、六爻（纳甲、京房飞伏、黄金策诸象、增删卜易 298 例）、周易大衍考变占、玄空（飞星、零正、兼向、大卦、三针、日时紫白）
- [x] 网页：择日页、排盘页（多层罗盘、随时刻转盘、GPS 与 WMM2025 磁偏角罗盘）；首屏按需加载；部署拉取 LFS 数据表并查指针
- [x] 读者站：按 数学→天文→历法→民俗 分篇，各章重写并核原文书目；民俗篇总论与九章

**民俗待做（摘自各 doc/folk/*.md「待做」，按优先级）**
- [ ] P1 择日通书：鳌头卷二至卷五各门吉日表与凶神起例录出、读卷六补复日等起例（[tongshu_yiji.md](doc/folk/tongshu_yiji.md) 第十一节）；类编天贼作选项、鳌头中查天愿地囊四击枯焦天狱（[tongshu.md](doc/folk/tongshu.md) 第九节）；董公录文以书影校
- [ ] P1 时宪书：「不宜」取舍规则；交节界限按《历象考成后编》推当年交节时刻；协纪四库录文以影印本校（[shensha.md](doc/folk/shensha.md) 第三节）
- [ ] P2 式占：奇门飞盘阴遁与九神飞布原典、年家年界、日家月家顺飞、白奸（[qimen.md](doc/folk/qimen.md) 第七节）；六壬未做课格十二种、平气换将与恒星过宫、土旺四季（[liuren.md](doc/folk/liuren.md) 第七节）；太乙交至加时起门、十精立成图、飞符、章岁法闰月（[taiyi.md](doc/folk/taiyi.md) 第十六节）
- [ ] P2 命理：七政授时过宫度表余八宫 240 格、鄭氏星案命例、紫炁历元、罗计改年（[qizheng.md](doc/folk/qizheng.md) 第五节）；紫微明刻全书书影核庙旺与 27 处异、整盘命例（[ziwei.md](doc/folk/ziwei.md)）；十八飞星庙乐旺表（[ziwei18.md](doc/folk/ziwei18.md)）
- [ ] P2 风水筮法：沈氏玄空原刊、章仲山原书、无常派并列（[xuankong.md](doc/folk/xuankong.md) 第八节）；《黄金策》原书与 298 例断语结构化比对（[liuyao.md](doc/folk/liuyao.md)）；《啟蒙》三十二图影印本（[zhouyi.md](doc/folk/zhouyi.md)）
- [ ] P3 婚嫁：人隔、天狗無嗣、董公五鬼黑煞到州星起例待查（[jiaqu_shensha_inventory.md](doc/folk/jiaqu_shensha_inventory.md) 第十一节）、男女禁婚年、鳌头卷六制煞法（[jiaqu.md](doc/folk/jiaqu.md) 第七节）
- [ ] 待用户定：子平命理是否立章；读者站通书异同是否单列一章；合婚是否作通书说选项（[yuanliu.md](doc/folk/yuanliu.md)「民俗篇待做」）

**P0 合并前**
- [x] 全量重跑 `scripts/dev/phenomena_checks.sh`（含 `compare`，两构建逐格一致）与 `cargo test -p lunar-core`（270 过、0 败），2026-10-29

**P1 价值最高**
- [x] 以本纪核验主表：《三国志》至两《唐书》十二种本纪，明书朔日、实际合朔、閏月位置（`doc/calendar/main-table-checks.md`，
  结论见 historical-calendar.md「以本纪核验主表」）；顺带订正北齐天保历实行年（push-algorithms.md §3.6）
- [x] 五代以后本纪（两《五代史》、《宋史》《辽史》《金史》《元史》《明史》）同法核验主表：合 96%—99%；授时历废进朔得证；辽、金进朔与宋不同
- [x] 敦煌唐五代历日：邓文宽 1996 附录一比较表 35 年转录（`data/calendar/敦煌历日_比较表.csv`），与主表逐月比较（main-table-checks.md 第三节）
- [x] 敦煌地方历接入换算：`data/calendar/敦煌朔日表.csv`（254 月）→ `calendar::dunhuang`（第 2 层）、`lifa::resolve_dunhuang_day`
  （第 4 层，无据之月缺省退回主表，可选只认有据）、`era_names::dunhuang_year_name`（吐蕃地支、归义军奉中原年号）；比较表 257 格换算全部重现；
  网页「历日」分政权一栏有据之日显示敦煌（`doc/calendar/dunhuang-calendar.md`）。下编查核：882 剑南印本、978 斯0612（宋应天历官本，正月丙戌朔、
  355 日合主表）非敦煌自造之历，928 北图新0836 只存历序，均无逐月可录
- [ ] 978 年斯0612 历序「六十相屬宫宿法」所列 919—978 年历年閏月与主表逐条核（第 1 级旁证）

**P2**
- [ ] 后秦、北凉、蜀推步表检验：《晋书》载记（卷一百十六至一百十九、一百二十九）、《魏书》卷九十五与九十九、《宋书》卷九十八均无「月+日干支」纪日，
  《三国志》蜀书仅 4 条（已入 sanguozhi），史书无可用之证。须出土纪日：王素《吐鲁番出土高昌文献编年》（1997）、《吐鲁番出土文书》第一册、
  殷光明《北凉石塔研究》（2000）——待取得后转录入 `data/calendar/出土纪日.csv`
- [ ] 推步已知例外之因：北周天和二年闰六、天和/大象数月朔早一日（push-algorithms.md §3.3、§3.4；常数已与《律历志》核合）。
  北魏太和四年闰九为纪日误认（已解决），北齐天保二年为天保历行用年（已订正，§3.6）

**P3 缺数据或长期**
- [x] 金历：重修大明历定朔复原（`calendar::daming_li`，表据点校本《金史》），推步政权「金」1137—1234；1182 年起术文原限合本纪 192/192，
  1137—1181 杨级历术不传，以之代推，进朔限经验加 0.25 时后 144/144（样本内；historical-calendar.md「金大明历」）
- [x] 辽历：贾俊大明历术不传；历象志所载祖冲之大明历平朔合本纪 71%（偶合率），定朔 95%——辽行定朔，暂按宋历（`liao_calendar_check.py`）；
  不合宋历之十条进朔两向皆有，不足以复原
- [x] 《开元占经》卷一百五对勘（`开元占经_积年表.csv`）：玄始常数得证，平朔十一历全合；卷一百三《麟德历经》未与《新唐书》逐表对勘；PDF 暂未入库
- [x] 《宝祐四年会天历》转录核验：主表 12/12（总录第八节）；PDF 暂未入库，待用户定
- [ ] 《俄藏黑水城文献》：第 2—5 册目录只见 TK269 无纪年残历；第 1 册目录与《俄藏敦煌文獻》第 1 册未查；世俗文书所在后续各册本地没有（是否在那里未核）
- [ ] 用《中国古代历法》核对其余「本项目解释」（戊寅差法、观天公式等），只算第 5 级旁证；乾符四年（877）敦煌历待找
- [x] 《吐鲁番出土高昌文献编年》：标题少干支，北凉无可检之条；高昌 11 条卅日中 4 条主表、玄始皆不合（`高昌_卅日.csv`）
- [ ] 高昌历：须《吐鲁番出土文书》、墓砖录文中的干支朔与闰位
- [x] 授时、大统定朔复原：本纪 388/392（通轨应数）、179/181；元 1300 年十月、1335 年八月本纪与推算、天象皆不合，未解
- [x] 两汉本纪纪日（太初后）与三统、四分推步：主表即纯推步（2321/2325、1681/1682）；本纪分不出高下
- [ ] 汉简历谱（居延、敦煌、尹湾等出土历日）转录：两汉唯一第 1 级证据
- [x] 清时宪：不复原；本纪明书朔日 633/653，主表 3275/3303 月与北京真合朔同日，余皆在子夜前后 23 分钟内
- [x] 古六历积年核验（《开元占经》）：黄帝、夏、殷、周（占经脱「七」）一致；颛顼元日己巳、鲁历闰余一（积年作 2761514）已解
- [x] 《春秋》经文检验：春秋历 333/386、书朔 26/29（今人排定，非独立）；古六历无一通贯；37 条日食 28 条当日确有，27 条曲阜可见
- [x] 《左传》传文 177 条：夏正差两月此法验不出；襄二十七、昭二十二传文合王韬月序，不合张培瑜
- [x] 表 3-7 王韬栏转录比较：经 336 对张 333；传 152 对张 138，优势 15/18 集中在襄二十七、昭二十、昭二十二
- [ ] 战国：秦简等出土历谱检验古六历（需资料）
- [ ] 杨级大明历复原：常数不传，只知历元 383768657、日法 5230；可试以本纪 1137—1181 反求（证据少，未必可定）
- [ ] 清初 1645—1673 距星以《灵台仪象志》近似（缺《崇祯历书》距星；已入库的故宫珍本丛刊第 382 册只含五纬表、五纬历指，无恒星表）
- [ ] 《崇祯历书·五纬历指》卷九崇祯年间历局实测五星（`doc/references/calendar/[徐光启1634]崇祯历书_五纬表_五纬历指（故宫珍本丛刊382）.pdf`，书页 = PDF 页 − 4，约书页 300—313）：
  逐条录文（日期、实测宿度，并列大统、新法所推），以 DE441 检验实测，并比较大统、新法推算之误差；无文字层，须识读扫描
- [ ] 古历复原：招差术（迟疾历、盈缩历、三次差内插）与定朔定气；唐麟德、大衍、宣明历定朔与进朔复算后，判定主表疑误之月
  （戊寅、麟德、大衍、五纪、正元、宣明、崇玄已复原（总录 `doc/calendar/calendar-compendium.md`）：756、900 年《旧唐书》诸条皆不合其历，从主表；703 年九月未解；821 年属观象历，术不传）；891 年《旧唐书》三条早一日待查；宋历已复原崇天、明天、观天、纪元、统元至成天；应天至仪天读法未定，暂缓（总录 §7）
  （武周长安三年九月三书皆庚寅朔而主表己丑，及《旧唐书》756、821、900 年四条，见 historical-calendar.md「以本纪核验主表」）
- [ ] 《旧唐书》本纪明书朔日「其他」类 45 条（干支与主表差 3—28 日）逐条排查传写之误与纪日解析之误
- [ ] 古历复原：ziOffset 建正映射与春秋历、HanZhuanXu 完整 Rust 移植
- [ ] 与 `实历朔日表.csv` 批量对历验证脚本
- [ ] `push_calendar` 与 `ancient_restore` 分数版结果对照测试

**P4 低**
- [ ] 《资治通鉴目录》逐年比对：维基文库两本皆无正文，须识读共享资源扫描本（约千页）；其朔闰据刘羲叟《长历》，亦为复原

---

*本清单路径均相对仓库根目录。*
