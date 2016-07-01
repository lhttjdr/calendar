(function() {
  var Atomsphere = Calendar.createNS("Calendar.Atomsphere");
  //大气折射,h是真高度
  Atomsphere.correctAltitude = function(h) {
    return 0.0002967 / Math.tan(h + 0.003138 / (h + 0.08919));
  };
  //大气折射,ho是视高度
  Atomsphere.correctApparentAltitude = function(ho) {
    return -0.0002909 / Math.tan(ho + 0.002227 / (ho + 0.07679));
  }
})();
