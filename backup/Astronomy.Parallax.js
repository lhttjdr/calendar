(function() {
  var Parallax = Calendar.createNS("Calendar.Astronomy.Parallax");
  var Maths = Calendar.importNS("Calendar.Maths");
  var Constant=Calendar.importNS("Calendar.Astronomy.Constant");
  var Equator=Calendar.importNS("Calendar.Astronomy.Coordinate.Equator");
  // for short
  var cos=Maths.cos, acos=Maths.acos;
  var sin=Maths.sin, asin=Maths.asin;
  var tan=Maths.tan, atan=Maths.atan;
  var sqr=Maths.sqr, sqrt=Maths.sqrt;
  // 赤道地平视差
  // sin p / a = sin(180-z')/ r =sin(z') /r
  // parallax p is greast when z'=90, thus
  // sin P = a / r, horizontal parallax
  // -----------------------------------
  // Usage: sin p = sin P. sin z'
  // for small angle: p = P. sin z'
  Parallax.solarParallax=function(){ // 太阳的赤道地平视差
    // return Math.asin(Constant.Earth.MeanRadius/Constant.AU);
    return Constant.solarParallax;
  }

  // (celestial) body, 赤道坐标系的一天体
  // hourAngle, 该天体的地心时角
  // latitude，观测者所在地的纬度
  // altitude，观测者所在地的海拔高度
  // 本质就是地球坐标系问题，地球被近似为椭球
  // 参心大地坐标系转换到参心直角坐标系
  // x=N.cosB.cosL, y=N.cosB.sinL, z=N(1-e^2)sinB
  // N=a/W, W=sqrt(1-e^2sin^2B)
  // 不仅是平移坐标系中心？春分点不变？
  Parallax.correction=function(body, hourAngle, latitude, altitude){ //视差修正
    var LST=body.rightAscension + hourAngle; // RA + HA，当地恒星时, 经度
    var a=Constant.Earth.EquatorialRadius;
    var b=Constant.Earth.PolarRadius;
    var theta=atan(b/a*tan(latitude));
    var xy=a*cos(theta)+altitude*cos(latitude);
    var z=b*sin(theta)+altitude*sin(latitude);
    var x=xy*cos(LST);
    var y=xy*sin(LST);

    var v=new Maths.Vector3(x,y,z);
    return Equator.fromStandard(body.toStandard().translate(v).transform());
  }
})();
