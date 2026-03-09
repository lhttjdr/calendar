//! 光行时修正：观测时刻 t，推迟时 tr = t − D/c（D 为地心距）。

mod lt;
pub use lt::{light_time, retarded_time_point};

#[cfg(test)]
mod tests;
