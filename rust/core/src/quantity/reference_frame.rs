//! 参考架（架与历元正交）。对齐《岁差模型与参考标架解析》。历元用 R 表示，不写死 f64。
//!
//! **架与度规**：当前所有架均为直角坐标（正交归一化基与坐标基一致），度规为单位阵；
//! 位置/速度在该架下用 [Vector3](crate::quantity::vector3::Vector3) 表示。若某架采用球坐标等，
//! 可携带 [度规/scale factors](crate::quantity::frame_metric)，与 [坐标分量](crate::quantity::coord_components) 配合使用。

use super::epoch::Epoch;

/// 参考架：定义空间 X-Y-Z 轴的物理朝向（及度规；当前均为直角，度规为单位阵）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReferenceFrame {
    ICRS,
    FK5,
    MeanEcliptic(Epoch),
    MeanEquator(Epoch),
    TrueEquator(Epoch),
    TrueEcliptic(Epoch),
    ApparentEcliptic(Epoch),
    ApparentEquator(Epoch),
    /// 站心真赤道：坐标轴同 [`Self::TrueEquator`]，原点平移到地面观测者。观测者不入架标识，由路径执行时给定。
    TopocentricEquator(Epoch),
    /// [`Self::TopocentricEquator`] 加周日光行差后的状态，坐标轴与原点不变。
    TopocentricAberratedEquator(Epoch),
    /// 站心地平直角坐标（ENU：x 东、y 北、z 天顶），未含折射：几何地平。
    HorizontalGeometric(Epoch),
    /// [`Self::HorizontalGeometric`] 加大气折射后的状态，坐标轴与原点不变：视地平。
    HorizontalApparent(Epoch),
    Elpmpp02MeanLunar,
    Elpmpp02LaskarCartesian,
}

impl ReferenceFrame {
    pub fn is_epoch_dependent(self) -> bool {
        matches!(
            self,
            ReferenceFrame::MeanEcliptic(_)
                | ReferenceFrame::MeanEquator(_)
                | ReferenceFrame::TrueEquator(_)
                | ReferenceFrame::TrueEcliptic(_)
                | ReferenceFrame::ApparentEcliptic(_)
                | ReferenceFrame::ApparentEquator(_)
                | ReferenceFrame::TopocentricEquator(_)
                | ReferenceFrame::TopocentricAberratedEquator(_)
                | ReferenceFrame::HorizontalGeometric(_)
                | ReferenceFrame::HorizontalApparent(_)
        )
    }

    pub fn id_str(self) -> &'static str {
        match self {
            ReferenceFrame::ICRS => "ICRS",
            ReferenceFrame::FK5 => "FK5",
            ReferenceFrame::MeanEcliptic(_) => "MeanEcliptic(epoch)",
            ReferenceFrame::MeanEquator(_) => "MeanEquator(epoch)",
            ReferenceFrame::TrueEquator(_) => "TrueEquator(epoch)",
            ReferenceFrame::TrueEcliptic(_) => "TrueEcliptic(epoch)",
            ReferenceFrame::ApparentEcliptic(_) => "ApparentEcliptic(epoch)",
            ReferenceFrame::ApparentEquator(_) => "ApparentEquator(epoch)",
            ReferenceFrame::TopocentricEquator(_) => "TopocentricEquator(epoch)",
            ReferenceFrame::TopocentricAberratedEquator(_) => "TopocentricAberratedEquator(epoch)",
            ReferenceFrame::HorizontalGeometric(_) => "HorizontalGeometric(epoch)",
            ReferenceFrame::HorizontalApparent(_) => "HorizontalApparent(epoch)",
            ReferenceFrame::Elpmpp02MeanLunar => "ELPMPP02_MEAN_LUNAR",
            ReferenceFrame::Elpmpp02LaskarCartesian => "ELPMPP02_LASKAR_CARTESIAN",
        }
    }
}
