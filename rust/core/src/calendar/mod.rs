//! 历法：公历、农历等。
//!
//! **制造日历的逻辑由本包负责**：定气与定朔由 astronomy 提供，本包据此计算岁数据（14 朔 + 12 中气）、
//! 公历↔农历换算及月名/闰月规则。
//! 显示（月历排版、TUI）由上层负责。

pub mod chinese_lunar;
pub mod convert;
pub mod gan_zhi;
pub mod gan_zhi_options;
pub mod gregorian;
pub mod options;
pub mod system;
