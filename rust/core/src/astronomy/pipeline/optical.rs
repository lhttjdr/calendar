//! 光学修正器：同一架下光行差与折射（文档 3.1 OpticalCorrector）。泛型于 R: Real。

use super::state::State6;

/// 光学修正：在同一标架下施加光行差或折射等（文档 3.1）。标量 Real。
pub trait OpticalCorrector {
    fn apply(&self, state: State6) -> State6;
}

/// 占位：不做光学改正，原样返回状态。
#[allow(dead_code)]
pub struct NoOpticalEffect;

impl OpticalCorrector for NoOpticalEffect {
    fn apply(&self, state: State6) -> State6 {
        state
    }
}
