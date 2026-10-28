//! 视位置：光行时、岁差、章动与太阳/月球视黄经；视赤道位置（含距离）、站心视位置与地平坐标；行星地心视位置与视星等；恒星视位置。

mod equatorial;
mod longitude;
mod magnitude;
mod planets;
mod star;
mod topocentric;
pub use equatorial::*;
pub use longitude::*;
pub use magnitude::*;
pub use planets::*;
pub use star::*;
pub use topocentric::*;

#[cfg(test)]
mod tests_lunar_ndot;
#[cfg(test)]
mod tests_planets;
#[cfg(test)]
mod tests_topocentric;
