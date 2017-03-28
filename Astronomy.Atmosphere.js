/*
 * 蒙气修正
 * h：真高度（高度角）； ha：视高度（地平纬度）
 */
(function() {
  var Atomsphere = Calendar.createNS("Calendar.Atomsphere");
  var Maths = Calendar.importNS("Calendar.Maths");

  Atomsphere.inverseRefraction = function(h, P, T) {
    // Sæmundsson formula
    // It is consistent with Bennett’s to within 0.1′
    h = h / Maths.DegreePerRadian; // degree
    P=P || 101.0; // pressure, default 101.0 kPa
    T=T || 10; // temperature, default 10 °C
    var f=(P/101)*(283/(273+T));
    var R= f* 1.02 / Math.tan( h + 10.3/ (h + 5.11)); // arc of minutes
    return R/Maths.MinutePerRadian; // to Radian
    // return 0.0002967 / Math.tan(h + 0.003138 / (h + 0.08919));
  };
  Atomsphere.Refraction = function(ha, formula, P, T) {
    formula=formula || "Bennett";
    P=P || 101.0; // pressure, default 101.0 kPa
    T=T || 10; // temperature, default 10 °C
    var R=0.0;
    switch (formula) {
      case "Bennett":
        // low-presion . maximum error is 0.07'=4.2", when ha=12°
        ha= ha*Maths.DegreePerRadian;
        R=1.0/Math.tan((ha+7.31/(ha+4.4))/Maths.DegreePerRadian); // arcminutes
        var dR=-0.06*Math.sin((14.7*R + 13)/Maths.MinutePerRadian); // with dR, biggest error is 0.015'=0.9"
        R += dR;
        break;
      case "Smart": // 1980, highly accurate
        var ha_d= ha*Maths.DegreePerRadian;
        if(15<=ha_d && ha_d<90){
          R=Maths.parseAngle('58.294"')/Math.tan(ha)-Maths.parseAngle('0.0668"')/Math.pow(Math.tan(ha),3.0); // 58.294"/tan ha - 0.0668"/tan^3 ha
        }else if(0<= ha_d && ha_d <15){ // Explanatory Supplement to the Astronomical Almanac
          R=(34.133 + 4.197*ha_d + 0.00428*ha_d*ha_d)/(1+0.505*ha_d + 0.084*ha_d*ha_d);
        }
        break;
      case "Meeus": // 1999
        var ha_d= ha*Maths.DegreePerRadian;
        if(15<=ha_d && ha_d<90){
          R=Maths.parseAngle('58.276"')/Math.tan(ha)-Maths.parseAngle('0.0824"')/Math.pow(Math.tan(ha),3.0); // 58.276"/tan ha - 0.0824"/tan^3 ha
        }else if(0<= ha_d && ha_d<15){ // Explanatory Supplement to the Astronomical Almanac
          R=(34.133 + 4.197*ha_d + 0.00428*ha_d*ha_d)/(1+0.505*ha_d + 0.084*ha_d*ha_d);
        }
        break;
      default:
        R=0.0;
    }
    var f=(P/101)*(283/(273+T));
    return -f*R/Maths.MinutePerRadian;
  }
})();
