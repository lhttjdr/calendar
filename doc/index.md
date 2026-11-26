# 文档索引

> 本目录是开发与 AI 用的知识库（含代码路径、逐条校验与交叉引用），不发布。面向读者的文档在仓库 `site/`，发布于 <https://lhttjdr.github.io/calendar/docs/>，内容由本目录改写而来。

> **预览**：本目录宽表多，VS Code 预览请在工作区设置加 `"markdown.styles": ["doc/markdown-preview.css"]`（`.vscode/` 不入库），表格改为横向滚动、单元格不折行。

本仓库文档按主题分为五部分：**总体架构**（三大域与域内四段结构）、**天文基础**（时标、参考系、岁差章动、历表、视位置、月相节气等推导）、**农历编算**（由天文量算出农历、实历与古历复原）、**工程实现**（Rust/WASM/前端与数据格式）、**案例分析**（具体问题的排查记录）。参考文献见 [doc/references/](https://github.com/lhttjdr/calendar/tree/master/doc/references)；编算农历网页见 [ChineseCalendar](https://ytliu0.github.io/ChineseCalendar/computation_simp.html)。

## 〇、总体架构（`architecture/`）

| 文件 | 内容 |
|------|------|
| [three-domains.md](architecture/three-domains.md) | 天文/历法/民俗三大域，域内共同的"独立基础层→聚合/实算层→派生层→仲裁/路由层"四段结构，各域具体模块映射 |

## 一、天文基础（`astronomy/`）

依《月相和二十四节气的计算》章节顺序，从时标一路推导到视位置、月相与节气。

| 文件 | 内容 |
|------|------|
| [time-and-timescales.md](astronomy/time-and-timescales.md) | 时间与儒略日、时标、ΔT；时标换算现状（`TimePoint::to_scale` 经 TT 中转、UTC 闰秒表、UT1 = TT − ΔT） |
| [reference-frames.md](astronomy/reference-frames.md) | 参考系与坐标、Frame Bias、管线三正交与设计原则、几何与架 |
| [precession.md](astronomy/precession.md) | 岁差（P03、Vondrák 2011） |
| [nutation.md](astronomy/nutation.md) | 章动（IAU 2000A） |
| [ephemerides-and-de-align.md](astronomy/ephemerides-and-de-align.md) | 历表与 DE 对齐（VSOP87、Chapront、ELPMPP02） |
| [model-pairing.md](astronomy/model-pairing.md) | 模型配套表：历表、ΔT、标架、岁差章动各以何为基准，以 DE431 为枢纽；不允许的组合与代码保证 |
| [light-time-and-aberration.md](astronomy/light-time-and-aberration.md) | 光行时与光行差 |
| [light-deflection.md](astronomy/light-deflection.md) | 太阳引力光线偏折：公式（SOFA iauLd/iauLdsun 同式）、管线位置与选项、对 SOFA 与 Skyfield 的逐条检验、对天象检验结果的影响 |
| [apparent-longitude-and-syzygy.md](astronomy/apparent-longitude-and-syzygy.md) | 视位置计算（因素与两条路径、Rust 管线 §7.2、其它效应） |
| [lunar-phases-and-syzygy.md](astronomy/lunar-phases-and-syzygy.md) | 月相与合朔、望/上弦/下弦、W0 公式、精度事实 |
| [solar-terms.md](astronomy/solar-terms.md) | 二十四节气与定气、八步 pipeline、节气朔望标准时刻表格式 |
| [celestial-phenomena.md](astronomy/celestial-phenomena.md) | 天象模块：日月食、行星会合与视星等（含火星 IAU WGCCRE 2015 自转模型与直下点）、近距与月掩星、伏见可见性（含月亮、行星眩光源）、恒星与二十八宿（含入宿度、去极度、古代黄道度与现代坐标的关系）；设计、实施进展与校验结论 |
| [sky-view-3d.md](astronomy/sky-view-3d.md) | 3D 星空：站心、折射、气象、海平线俯角、视半径、天色、月亮眩光、月相、日食、残余光、日冕、银河、地形、历史地理调研、播放与限制 |
| [phenomena-checks.md](astronomy/phenomena-checks.md) | 天象模块逐条校验结果（脚本生成）：NASA 日月食典、史书日月食与凌犯、Skyfield 对照、原文表格复现等 |
| [chongzhen-wuwei-checks.md](astronomy/chongzhen-wuwei-checks.md) | 崇祯历局五星实测检验（生成文件）：《崇禎曆書·五緯曆指》卷九崇祯七至九年 18 条五星会合、凌犯、入宿与见伏记录，DE406 对新法与大统：位置误差中位数新法 0.12°、大统约 3 古度；15 项实测 12 相符、3 两可 |
| [chongzhen-juxing-checks.md](astronomy/chongzhen-juxing-checks.md) | 二十八宿距星：《崇祯历书·恒星历指》（1628 历元赤经、黄经积度）与《钦定天文正义》卷四（1744 历元黄道去极入宿度）逐宿对 Hipparcos；崇祯距星与《灵台仪象志》同星，清初 1645—1673 认星无误差；天文正义属《仪象考成》体系 |
| [geomag.md](astronomy/geomag.md) | 地磁：世界磁场模型 WMM2025（NOAA/BGS）磁偏角、磁倾角、总强度；官方 100 点与 12 点测试值逐条检验；网页罗盘磁北→真北改正与校准提示 |

## 二、农历编算（`calendar/`）

从天文量到农历日期的编算逻辑，以及历史区间的实历与古历复原。

| 文件 | 内容 |
|------|------|
| [lunar-calendar.md](calendar/lunar-calendar.md) | 农历编算、流程与分层 |
| [calendar-layers.md](calendar/calendar-layers.md) | 历法域内部四层结构（独立基础层/聚合实算层/派生层/仲裁路由层），`rust/core/src/calendar/` 模块划分依据 |
| [rili-genealogy.md](calendar/rili-genealogy.md) | 历表与历日资料的源流：实物历日汇编、古人复原（杜预、刘羲叟、汪曰桢等）、今人历表（陈垣、方诗铭、张培瑜、洪金富等）与其底本关系，今人历术复原，实物出处核查，待办 |
| [rili-sources.md](calendar/rili-sources.md) | 历日的来源与取舍（中原一线）：实物历日、按术推步、今人历表（张培瑜，撤「张表」概念）、现算的次序与选项，张表来历，推步与张表逐月差异，同类著作 |
| [jindai-calendar.md](calendar/jindai-calendar.md) | 近现代中原一线历日（1914—2016）：按当时编算规则现算（定气定朔，北京地方平时 / 东经 120°），依据（三年历书凡例、陈展云、天文研究所《天文年历》1930 南京平时、1931 起东经 120°、商务本 1940 东经 105°）、1924 年起官历不载旧历、1923 年官历 12 朔逐月核对、存疑之月 |
| [jindai-checks.md](calendar/jindai-checks.md) | 近现代历日逐月检验（生成文件）：1274 月与张表、与另一定日基准之比较，合朔/中气近子夜者 |
| [historical-calendar.md](calendar/historical-calendar.md) | 实历模块：历史朔日表、优先实历的公历↔农历换算 |
| [benji-checks.md](calendar/benji-checks.md) | 以本纪核验历日逐条结果（脚本生成）：明书朔日、实际合朔、閏月位置 |
| [benji-qita-kaozheng.md](calendar/benji-qita-kaozheng.md) | 本纪明书朔日「其他」类（差 3 日以上）逐条排查（生成文件）：一字误、「朔」字衍、年月误、食日书朔、未明，附已改正的纪日定年之误 |
| [dunhuang-tuban-checks.md](calendar/dunhuang-tuban-checks.md) | 敦煌历日图版核对（生成文件）：比较表伯希和诸件逐格对《法藏敦煌西域文献》图版，原件格 78/86、推算格 27/65 得证，不合 0 |
| [dunhuang-calendar.md](calendar/dunhuang-calendar.md) | 敦煌历日换算（吐蕃、归义军）：出土历日有据之月作「敦煌」政权历表，无据之月退回中原历的规则、纪年，比较表逐格重现检验 |
| [ancient-calendar-restore.md](calendar/ancient-calendar-restore.md) | 古历复原：分数推步、平朔平气、历谱生成；三千五百年仅作验证 |
| [guliuli-shuo.md](calendar/guliuli-shuo.md) | 古六历书朔逐历分析：七套参数的平朔相差、各时段平朔与天文真朔之差，逐条对春秋经传书朔、《汉书》日食朔晦、实历朔日表与秦汉初复原历 |
| [tongjian-mulu-checks.md](calendar/tongjian-mulu-checks.md) | 《资治通鉴目录》（刘羲叟《长历》）朔闰逐条检验（生成文件）：328—959 年连续 632 岁，中原一线合张表 7673/7792、合推步 7399/7602，北朝诸国行合分政权推步 1921/1935（魏历不合处皆闰位差一月）；麟德 670、692、697、722 从张表，738 大衍从推步，716 两家各半；目录独异处本纪多不从目录（宋人复原，只作对照） |
| [dingshuo-zhangbiao-diffs.md](calendar/dingshuo-zhangbiao-diffs.md) | 定朔诸历推步与张表朔日不同之月逐条考证（214 月、闰异 7 岁；797 年二月依史入例外表）：定小余、进朔限与距限，张表统一四分之三进朔 111 月，本纪/纪日/节日证据，703 年九月庚寅、891 年旧纪三条之精算 |
| [calendar-compendium.md](calendar/calendar-compendium.md) | 历代历法总录：总表与已复原定朔历（戊寅、麟德、大衍、五纪、正元、宣明、崇玄、金大明）的常数、表、算法、检验；辽（调元、贾俊大明以宣明代推，947—1087） |
| [calendar-source-texts.md](calendar/calendar-source-texts.md) | 历法原文摘录（脚本生成）：唐诸历、重修大明历、庚午元历表、《辽史》大明历 |
| [push-algorithms.md](calendar/push-algorithms.md) | 各朝代/政权历法推步算法与历术参数总结 |
| [xixia-calendar.md](calendar/xixia-calendar.md) | 西夏历日 1120—1207：彭向前整理俄藏 Инв.No.8085 等历书之朔闰对照表转录、核对（宋历一栏与张表全合）、接入推步政权表 |
| [baoshan-checks.md](calendar/baoshan-checks.md) | 包山楚简纪日逐条检验（生成文件）：77 条纪日在颛顼、殷、鲁、周、黄帝、夏诸历中为月中第几日，各假设合数 |
| [shuihudi-checks.md](calendar/shuihudi-checks.md) | 睡虎地秦简《编年记》《语书》与魏律纪日 10 条逐条检验（生成文件）：颛顼历合 8/8，汉传六历无一全合；秦王政二十年相位窗口与统一后秦历不连续；魏律闰再十二月惟夏历合 |
| [xincai-checks.md](calendar/xincai-checks.md) | 新蔡葛陵楚简纪日检验（生成文件）：31 条岁名·月·干支，组内自洽、前 450—前 350 年逐年相位窗口 |
| [chu-sky-checks.md](calendar/chu-sky-checks.md) | 楚简纪日与实际月相对照（生成文件）：包山 87 条、新蔡前 398 年 19 条在颛顼平朔与看天（合朔日 + 0/1/2 日）下逐条合否 |
| [s0612-runyue-checks.md](calendar/s0612-runyue-checks.md) | 斯0612 历序閏月（生成文件）：978 年应天历官本历序「六十相屬宫宿法」所列 919—978 年逐年閏月，与张表、中原一线比较（59/60 合，936 年漏书） |
| [zoumalou-checks.md](calendar/zoumalou-checks.md) | 走马楼吴简纪日（生成文件）：竹简壹至叁、《简牍集成》中带日序干支或书朔之简逐条（已对原书图版核对），与吴乾象推步、魏历比较，及吴、魏朔不同之月按月归并 |
| [gaochang-630-checks.md](calendar/gaochang-630-checks.md) | 高昌延寿七年（630）残历各月朔与张表、玄始历外推（生成文件）：有据 11 月，合玄始 10、张表 9 |
| [tulufan-jiri-checks.md](calendar/tulufan-jiri-checks.md) | 吐鲁番出土文书与墓表纪日检验（生成文件）：麴氏高昌朔日 7 条合玄始外推 5、张表 3，闰月 4 条合玄始 0、正光外推 3；630 年十二月差一日非进朔，620 年前后高昌历元或改；北凉无可检之条 |
| [chutu-rili-checks.md](calendar/chutu-rili-checks.md) | 吐鲁番、黑城出土唐宋历日残件（720、1182、1211）月朔与张表、中原一线（生成文件）；多为定年所用，独立者一月 |
| [yinwan-lipu-checks.md](calendar/yinwan-lipu-checks.md) | 尹湾汉墓元延元年、三年历谱与元延二年日记六单月，与张表、三统推步（生成文件）：20 月全合 |
| [hanjian-checks.md](calendar/hanjian-checks.md) | 居延汉简、居延新简、敦煌汉简、肩水金关、悬泉汉简公文书朔日 451 条与张表、两汉推步（生成文件）：合 424，不合 27 条（简牍集成 19 条已对图，肩水金关释文 3 条，悬泉 5 条对书影）；同简异本重出 1 条不计 |
| [zhongguo-gudai-lifa-checks.md](calendar/zhongguo-gudai-lifa-checks.md) | 《中国古代历法》逐条核对推步读法 23 条：戊寅差法、观天公式、统天距算相合；崇玄定朔（志言同大衍）、明天「二至/二百」等相异；宣明清明限疑书误；乾符四年历日著录 |
| [chunqiu-luli-checks.md](calendar/chunqiu-luli-checks.md) | 春秋鲁历据经文排定（生成文件）：月首模型与先验参数的边际似然比较、逐年正月与闰、标注（经文定/按建推定/未定）与后验、张表 3-7 对照 |
| [regime-benji-checks.md](calendar/regime-benji-checks.md) | 分政权推步之史书逐条检验（生成文件）：吴、蜀、北朝诸国、辽、金等本纪纪日，以本国推步与中原张表两家判在月 |
| [jiudian-yingshi-checks.md](calendar/jiudian-yingshi-checks.md) | 九店简 96「荆𡰥朔於营室」检验（生成文件）：前 350—前 250 逐年荆𡰥朔太阳赤经，亥正、丑正两假设下是否在营室 |
| [zhanguo-calendars.md](calendar/zhanguo-calendars.md) | 春秋鲁历与战国古六历按国别接入：各国用历证据（秦颛顼、魏夏历、楚有争议）、汉传六历整数推步与《中国古代历法》表 3-17/3-18 逐格核对、年终闰与年中无中气两种闰法 |
| [qin-han-calendar-evaluation.md](calendar/qin-han-calendar-evaluation.md) | 秦与汉初（前221—太初元年四月）历法复原接入评估：与张表衔接、《汉书》检验、待补的日月食与出土材料验证 |
| [qinhan-shiwu-checks.md](calendar/qinhan-shiwu-checks.md) | 秦汉初出土历日（岳麓、周家台、张家山、里耶、转引元光元年历谱）逐月检验：各段合朔改正 δ 的可行区间与定界之条、实物对实用历推步/颛顼推步/李忠林复原历、前 221—太初元年四月各月依据分类（实物年/唯一/区间未定/段界未定）、与复原历不同之月（自动生成） |

## 三、工程实现（`engineering/`）

项目结构、Rust/WASM/前端实现与数据格式。

| 文件 | 内容 |
|------|------|
| [project-and-implementation.md](engineering/project-and-implementation.md) | 项目结构、Rust 实现、Real 后端比较、f64 约定与物理量审计 |
| [wasm-react-performance.md](engineering/wasm-react-performance.md) | Rust+WASM+React 性能调优与边界优化 |
| [ephemeris-binary-format.md](engineering/ephemeris-binary-format.md) | 历表二进制化与零解析方案（VSOP87/ELPMPP02） |
| [data-paths-summary.md](engineering/data-paths-summary.md) | 历表与数据路径总结（VSOP87/ELPMPP02/DE406 路径、标架、时间尺度） |
| [pipeline-graph-taxonomy.md](engineering/pipeline-graph-taxonomy.md) | 管线图：节点与边的分类与抽象（OriginRole、TransitionKind/Form） |
| [ci.md](engineering/ci.md) | 持续集成：Pages 部署与覆盖率工作流的 LFS 数据、DE406 缓存、工具版本、本地模拟 CI |
| [web-folk-pages.md](engineering/web-folk-pages.md) | 网页择日页（年神二十四山罗盘图、神煞宜忌）与排盘页（奇门九宫、六壬天地盘、太乙盘）；GPS 定位、罗盘（磁北经 WMM 磁偏角改真北、平放检测） |
| [ui-interaction.md](engineering/ui-interaction.md) | 网页交互设计：全站一份时空上下文（时刻、地点）常驻顶栏、网址即状态、每页「结论→依据→设置」三层、盘面与文字联动、手机底栏；现状问题与分期 |
| [site-writing-guide.md](engineering/site-writing-guide.md) | 读者站写作规范：原理推导、源流、各家并列、原典核对与存放、生成页处理、多会话协作 |

## 四、案例分析（`case-studies/`）

| 文件 | 内容 |
|------|------|
| [de406-solar-term-failure-analysis.md](case-studies/de406-solar-term-failure-analysis.md) | DE406 定气 vs TDBtimes 失败原因分析 |

---

**参考文献**：[doc/references/](https://github.com/lhttjdr/calendar/tree/master/doc/references)；仓库根 [README.md](https://github.com/lhttjdr/calendar/blob/master/README.md) 快速开始与项目结构总览。AI 辅助开发用的项目总览见仓库根 [INVENTORY.md](https://github.com/lhttjdr/calendar/blob/master/INVENTORY.md)（不纳入本文档站）。

## 五、民俗（`folk/`）

| 文档 | 说明 |
|------|------|
| [ganzhi-calendar.md](folk/ganzhi-calendar.md) | 干支历四柱：换年、换月、闰月、换日各选项与预设的原典出处、核对状态、代码位置 |
| [nayin-zhiri.md](folk/nayin-zhiri.md) | 纳音（三种推法）、二十八宿值日与七曜、年月紫白九星：原典依据、时宪书实物核对、代码位置 |
| [zajie.md](folk/zajie.md) | 杂节气：三伏、社日、寒食、腊日、入梅出梅、土王用事、数九、七十二候的规则、出处、节气当日计入与否、检验 |
| [gengdian.md](folk/gengdian.md) | 更点时刻：元授时法与清台官法、日出入与朦影的球面三角、协纪京师更点表核对 |
| [jianchu.md](folk/jianchu.md) | 建除十二值、黄道黑道十二神、百忌日：协纪原文、起例公式、时宪书与协纪按语核对 |
| [shensha.md](folk/shensha.md) | 神煞宜忌（进行中）：月家日神煞数据表与协纪改定；协纪月表宜忌推算与 720 日逐页核对（合 701、疑 19）；年份节气朔望诸日的铺注与乾隆三十年时宪书逐日核对；各家通书对照计划 |
| [tongshu.md](folk/tongshu.md) | 各家通书对照（一）：元末《類編曆法通書大全》与协纪月家 63 神、年神逐神对照（同 64、異 12、类编无 27、协纪无 28），协纪「通書誤以」诸条（鸣吠、复日、天火、蚕命等）对上原文；类编独有之神；类编起例选项（天德四仲、天火顺行、鸣吠鸣吠对、复日四说、荒芜、天贼、蚕命；`ShenshaOptions`、`NianShenOptions`，协纪缺省）及与类编原表逐日检验；类编卷十一逐月日表 720 日转录（`leibian_juan11_days.csv`）与卷十五、本项目起例（`shensha::tests::leibian_juan11`）、协纪月表逐日互校（天贼全异，类编各卷天贼起例同一，已做选项）；日游神鹤神未见、卷十三独有年神在协纪各卷的去留、木马杀四月定申；《選擇宗鏡》抄本 57 页目次、题记、与协纪所引对勘（协纪删荒芜等句） |
| [tongshu_yiji.md](folk/tongshu_yiji.md) | 各家通书对照（宜忌）：《董公選擇日要覽》按月×建除逐日宜忌与协纪月表 664 日、5991 条对照（合 55%、半合 14%、異 24%；分事、分建除、正月逐日），分歧类型（煞集中宫、黄罗紫檀诸星、德神相抵、粒度）；鳌头录文与对照（卷一十二星斷逐字、卷三求婚下定嫁娶備急四表与协纪月表、董公、类编逐日对照、鸣吠鸣吠对从类编不从协纪；卷二至卷四出行、上官、開張、立契、入學、行船、動土、起工、豎柱上梁、蓋屋、入宅、斬草、安葬十三门逐月吉日表 1187 日与协纪、董公逐日对照及按本书起例自检；天德四仲、天火、复日、荒芜、天賊亦从类编，各表按旧起例筛日，只安葬表依「曆法」删十月开日；`aotou_*.csv`、`folk::aotou`、`ShenshaOptions::aotou()`）；宗镜属造命；董公作择日页第二套宜忌（`folk::dong_gong`、`dong_gong_day_json`；原文表、逐日判定表 681 日逐条校订、事目映射） |
| [aotou-qili-checks.md](folk/aotou-qili-checks.md) | 生成文件：鳌头卷二至卷五凶神吉神起例与协纪月表、项目实现逐条比对（315 条），附天愿、地囊、四击、枯焦、天狱考 |
| [aotou_men_checks.md](folk/aotou_men_checks.md) | 生成文件：鳌头卷二至卷四十三门逐月吉日表逐日对照（协纪月表、董公、本书起例所值、表尾所避之神自检、天賊天火重喪两家起例对比），由 `scripts/folk/aotou_men_vs_xieji.py` 生成 |
| [nianshen.md](folk/nianshen.md) | 年神方位：协纪卷三诸年神起例、六十年同位关系检验、乾隆三十年时宪书年神方位之圖逐位核对 |
| [jiaqu.md](folk/jiaqu.md) | 婚嫁择吉：协纪为纲（月表铺注宜忌、周堂、不将，大利月诸说奏准删除）与类编卷七并列（女命行嫁月、阴阳不将起例、嫁娶凶神表、白虎入中宫、三种周堂、逐月求婚下定嫁娶吉日表录文与逐条比对；卷十五赤口小六壬；协纪卷三十六所述兀日起法即类编卷二六輪兀，与起例图、上下兀日表逐格比对；闰月从本月）、鳌头橫天朱雀；制化（周堂回避、白虎禳法，缺省不用）三态判定；逐条检验；诸说补全（第三之三节）：納采問名入重推、铺注九项与歲薄逐陣陰陽錯可逐项不看、协纪改删旧说与类编鳌头董公诸神 19 位逐神可否决（书例与 2026 年逐日检验）、董公婚姻宜忌与鳌头婚姻四表接入、可取舍诸项各书之说目录（`jiaqu_catalog`） |
| [jiaqu_shensha_inventory.md](folk/jiaqu_shensha_inventory.md) | 婚嫁择吉神煞清点：协纪卷十一婚事三条宜神忌神、卷十特定日与铺注、卷三十五附录、卷三十六辨讹与卷首奏议，类编卷七、鳌头卷三嫁娶门（读图补录紅嘴朱雀、婚娶忌宿、離窠、女嫁厄年等）、董公诸日表，逐神列起例、所主、协纪之断（用改删斥不载）与实现状态，共 152 条（已实现 123：待实现清单 23 项 2026-11-09 全部实现，余为起例未明、配命与未见原典）；合婚配命与未见原典者（三娘煞、冲新人等）另表 |
| [qimen.md](folk/qimen.md) | 奇门遁甲定局：节气配卦局数、三元符头、置闰（演义、统宗两界）与拆补两法、行用历平气；演义万历、淳祐与统宗康熙诸例逐条检验；日家月家；格局（演义卷二 33 例）、年家奇门、飞盘依宝鉴改正 |
| [liuren.md](folk/liuren.md) | 大六壬：月将（按中气换将，卷二原注照录；按节换将为选项）与起课（十干寄宫、九法、涉害深浅），《六壬大全》卷五课例 22 则逐例检验；天乙贵人两说与十二天将，三传天将 49 则逐例检验 |
| [taiyi.md](folk/taiyi.md) | 太乙四计（岁月日时）入局与太乙宫、文昌、计神、始击、主客算将；金镜、集成、太乙秘书与淘金歌积年并列；四神天乙地乙直符、三基、五福、大游小游、定计及掩击迫囚关格对提挟诸格局（各家异说为选项）；书例逐条检验，金鏡卷三、卷五立成与太乙秘書一百四十四局逐格比对；金鏡章岁法积月与闰月、王希明二至加时起门；三元九运 |
| [ziwei.md](folk/ziwei.md) | 紫微斗数（十四主星派）：命身宫、五行局、紫微天府十四主星、辅佐煞曜、四化、长生博士、大小限；全书卷二五局图 147 日与同度四十一条逐条检验；闰月、魁钺、四化、长生顺逆诸说并列；截空旬空、流年诸星、庙旺表；韩国藏写本核定缺省；写本诸星（天官天福华盖等，选项）与流年羊陀迭并、七杀重逢、小限羊陀重逢、竹罗三限 |
| [ziwei18.md](folk/ziwei18.md) | 紫微斗数十八飞星派（续道藏本，明万历刻本校）：紫系十二星、杖异毛刃、刑姚哭、三台八座龙池凤阁、命身宫、大小限与流年；十干阴阳、"申宫"、小限两法诸说并列；卷一「廟樂旺」图逐格转录（卷三星论并列）、星之五行两说、本命星；卷一诸例逐条检验 |
| [xuankong.md](folk/xuankong.md) | 玄空飞星：运盘、山向盘（二十四山阴阳顺逆、五黄入中）、替卦（《青囊奧語》挨星诀与沈氏扩表）、兼向与空亡及界线（出线、出卦）、零神正神（《天玉經》四库本与四库注双山之读并存）、格局（旺山旺向、上山下水、双星、伏反吟、合十、三般卦、城门、七星打劫、入囚）与逐宫标记、流年流月紫白；216 局顺逆定理、432 局打劫与逐宫检验；沈氏排法出处未核 |
| [liuyao.md](folk/liuyao.md) | 六爻纳甲筮法：京房八宫世应、纳甲纳支、六亲、六神、伏神、旬空、卦身；掷钱与梅花易数（年月日时、数字）起卦、互卦；增删卜易诸例逐条检验 |
| [zhouyi.md](folk/zhouyi.md) | 《周易》筮法：大衍揲蓍成爻（概率 1/16、5/16、7/16、3/16）、朱熹《易学启蒙·考变占》取辞，六十四卦经文；左传国语筮例逐条检验 |
| [ziping.md](folk/ziping.md) | 子平命理：源流与版本，十神、藏干三表与人元司令、十二长生两说、大运起运折算诸说、小运胎元命宫身宫、空亡神煞、真詮月令取格；古籍命例逐条检验 |
| [qizheng.md](folk/qizheng.md) | 七政四余：七政视黄经、四余（罗计真/平交点、升交归属三期更替、月孛平远地点、紫炁）、中气分宫与黄道宿度、命宫；道光十三年七政时宪书逐旬四余 48 条与七政、康熙九年本逐条比对；星命（宫主度主、廟旺、恩難、量天尺与授时过宫度图部分转录、童大小月限、流年變曜、洞微飛限、竹羅三限、神煞百余项逐条对书） |
