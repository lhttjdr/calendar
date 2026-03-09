//! 历法换算选项。精度在**本层**选择，天文层不指定 f64；default 时由**调用处**传入基于 f64 的 Real。

/// 默认精度：基于 f64。顶层（calendar 或应用）在「默认」时传入此 Real 实现；
/// 天文层不写死 f64，由调用处通过泛型或本类型别名选择。
/// 与 `crate::math::real::Real` 一致，保留用于兼容。
pub type DefaultReal = f64;

/// 历法换算选项（预留）。标量统一使用 `Real`（即 f64）。
#[derive(Clone, Default)]
pub struct CalendarOptions {}
