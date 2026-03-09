//! 计算流水线与多种变换（文档 3、4）。
//!
//! - 6D 状态与 6×6 状态转移
//! - EphemerisProvider / FrameMapper / LightTimeCorrector / TransformGraph / OpticalCorrector
//! - Fluent 管线 API

mod chain;
mod ephemeris_provider;
mod frame_mapper;
mod frame_registry;
mod light_time;
mod optical;
mod state;
mod transition;
mod transform_graph;

pub use chain::Pipeline;
pub use ephemeris_provider::{Body, EphemerisProvider};
pub use frame_mapper::{FrameMapper, VsopToDe406IcrsFit};
pub use light_time::LightTimeCorrector;
pub use optical::OpticalCorrector;
pub use state::{CoordinateRepresentation, SphericalCoords, State6};
pub use transition::{StateTransition6, precession_transition};
pub use transform_graph::{TransformEdge, TransformEdgeViz, TransformGraph, TransformGraphVizData};
pub use frame_registry::{FrameId, get_transform, register_transform};
