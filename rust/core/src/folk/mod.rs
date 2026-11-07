//! 民俗：依附在历法产出（日期、干支）上的纪时与择日规则。只算到规则给出的结果，不做吉凶解读。
//!
//! 分层见 `doc/architecture/three-domains.md`「民俗域」；各规则的原典出处与核对见 `doc/folk/`。
//! - **独立基础层**（只依赖日期与干支的固定循环）：[nayin]（六十甲子纳音）、[zhiri]（二十八宿值日、七曜）、
//!   [jiuxing]（年、月紫白九星）。

pub mod jiuxing;
pub mod nayin;
pub mod zhiri;
