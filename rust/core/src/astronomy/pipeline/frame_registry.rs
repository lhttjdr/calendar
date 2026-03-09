//! 架→架变换全局注册表，pipeline 可自注册表取链。

use crate::math::real::Real;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

/// 架标识（不含历元），用于注册与查询变换。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FrameId {
    MeanEcliptic,
    FK5,
    ICRS,
    MeanEquator,
    TrueEquator,
    ApparentEcliptic,
}

type TransformFn = Box<dyn Fn(Real, [Real; 3]) -> [Real; 3] + Send>;

static REGISTRY: Lazy<Mutex<HashMap<(FrameId, FrameId), TransformFn>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 注册 (from, to) 的变换：给定 JD 与位置 [x,y,z]（米），返回变换后位置（米）。
pub fn register_transform(from: FrameId, to: FrameId, f: TransformFn) {
    REGISTRY.lock().unwrap().insert((from, to), f);
}

/// 查询 (from, to) 的变换；返回 None 表示未注册。返回的闭包在每次调用时执行注册的变换。
pub fn get_transform(
    from: FrameId,
    to: FrameId,
) -> Option<Box<dyn Fn(Real, [Real; 3]) -> [Real; 3] + Send>> {
    let key = (from, to);
    if REGISTRY.lock().unwrap().contains_key(&key) {
        Some(Box::new(move |jd: Real, pos: [Real; 3]| {
            REGISTRY.lock().unwrap().get(&key).unwrap()(jd, pos)
        }))
    } else {
        None
    }
}
