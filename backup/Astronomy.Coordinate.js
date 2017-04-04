(function() {
  var Coordinate = Calendar.createNS("Calendar.Astronomy.Coordinate");
  var Maths = Calendar.importNS("Calendar.Maths");
  var Precession = Calendar.importNS("Calendar.Astronomy.AxialPrecession");
  var Nutation = Calendar.importNS("Calendar.Astronomy.Nutation");
  ////////////////////////////////////////////
  // Point in Equator Coordinate
  // -- RA, Right Ascension, alpha, 赤经
  // -- Dec, Declination, delta, 赤纬
  Coordinate.Equator = function(rightAscension, declination, distance) {
    this.rightAscension = rightAscension;
    this.declination = declination;
    this.distance = distance;
  };
  // Private:
  // TODO: Now they are public, because they should be access by public methods.
  Coordinate.Equator.prototype.toStandard = function() {
    return new Maths.Coordinate.Sphere(this.distance, Maths.HalfPi - this.declination,
      this.rightAscension);
  };
  // Static:
  Coordinate.Equator.fromStandard = function(point) {
    return new Coordinate.Equator(Maths.normalizeToRangeZeroDoublePi(point.phi),
      Maths.normalizeToRangePlusMinusPi(Maths.HalfPi - point.theta),
      point.r);
  };
  // Public:
  Coordinate.Equator.prototype.toEcliptic = function(obliquity) {
    return Coordinate.Ecliptic.fromStandard(this.toStandard().rotateX(
      obliquity).transform());
  };
  // GST, Greenwich Sidereal Time，格林威治恒星时
  // LST, Local Sidereal Time， 本地恒星时
  // LHA, Local Hour Angle， 本地时角
  // LHA = LST - RA = GST - L -RA
  // Longitude, 本地经度
  Coordinate.Equator.prototype.toHourAngle = function(longitude, GST) {
    return new Coordinate.HourAngle(GST - longitude - this.rightAscension,
      this.declination, this.distance);
  };
  // Latitude, 本地纬度
  Coordinate.Equator.prototype.toHorizon = function(longitude, latitude, GST) {
    return this.toHourAngle(longitude, GST).toHorizon(latitude);
  };
  Coordinate.Equator.prototype.toMeanDateEqutator = function(t, model) {
    var precession = Precession[model];
    return Coordinate.Equator.fromStandard(this.toStandard().rotateZ(Maths.HalfPi -
      precession.zeta(t)).rotateX(precession.theta(t)).rotateZ(-(Maths.HalfPi +
      precession.z(t))).transform());
  };
  Coordinate.Equator.prototype.toDateEqutator = function(t, model) {
    var nutation = Nutation.value(t);
    var epsilon = Precession[model].epsilon(t);
    return Coordinate.Equator.fromStandard(this.toStandard().rotateX(
        epsilon).rotateZ(-nutation.psi).rotateX(-(epsilon + nutation.epsilon))
      .transform());
  };
  Coordinate.Equator.prototype.toMeanEpochEqutator = function(t, model) {
    var precession = Precession[model];
    return Coordinate.Equator.fromStandard(this.toStandard().rotateZ(Maths.HalfPi +
      precession.z(t)).rotateX(-precession.theta(t)).rotateZ(-(Maths.HalfPi -
      precession.zeta(t))).transform());
  };
  // End Equator
  ////////////////////////////////////////////
  // Point in HourAngle Coordinate
  // -- a kind of Equatorial Coordinate
  // -- HA, Hour Angle, H, 时角
  // -- Dec, Declination, delta, 赤纬
  Coordinate.HourAngle = function(hourAngle, declination, distance) {
    this.hourAngle = hourAngle;
    this.declination = declination;
    this.distance = distance;
  };
  // Private:
  // TODO: Now they are public, because they should be access by public methods.
  Coordinate.HourAngle.prototype.toStandard = function() {
    return new Maths.Coordinate.Sphere(this.distance, Maths.HalfPi - this.declination,
      Maths.HalfPi - this.hourAngle);
  };
  // Static:
  // a overload of construtor
  // TODO: If javascript allows overload, it should be a kind of construtor
  Coordinate.HourAngle.fromStandard = function(point) {
    return new Coordinate.HourAngle(Maths.HalfPi - point.phi, Maths.HalfPi -
      point.theta, point.r);
  };
  // Public:
  // GST, Greenwich Sidereal Time，格林威治恒星时
  // LST, Local Sidereal Time， 本地恒星时
  // LHA, Local Hour Angle， 本地时角
  // L, Longitude, 本地经度
  // LHA = LST - RA = GST - L -RA
  Coordinate.HourAngle.prototype.toEquator = function(longitude, GST) {
    return new Coordinate.Equator(GST - longitude - this.hourAngle, this.altitude,
      this.distance);
  };
  Coordinate.HourAngle.prototype.toHorizon = function(latitude) {
    return Coordinate.Horizon.fromStandard(this.toStandard().rotateX(
      Maths.HalfPi - latitude).transform());
  };
  // End Equator
  ////////////////////////////////////////////
  // Point in Ecliptic Coordinate
  // -- ecliptic longitude, 黄经
  // -- ecliptic latitude, 黄纬
  Coordinate.Ecliptic = function(longitude, latitude, distance) {
    this.longitude = longitude || 0;
    this.latitude = latitude || 0;
    this.distance = distance || 0;
  };
  // Private:
  // TODO: Now they are public, because they should be access by public methods.
  Coordinate.Ecliptic.prototype.toStandard = function() {
    return new Maths.Coordinate.Sphere(this.distance, Maths.HalfPi - this.declination,
      this.longitude);
  };
  // Static
  // a overload of construtor
  Coordinate.Ecliptic.fromStandard = function(point) {
    return new Coordinate.Ecliptic(point.phi, Maths.HalfPi - point.theta,
      point.r);
  };
  // Public:
  Coordinate.Ecliptic.prototype.toEquator = function(obliquity) {
    return Coordinate.Equator.fromStandard(this.toStandard().rotateX(-
      obliquity).transform());
  };
  Coordinate.Ecliptic.prototype.toDateEcliptic = function(t, model) {
    var precession = Precession[model];
    return Coordinate.Equator.fromStandard(this.toStandard().rotateZ(-
      precession.psi(t)).rotateX(-precession.omega(t)).rotateZ(
      precession.chi(t)).rotateX(precession.epsilon(t)).transform());
  };
  Coordinate.Ecliptic.prototype.toEpochEcliptic = function(t, model) {
    var precession = Precession[model];
    return Coordinate.Equator.fromStandard(this.toStandard().rotateZ(-
      precession.epsilon(t)).rotateX(-precession.chi(t)).rotateZ(
      precession.omega(t)).rotateX(precession.psi(t)).transform());
  };
  ////////////////////////////////////////////
  // Point in Horizontal Coordinate
  // -- azimuth, Az, 方位角, 地平经度
  // -- altitude, Alt, elevation, 高度角, 仰角, 地平纬度
  Coordinate.Horizon = function(azimuth, altitude, distance) {
    this.azimuth = azimuth || 0;
    this.altitude = altitude || 0;
    this.distance = distance || 0;
  };
  // Private:
  // TODO: Now they are public, because they should be access by public methods.
  Coordinate.Horizon.prototype.toStandard = function() {
    return new Maths.Coordinate.Sphere(this.distance, Maths.HalfPi - this.altitude,
      Maths.HalfPi - this.azimuth);
  };
  // Static:
  // a overload of construtor
  Coordinate.Horizon.fromStandard = function(point) {
    return new Coordinate.Horizon(Maths.HalfPi - point.phi, Maths.HalfPi -
      point.theta, point.r);
  };
  // Public:
  // Longitude, 观测者位置的经度
  // Latitude, 观测者位置的纬度
  // GST, Greenwich Sidereal Time， 格林尼治恒星时
  Coordinate.Horizon.prototype.toHourAngle = function(latitude) {
    return new Coordinate.HourAngle().fromStandard(this.toStandard().rotateX(
      Maths.HalfPi - latitude).transform());
  };
  Coordinate.Horizon.prototype.toEquator = function(longitude, latitude, GST) {
    return this.toHourAngle(latitude).toEquator(longitude, GST);
  };
})();
