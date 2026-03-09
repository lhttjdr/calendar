//! 光行差：观测者（地心）运动导致的视方向改正。

mod direction;
pub use direction::{annual_aberration_direction, annual_aberration_direction_derivative};

#[cfg(test)]
mod tests;
