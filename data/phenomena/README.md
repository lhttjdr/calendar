# 天象记录与检验结果

本目录放史书天象记录的逐条转录，以及用本项目天象模块（`rust/core/src/astronomy/phenomena`）算出的检验结果。
方法与结论见 [doc/astronomy/celestial-phenomena.md](../../doc/astronomy/celestial-phenomena.md) §九 P2。

## hanshu_solar_eclipses.csv

《汉书·五行志下之下》西汉日食记录（维基文库 `漢書/卷027` wikitext 解析），由
`scripts/calendar_data/qinhan_eval/hanshu_eclipse_dates.py` 生成。原文总结作「日食五十三，朔十四，晦三十六，先晦一日三」，
所列实为 54 条（晦 37 条），按所列逐条收录。

| 列 | 含义 |
|---|---|
| `emp`、`era`、`year_in_era` | 帝号、年号（文、景二帝的「中元」「後元」记在 `era`）、年数 |
| `year` | 天文纪年。太初以前十月岁首，十月至十二月属该年但在公历上一年 |
| `month`、`ganzhi`、`position` | 月、日干支、朔/晦/先晦一日 |
| `extent`、`lodge` | 食甚描述（既、幾盡、不盡如鉤）、所在宿度 |
| `calendar` | 日期换算所据：`复原历`（太初以前，李忠林 2012，`hzx.py`）或 `实历朔日表`（太初以后，vendor） |
| `jdn`、`julian`、`lunar_day` | 记录日的儒略日数、儒略历日期、该月第几日 |
| `note` | 干支不在所记之月等问题 |
| `text` | 原文首句 |

日期只由历法换算，不用天象，以保证下面的检验独立。

## houhanshu_solar_eclipses.csv、houhanshu_lunar_eclipses.csv

《后汉书》日月食记录，由 `scripts/calendar_data/qinhan_eval/houhanshu_eclipse_dates.py` 生成（维基文库 `後漢書/卷108`、`卷92`）。
东汉全在太初以后，日期一律按实历朔日表换算。

- **日食**：志第十八《五行六》「日蝕」一节 72 条，与原文总结「日蚀七十二：朔三十二，晦三十七，月二日三」条数一致（永元二年二月
  壬午一条未记朔晦）。刘昭注所引《古今注》等日食不收。列同上，另有：`leap`（闰月为 1，「闰月」所闰之月从朔日表查得）、
  `reported_by`（「史官不见，某郡以闻」中报告者）。原文字误：元初四年「乙已」校作「乙巳」；延熹八年「丙甲」非干支，按「晦」定日。
- **月食**：东汉无系统月食记录。收志第十八「月蚀非其月」2 条（有日干支）与志第二《律历中》「论月食」可定月份者 5 事；对「天以某月食，
  历以某月」一类，历法预报之月也各收一行，检验其无食。列：`year`、`month`、`leap`、`day`（干支或日序，空为只记月份）、`source`、
  `jdn`、`lunar_day`、`guess_jdn`（搜索中心：有日者为该日，只记月份者为该月第 15 日）、`check`（检验要点）、`note`、`text`。

## *_check.csv（检验结果）

`examples/historical_eclipses.rs` 生成（仓库根下）：

```
cargo run --release -p lunar-core --example historical_eclipses --no-default-features --features twofloat -- \
  <solar|lunar> <输入 csv> <输出 csv> <东经°> <北纬°>
```

都城取长安（108.88°E，34.30°N，西汉）、洛阳（112.62°E，34.73°N，东汉，汉魏洛阳城）。输出保留输入各列（不含 `note`、`text`），后接：

| 列（日食） | 含义 |
|---|---|
| `greatest_tt_jd`、`kind`、`gamma`、`global_magnitude` | 记录日 ±1.5 日内日食的全球极大食（TT）、类型、γ、食分；空表示无日食 |
| `day_offset` | 都城地方平时下的日食日期减记录日（0 = 记录日即日食日） |
| `delta_t_s`、`sigma_s` | Morrison–Stephenson 2021 ΔT（已与月历表 ELPMPP02 配套修正：缺省按 ELPMPP02 − DE431 实测月亮黄经差，可选按 ṅ 二次修正）及其 σ |
| `site_magnitude_*` | 都城在 ΔT −2σ、−σ、0、+σ、+2σ 下所见最大食分（日面在标准地平高度以上的部分；0 为不可见） |
| `sun_altitude_greatest_deg` | ΔT 取中值时都城当地食甚的太阳几何高度 |
| `verdict` | 无日食 / 都城可见 / 都城不可见 / 都城可见与否取决于ΔT |

| 列（月食） | 含义 |
|---|---|
| `greatest_tt_jd`、`kind`、`gamma`、`penumbral_magnitude`、`umbral_magnitude` | 搜索中心 ±1.5 日内月食的食甚（TT）、类型（半影/月偏/月全）、γ、食分 |
| `local_start_h`、`local_end_h` | 本影食（半影食则为半影食）始终的都城地方平时（时） |
| `day_offset` | 食甚的都城地方平时日期减记录日（只记月份者为空）；古人以天明为日界，凌晨的食记在前一日，故凌晨食 +1 属正常 |
| `site_magnitude_*` | 都城在各 ΔT 下月亮上缘在地平以上时的最大食分（本影食取本影食分，半影食取半影食分） |

日月食的有无、类型、食分已与 NASA 五千年日月食典（`vendor/nasa_eclipse/`）逐条核对，见 doc/astronomy/celestial-phenomena.md §九 P2。

## schaefer1987_table1.csv

Schaefer B.E. 1987, Heliacal rise phenomena（*JHA* 18, Archaeoastronomy 11, S19）表 1 逐行转录：作者 1983—1986 年肉眼观测的
45 条恒星伏见记录（原文称共 556 条，表 1 为其选录）。对原版面逐行核对，并按原文脚注自检：「Δ_obs 与 Δ_pred 的不等号」不成立
4—7 日标 ?、8 日以上标 ??，与转录一致（5 条未标者均带 ZL 或 Q 注记，原文正文说明这类不计）。

| 列 | 含义 |
|---|---|
| `star`、`hip` | 星名、Hipparcos 编号（CDS Sesame 名称解析；ρ Sgr 为 ρ¹ Sgr） |
| `rise_set` | R 偕日升（晨），S 偕日落（昏） |
| `lat_deg`、`k`、`mz` | 观测纬度、当晚 V 天顶消光、暗夜天顶极限星等 |
| `delta_obs`、`delta_pred` | 观测日、作者 1987 模型预报日（春分后日数） |
| `symbol` | 原文「Visibility」：`>>` `>` `>~` `<~` `<` `<<` 依次为 ≫ > ≳ ≲ < ≪；升时为极易见、可见、勉强可见、疑似未见、不见，落时方向相反 |
| `comments` | ZL 黄道光干扰、MW 银河干扰、Q 模型对 k、m_z 极敏感、?/?? 见上 |

用于 `rust/core/src/astronomy/phenomena/tests_heliacal.rs` 的 `schaefer1987_table1_observations`（各可见性模型组合与实测比对）。

## houhanshu_star_records.csv

《后汉书·天文志》（志第十至十二，维基文库 `後漢書/卷100–102`）中月与五星「犯、入、乘、守、掩、食、在」记录，由通用解析器
`scripts/calendar_data/qinhan_eval/zhi_star_records.py houhanshu` 解析（2026-10-28 起；此前的专用脚本 `houhanshu_star_records.py`
改为共用工具库。改用后 150 条，旧脚本 116 条全部包含在内，新增多为多天体同在一宿与桓帝以下简体段；`source` 列记「正文」或「古今注」）：
- 收正文与刘昭注所引《古今注》；日期须与事件同句（或与上一条记录同句），占辞复述不收；
- 日期按实历朔日表换算（`in_month = 0` 表示日干支不在所记之月，取最近同名日）；
- 目标匹配陈卓体系星官（`target_id` 对应 `data/sky_culture/chinese_asterisms.csv`）；「在某宿某度」另记 `lodge`、`lodge_deg`。

逐条推算结果在 `checks/houhanshu_star_records.csv`（见下）。

## hanshu_star_records.csv、jinshu_star_records.csv

格式同 `houhanshu_star_records.csv`（列 `bodies` 可含多个天体，`jdn_start`—`jdn_end` 为日期范围：记日干支者为当日，只记到月、旬者为整月、整旬）。
- 《汉书·天文志》（卷二十六，`scripts/calendar_data/qinhan_eval/hanshu_star_records.py`）：
  - 汉初帝号纪年（孝文後二年、孝景中三年……）；
  - 太初以前月份按秦汉初复原历（`hzx.py`，十月岁首），以后按实历朔日表；
  - 多星相合（「水、木、火三合於東井」）作一条；
  - 每段开始时日期上下文恢复为上一段段首纪年，以免占辞后的应验叙述（「至河平元年三月，旱」）改写下一段的日期。
- 《晋书·天文志》（卷十二「月奄犯五纬」「五星聚舍」、卷十三「月五星犯列舍经星变附见」，`zhi_star_records.py jinshu`）：
  - 年号由 `vendor/calendar/chinese_calendar/年号表.csv`（魏、晋主线）取元年，同名年号取不早于前文年份且最近者；
  - 「升平」在升平五年（361）后的记载为前凉沿用晋年号，照年号表连续计年。
- 《宋书·天文志》（卷二十三至二十六全卷，`zhi_star_records.py songshu`）→ `songshu_star_records.csv`：魏、晋、宋（220—479），
  魏晋部分与《晋书·天文志》多有重出，可互校；南朝宋按实历朔日表（主表此段即南朝历）；观测地 317 年前洛阳、其后建康。
- 《魏书·天象志》（卷一百五之二「月」、之三、之四「五星」，`zhi_star_records.py weishu`）→ `weishu_star_records.csv`：北魏、东魏
  （386—550）。年号取并存政权表（`data/calendar/汇总/era_names_merged.csv` 的北魏、东魏）；日期按北朝推步历
  （`data/calendar/推步政权朔日表.csv`，398 年以前无表，该段日期不收）；观测地平城（494 年起洛阳），东魏邺。
  之三、之四为后人所补。
- 《南齐书·天文志》（卷十二「月暈犯」、卷十三「五星相犯列宿雜災」，`zhi_star_records.py nanqishu`）→ `nanqishu_star_records.csv`：
  齐（479—502），建康；多以日序记日（「十二月二十六日」），`ganzhi` 列记作「26日」。卷十二「月暈犯」一处「永元元年正月己亥」
  校作「永明元年」：此段承建元四年，其后连记至「十一年」（永元只三年、永明十一年），按永明计日干支在月 122/132 条、按永元计 23/132 条。
- 《隋书·天文志》（卷二十一「五代災變應」，`zhi_star_records.py suishu`）→ `suishu_star_records.csv`：梁、陈、北齐、北周、隋
  （502—618）。一书兼记南北，按段首「朝代＋帝号」（「梁武帝」「陳武帝」「齊文宣帝」「周閔帝」「高祖文皇帝」等）切换政权，
  句中「周將……」「齊……」不切换；`regime` 列记该政权。年号只在当时政权的年号表中查（同名年号如北齐天保与后梁天保分开）；
  周閔帝、明帝无年号，以帝号纪年。历表按政权：梁、陈及 589 年平陈后的隋用实历朔日表，北齐用东魏/北齐推步历，北周与
  589 年前的隋用北魏/西魏/北周/隋推步历（`calendar` 列记「实历朔日表」或「推步:政权」）。观测地按政权取都城：
  梁陈建康，北齐邺，北周、隋长安。
- 《新唐书·天文志》（卷三十三「月五星凌犯及星變」「五星聚合」，`zhi_star_records.py xintangshu`）→ `xintangshu_star_records.csv`：
  617—907，长安（武周神都洛阳，天祐元年起洛阳）。武周建子期（載初元年至久視元年九月）月名「正月」「臘月」「一月」经
  `data/calendar/month_label_reforms.csv` 换算为建寅表月，`month` 列此时记原月名（「正」「腊」「一」）；天寶、至德以「載」纪年。
  年号取合并年号表（唐、武周、隋），武周四字年号（天冊萬歲、萬歲通天）完整。点校本校勘记（`<ref>`）多处已按《二十史朔闰表》
  指出「是月無某日」，与本项目「干支不在月」可互证。
- 《旧唐书·天文志》（卷三十六「災異」及「災異編年至德後」，至「舊儀」司天臺沿革前止，`zhi_star_records.py jiutangshu`）→
  `jiutangshu_star_records.csv`：主体为 756—845 年，大曆年间逐日记录最密。多记距离（「月去東井一尺」「熒惑去南斗五寸」），
  动词「去」「近」「逼」照收，距离留在 `qualifier` 列；天体多用简称（「歲」「鎮」「火」）；肃宗元年以斗建名月（「建子月」），
  改制表把建子、建丑月记在次年（元年）名下，「上元二年……其年建子月」按次年查。
- 《旧五代史·天文志》（卷一百三十九全卷）→ `jiuwudaishi_star_records.csv`、《新五代史·司天考二》（卷五十九）→
  `xinwudaishi_star_records.csv`：907—960，后梁、后唐、后晋、后汉、后周（`regime` 列）；都城后唐洛阳、余为开封（后晋 937 年起）。
- 《宋史·天文志》（卷五十三「月犯五緯」「月犯列舍」、卷五十四「月犯列舍下」、卷五十五五星犯列舍、卷五十六「五緯相犯」）→
  `songshi_star_records.csv`：960—1279。按天象类型编排、主语省略：卷五十三、五十四主语为月（`sections` 第四项指定）；
  卷五十五以单独成段的「歲星」「熒惑」……为小标题，其下各条主语为该星。一年之事分数段，新段不回到上段段首纪年
  （`para_reset=False`）。熙甯、崇甯用「甯」（年号表异体另列）。北宋开封、南宋临安。
- 《金史·天文志》（卷二十「月五星淩犯及星變」）→ `jinshi_star_records.csv`：1115—1234。**金历（大明历）与宋历偶有一两日之差，
  本项目无金历朔日表，暂按实历朔日表（宋历）**，干支不在月者或即其差。都城会宁（1153 年迁中都燕京、1214 年迁开封）。
- 《元史·天文志》（卷四十八「月五星凌犯及星變上」、卷四十九「下」）→ `yuanshi_star_records.csv`：1260—1368，月称「太陰」；
  点校本改字「十（二）〔一〕月」取改正后文字（删圆括号、留方括号内字；对各书一致生效）。中統年号在年号表记于「蒙古」政权下。大都。
- 《明史·天文志》（卷二十六「月掩犯五緯」「五緯掩犯」「五緯合聚」「五緯掩犯恆星」）→ `mingshi_star_records.csv`：1368—1644。
  维基文库此卷为简体，先以 OpenCC 转繁再解析。多以「A及B，C，俱犯X」并列多日，「及」后接日期者作分句，「俱/並/皆」起头的动作
  展开到其前连续的每个日期。南京（1421 年迁北京）。
- 《清史稿·天文志》→ `qingshigao_star_records.csv`（1644—1795）。卷三十至三十五为恒星、仪象等，维基文库无录文；取：
  - 卷三十六「五星合聚」：「土木金聚於降婁」「土金同躔壁三度」，单字天体连写；「同躔」记宿度，按「在」检验所在宿与度数；
    「聚於」星次（降婁、鶉尾等）不是星官，不收。
  - 卷三十七「月五星淩犯掩距」：「熒惑犯歲星於昴」「太白距填星於鬥」。卷首定义：两星相距 3′ 内为淩、月与星 17′ 内为淩、
    1° 内为犯、相袭为掩；雍正初「淩犯书」改名「相距书」后多书「距」（入相距书之限）。「於某宿」记入 `lodge` 列（无度数），
    供推算所在宿比对（此规则对各书一致生效，「月犯太白於東井」等同样记宿）。「距星東南一尺」之「距星」不作动词。
  - 宿度体系：清时宪历（1645 年起）以黄经计宿度、周天 360°，距星 1754 年前取《灵台仪象志》、其后取《仪象考成》
    （`tests_records.rs::lodge_model`）；按此「同躔」607 条同宿 576（95%，按陈卓赤道体系仅 74%），所记度数与推算 |差| 中位 0.74°。
    逐条见 `checks/lodge_degrees.csv`（`scripts/phenomena/lodge_degrees.py`，各书记有入宿度者都在内）。
  - 卷三十七日食（50 条，「某月某朔某時，日食某分，次於某宿」）→ `qingshigao_solar_eclipses.csv`
    （`scripts/calendar_data/qinhan_eval/qingshigao_eclipses.py`，朔干支与实历朔日表全合），北京所见食况：
    `cargo run --release -p lunar-core --example historical_eclipses --no-default-features --features real-f64 -- solar
    data/phenomena/qingshigao_solar_eclipses.csv data/phenomena/qingshigao_solar_eclipses_check.csv 116.39 39.91`；
    所记时辰、食分与推算比较见 `checks/qingshigao_eclipses.csv`（`scripts/phenomena/qingshigao_eclipse_compare.py`）。
    `historical_eclipses` 日食输出另有 `local_greatest_lmt_h`（本地食甚地方平太阳时）、`site_magnitude_geometric`（几何食分）、
    `local_first_lmt_h`、`local_last_lmt_h`（初亏、复圆地方平太阳时）四列。结论：可判别的 26 条中，1742 年前所记时辰多合初亏
    （12:4），此后多合食甚（2:8）。
  北京。
- 目标匹配先以 OpenCC 繁→简（`houhanshu_star_records.T2S` 只有几十字）；目标位置的天体简称（「犯歲」「犯木」）作天体；
  太微垣内上將、上相、次相、左右掖、執法、端門不分东西者按太微二垣，但同句上一条目标为房宿时「上相、次相、上將、次將」
  归房宿（房四星同名）；別名另有井→东井、權→轩辕、御女/少民→轩辕、天罇/天鐏→天樽、天鑰→天籥、牛→牵牛、星→七星、
  庶子→心、氏→氐（误字）。
- 句式（对各书一致生效）：动词前可有「退行/逆行/順行/退/又/復」（「月又犯填星」此前漏收，晋书至新唐书共补收约 150 条）；
  「入犯」「掩食」「淩」作动词；主语省略的「與太白合」、目标省略的「又犯」「復犯之」（承上一条目标）。年数「元」只单用
  （「至元元年」）。
- 日月食（2026-10-28 起，`zhi_star_records.py` 各书同时输出 `<书>_solar_eclipses.csv`、`<书>_lunar_eclipses.csv`）：
  - 来源：各书 `eclipse_sections`（只取日月食、不出凌犯）与凌犯各节中的「日有食之」「日蝕」「又日有蝕之」「月食」「月有食之，既」
    「日當食不食」等分句（「月食昴」有目标，属凌犯）；《晋书》卷十二「日蝕」、《宋书·五行志五》（卷三十四）、《南齐书》卷十二
    「日蝕」「月蝕」、《魏书·天象志一》、《隋书》卷二十一、《新唐书》卷三十二「日食」、《旧唐书》卷三十六「災異」（日食以朔日
    列举、不另书「日食」，节配置 `solar_dates`）、两《五代史》、《宋史》卷五十二「日食」「月食」、《金史》卷二十、《元史》卷四十八。
    《明史》天文志不载日月食，《清史稿》见上。
  - 日期：有干支取其日；日食无干支取朔（记「晦」者取月末日），月食无干支取望（朔后 14 日）；同句「既」记入 extent，「在某宿
    某度」记入 lodge，「不見」「陰雲」记入 note，「當食不食」note 记「当食不食」。
  - 观测地：逐行 `site_lon`、`site_lat`，由都城表 `capitals.csv`（与 `tests_records.rs` 共用；洛阳、长安坐标统一为汉魏洛阳城、
    汉长安城）按 regime 与年份给出；`historical_eclipses` 逐行取用。检验结果 `<书>_{solar,lunar}_eclipses_check.csv`，由
    `scripts/dev/phenomena_checks.sh` 批量重算（f64）。
  - 月食日差按「夜日」（日界取当地正午）计：史书一夜记为入夜之日，子夜后的食甚不算次日（此前按子夜分日，宋史月食近半显示
    「日差 +1」，实为口径）。
  - 解析中改正：主线年号表也走异体展开（《晋书》永甯、興甯、甯康此前漏识，十余条误归前一年号）；`EXTRA_VARIANTS` 加升→昇
    （昇明）；合并年号表的「改元前年号」列入年号池（如意、證聖、建義、普泰此前查不到）；年号前明书朝代（「魏明帝太和」「晉武帝
    泰始」）者只在该朝年代内取年号（《宋书·五行志》各门类自魏重起，「不早于前文」之法失效），同名年号接续时沿用当前元年；
    节标题单独成段；原文校改：《新唐书》「長慶二年四年」→「四月」，《晋书》「嘉平元年二月二月己示」→「二月己未」、
    「太寧三年十一月癸己」→「癸巳」。
- 年号末年：同一政权各年号的末年至少延到下一年号元年（年中改元者旧年号当年仍行用；方诗铭表 1222 年只印「元光⑧1」，
  《金史》「興定六年正月」此前越界回退为貞祐）。《金史》「九年二年癸亥」校作「九年二月癸亥」（`corrections`）。
  《辽史》无天文志，未检验。
- 解析规则（2026-10-28，对各书一致生效，回归比对只改正了误定年的记录）：分句以「自」起头者是追述（「自寶曆元年九月，及茲而五」），
  不作日期；段首分句以「年号+年数」起头而整句不成日期式（「天復元年五月自丁酉至于己亥」）时仍切换年号；时刻后缀
  （「辰時」「夜五更」「一鼓二籌後」）不妨碍日期识别（《晋书》升平五年一条由此改正为 361 年）；目标前的「于」「於」略去。
- 各书 `calendar` 列：实际所用历表，「实历朔日表」或「推步:<推步政权朔日表的 regime>」；`record_date_shift.py` 据此判断
  干支可落之月。`era` 列为原文所写年号（或帝号），不由起始年反查，以免撞上同年起始的别国年号。
- 年号的简繁对照用 OpenCC（`opencc-python-reimplemented`）逐字转换，再叠加一简多繁的手工表。此前只有约 80 字的手工表，
  漏了「賜」「龜」「監」「開」「業」等字，致《魏书》天賜、神龜（27 条记录误归前一年号，年份差 2—6 年）、《隋书》天監、開皇、
  大業未被识别；已改正。
- 史书惯用名的别名表见 `houhanshu_star_records.py`（共用工具库）的 `ALIASES` 与 `zhi_star_records.py` 的 `EXTRA_ALIASES`：
  - 太微、天市、紫宫为两垣合成，按两垣成员合集比对；
  - 婺女对须女，斗对南斗，积尸对舆鬼积尸气；
  - 带括注的星官以主名匹配，如「钺(附东井)」可用「钺」匹配。

## checks/

天象模块全部逐条校验结果，每项一个 CSV（`#` 开头为说明）。由 `scripts/dev/phenomena_checks.sh` 设 `PHENOMENA_CHECKS=1` 运行
`rust/core` 各校验测试写出，`scripts/phenomena/checks_to_md.py` 汇成 `doc/astronomy/phenomena-checks.md`。缺省用 f64 构建（`PHENOMENA_REAL=dd` 改用 double-double），每表最后一行说明记所用数值后端；`compare` 模式比对两种构建。平时 `cargo test`
不写这些文件。

另有 `checks/<book>_date_shift.csv`（hanshu、houhanshu、jinshu），由 `scripts/phenomena/record_date_shift.py`（Skyfield + DE441）写出：
行星凌犯记录在记载日 ±4 年内与目标地心角距 <1° 的全部时段及距记载日最近的一段；日干支不在所记之月者，同一干支可落在的邻近月份。
用来区分「日期记错（换个年、月就合）」与「换日期也不合」。
