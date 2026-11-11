//! 历法：公历、农历及政权纪年，历法域内部分四层。
//!
//! **四层结构**（见 `doc/calendar/calendar-layers.md`、跨域关系见 `doc/architecture/three-domains.md`）：
//! - **第 1 层·独立基础层**（不查天文）：[gregorian]（儒略历/格里高利历）、[ancient_restore]（古历复原：分数推步、
//!   平朔平气，仅作验证）、[push_calendar]（历谱生成；定朔的金重修大明历在 [daming_li]）、[gan_zhi]（干支纯循环命名）、
//!   [pingqi]（按行用历推平气；只在换时标时用 ΔT）。
//! - **第 2 层·聚合/实算层**：某一部历的具体历日，按来源分四种——[artifacts]（实物历日，如《宝祐四年会天历》）、
//!   [push_calendar] 按术推步（第 1 层的历谱生成在此用作某历的历日）、[historical]（[historical::ShuoriTable]，今人历表：
//!   张培瑜《三千五百年历日天象》经 ChineseCalendar 转录）、[chinese_lunar] 现算定气定朔；[dunhuang]
//!   （[dunhuang::DunhuangCalendar]）为敦煌政权（吐蕃、归义军时期）提供出土历日有据之月。
//! - **第 3 层·派生层**（依赖第 2 层产出）：[era_names]——依附于历法的纪年，按政权分，同一年可能有多个并存政权
//!   （如三国期间魏/蜀/吴），[EraNames::get_all] 返回全部，[EraNames::get_primary] 返回"正统主线"一条；数据经
//!   [EraNames] 解析 `data/calendar/汇总/era_names_merged.csv`——跨来源交叉校对出的汇总表（主线来自
//!   `vendor/calendar/chinese_calendar/年号表.csv`，并存政权补充自 `data/calendar/fang_shiming/`
//!   转录）；表结构与汇总方法见 `data/calendar/汇总/README.md`。
//!   [month_labels]——历史月序改制（新莽、魏景初建丑，武周、唐肃宗末建子）：建寅月序 ↔ 当时政权实际月名，
//!   只做命名，朔日仍取第 2 层；数据 `data/calendar/month_label_reforms.csv`。
//! - **第 4 层·仲裁/路由层**：[lifa]（[LifaRegistry] 解析 `data/calendar/汇总/lifa_summary.csv`，按时代/政权
//!   路由该用哪部历法；中原一线历日按来源次序取舍 [lifa::resolve_year]：实物历日 → 按术推步 → 今人历表 → 现算，
//!   结果标明来源 [lifa::RiliSource]；敦煌有据之月与中原历的逐月取舍 [lifa::resolve_dunhuang_day]）；[convert]（调用前者换算）。
//!
//! **制造日历的逻辑由本包负责**：定气与定朔由 astronomy 提供，本包据此计算岁数据（14 朔 + 12 中气）、
//! 公历↔农历换算及月名/闰月规则。显示（月历排版、TUI）由上层负责。

pub mod ancient_restore;
pub mod artifacts;
pub mod chinese_lunar;
pub mod convert;
pub mod daming_li;
pub mod dingshuo;
pub mod song_early;
pub mod shixian;
pub mod shixian_kaocheng;
pub mod shixian_xinfa;
pub mod dunhuang;
pub mod era_gaiyuan;
pub mod era_names;
pub mod gan_zhi;
pub mod gan_zhi_options;
pub mod gregorian;
pub mod historical;
pub mod lifa;
pub mod month_labels;
pub mod options;
pub mod pingqi;
pub mod push_calendar;
pub mod system;
pub mod timekeeping;
