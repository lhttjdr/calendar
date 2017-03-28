/*
 * 蒙气修正
 * h：真高度（高度角）； ho：视高度（地平纬度）
 */
(function() {
  var Atomsphere = Calendar.createNS("Calendar.Atomsphere");
  Atomsphere.correctAltitude = function(h) {
    return 0.0002967 / Math.tan(h + 0.003138 / (h + 0.08919));
  };
  Atomsphere.correctApparentAltitude = function(ho) {
    return -0.0002909 / Math.tan(ho + 0.002227 / (ho + 0.07679));
  }
})();
