//! 民俗：依附在历法产出（日期、干支）上的纪时与择日规则。只算到规则给出的结果，不做吉凶解读。
//!
//! 分层见 `doc/architecture/three-domains.md`「民俗域」；各规则的原典出处与核对见 `doc/folk/`。
//! - **独立基础层**（只依赖日期与干支的固定循环）：[nayin]（六十甲子纳音）、[zhiri]（二十八宿值日、七曜）、
//!   [jiuxing]（年、月紫白九星）、[zajie]（三伏、社日、寒食、腊日、入梅出梅、土王用事、数九、七十二候；
//!   输入为节气所在民用日）、[gengdian]（更点，元授时法与清台官法；输入为纬度与太阳赤纬）、
//!   [jianchu]（建除十二值、黄道黑道十二神、百忌日）、[shensha]（月家日神煞，数据表 data/folk/shensha_month.csv）。

pub mod gengdian;
pub mod jianchu;
pub mod jiuxing;
pub mod nayin;
pub mod shensha;
pub mod zajie;
pub mod zhiri;
