use super::vsop87::{Vsop87, Vsop87Parse};

pub const DEFAULT_VSOP87_EARTH_PATH: &str = "data/vsop87/VSOP87B.ear";
pub const DEFAULT_ELPMPP02_PATH: &str = "data/elpmpp02";

pub fn load_earth_vsop87(
    loader: &dyn crate::platform::DataLoader,
    path: &str,
) -> Result<Vsop87, crate::platform::LoadError> {
    Vsop87Parse::parse(loader, path)
}
