//! 历法：公历、农历及政权纪年，历法域内部分四层。
//!
//! **四层结构**（见 `doc/calendar/calendar-layers.md`、跨域关系见 `doc/architecture/three-domains.md`）：
//! - **第 1 层·独立基础层**（不查天文）：[gregorian]（儒略历/格里高利历）、[ancient_restore]（古历复原：分数推步、
//!   平朔平气，仅作验证）、[push_calendar]（历谱生成；定朔的金重修大明历在 [daming_li]）、[gan_zhi]（干支纯循环命名）。
//! - **第 2 层·聚合/实算层**（依赖天文的真朔真气，或查历史表）：[chinese_lunar] 现算；[historical]
//!   （[HistoricalCalendar]）为主表查表提供朔日岁数据（主表 = 各朝颁历据历还原推步得出的连续表）。
//! - **第 3 层·派生层**（依赖第 2 层产出）：[era_names]——依附于历法的纪年，按政权分，同一年可能有多个并存政权
//!   （如三国期间魏/蜀/吴），[EraNames::get_all] 返回全部，[EraNames::get_primary] 返回"正统主线"一条；数据经
//!   [EraNames] 解析 `data/calendar/汇总/era_names_merged.csv`——跨来源交叉校对出的汇总表（主线来自
//!   `vendor/calendar/chinese_calendar/年号表.csv`，并存政权补充自 `data/calendar/fang_shiming/`
//!   转录），主表对应 id [lifa::LIFA_ID_ZHONGYUAN_SHILI]（《三千五百年历日天象》无名为「中原正朔」的历法，
//!   主表为据历还原推步结果）；表结构与汇总方法见 `data/calendar/汇总/README.md`。
//!   [month_labels]——历史月序改制（新莽、魏景初建丑，武周、唐肃宗末建子）：主表建寅月 ↔ 当时政权实际月名，
//!   只做命名，朔日仍取第 2 层主表；数据 `data/calendar/month_label_reforms.csv`。
//! - **第 4 层·仲裁/路由层**：[lifa]（[LifaRegistry] 解析 `data/calendar/汇总/lifa_summary.csv`，按时代/政权
//!   路由该用哪部历法）；[convert]（"实历优先、缺省退回定气定朔"判断）。
//!
//! **制造日历的逻辑由本包负责**：定气与定朔由 astronomy 提供，本包据此计算岁数据（14 朔 + 12 中气）、
//! 公历↔农历换算及月名/闰月规则。显示（月历排版、TUI）由上层负责。

pub mod ancient_restore;
pub mod chinese_lunar;
pub mod convert;
pub mod daming_li;
pub mod era_gaiyuan;
pub mod era_names;
pub mod gan_zhi;
pub mod gan_zhi_options;
pub mod gregorian;
pub mod historical;
pub mod lifa;
pub mod month_labels;
pub mod options;
pub mod push_calendar;
pub mod system;
pub mod timekeeping;
