(function() {
  var Constant = Calendar.createNS("Calendar.Astronomy.Constant");
  var Maths=Calendar.importNS("Calendar.Maths");
  Constant.Earth={};
  // http://asa.usno.navy.mil/SecK/Constants.html
  Constant.Earth.EquatorialRadius= 6378136.6; // m, a_E, a_e, IAU2009/2012, ±0·1
  Constant.Earth.ReciprocalFlattening= 298.25642; // 1/f, IERS 2010, ±1e−5

  var a=Constant.Earth.EquatorialRadius;
  var f=1.0/Constant.Earth.ReciprocalFlattening;
  Constant.Earth.PolarRadius=a*(1-f); //m
  var b=Constant.Earth.PolarRadius;

  // International Union of Geodesy and Geophysics (IUGG)
  Constant.Earth.MeanRadius= (2*a+b)/3;

  // astronomical unit
  // Since 2010, the astronomical unit is not yet estimated by the planetary ephemerides.
  Constant.AU=149597870700;  // IAU2009, error=±3. After 2009, it is fixed, so no-error exists.
  Constant.SpeedOfLight=299792458; // m/s, c, IAU2009/2012
  Constant.LightTimeForUnitDistance=499.00478384; // s, τA = au/c, IAU2012

  Constant.J2000 = 2451545;

  Constant.Moon={};
  Constant.Moon.MeanEquatorialRadius=1737.4; // km, ±1
  Constant.Sun={};
  Constant.Sun.EquatorialRadius=696000; // km
  Constant.solarParallax=Maths.parseAngle('8".794143'); // asin(a_E/A), IAU2009
})();
