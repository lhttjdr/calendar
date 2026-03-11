//! 日月相关：黄经、节气（定气）、月相（合朔）。

mod longitude;
pub mod solar_term;
pub mod synodic;
pub use longitude::{
    moon_ecliptic_longitude, moon_ecliptic_longitude_mean_only, moon_ecliptic_longitude_with_max_terms,
    sun_ecliptic_longitude,
};
pub use solar_term::{
    mean_solar_longitude_velocity, solar_longitude_jd, solar_longitude_jd_de406, solar_term_jd,
    solar_term_jds_for_year, solar_term_jds_for_year_cached, solar_term_jds_for_year_de406,
    solar_term_jds_in_range, solar_term_jds_in_range_de406, solar_term_longitude, MEAN_TROPICAL_YEAR_DAYS,
};
pub use synodic::{
    approximate_new_moon_jd, expected_new_moon_longitude_difference, mean_synodic_velocity,
    new_moon_jd, new_moon_jd_de406, new_moon_jd_fine, new_moon_jd_fine_de406, new_moon_jd_with_options,
    new_moon_jds_in_range, new_moon_jds_in_range_de406, new_moon_jds_in_range_with_options,
    NewMoonOptions, MEAN_SYNODIC_MONTH, MEAN_SYNODIC_MONTH_W0, NEW_MOON_W0_EPOCH_JD,
};
