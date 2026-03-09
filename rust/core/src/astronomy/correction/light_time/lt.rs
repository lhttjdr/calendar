use crate::astronomy::constant;
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::quantity::{duration::Duration, length::Length};

#[inline]
pub fn light_time(distance: Length) -> Duration {
    let sec = distance.meters() / constant::light_speed().m_per_s();
    Duration::in_seconds(sec)
}

pub fn retarded_time_point<F>(t: TimePoint, get_distance: F, max_iter: usize) -> TimePoint
where
    F: Fn(TimePoint) -> Length,
{
    let scale = t.scale;
    let jd_tt = t.to_scale(TimeScale::TT).jd;
    let mut jd_ret = jd_tt;
    for _ in 0..max_iter {
        let tr = TimePoint::new(TimeScale::TT, jd_ret);
        let d = get_distance(tr);
        jd_ret = jd_tt - light_time(d).in_days_value();
    }
    TimePoint::new(scale, TimePoint::new(TimeScale::TT, jd_ret).to_scale(scale).jd)
}
