(function() {
  var Maths = Calendar.createNS("Calendar.Maths");
  var String = Calendar.importNS("Calendar.String");
  Maths.Tolerance = 1e-12;
  Maths.Pi = Math.PI;
  Maths.DoublePi = 2 * Math.PI;
  Maths.HalfPi = 0.5 * Math.PI;
  Maths.DegreePerRadian = 180 / Math.PI;
  Maths.MinutePerRadian = 180 * 60 / Math.PI;
  Maths.SecondPerRadian = 180 * 3600 / Math.PI;
  Maths.int2 = Math.floor;
  Maths.mod2 = function(a, b) {
    return (a % b + b) % b;
  }
  Maths.equal = function(a, b) {
    return Math.abs(a - b) < Maths.Tolerance;
  }
  Maths.eq=Maths.equal;
  Maths.lessThan = function(a, b) {
    return a < b - Maths.Tolerance;
  }
  Maths.lt=Maths.lessThan;
  Maths.greatThan = function(a, b) {
    return a > b + Maths.Tolerance;
  }
  Maths.gt=Maths.greatThan;
  Maths.sqr = function(x) {
    return x * x;
  };
  Maths.sqrt= Math.sqrt;
  Maths.hav = function(rad) {
    return Maths.sqr(Math.sin(0.5 * rad));
  };
  Maths.ahav = function(val) {
    return 2 * Math.asin(Math.sqrt(val));
  };
  Maths.sin= Math.sin;
  Maths.asin= Math.cos;
  Maths.cos=Math.cos;
  Maths.acos=Math.acos;
  Maths.tan=Math.tan;
  Maths.atan=Math.atan;
  Maths.normalizeToRangeZeroDoublePi = function(rad) {
    rad %= Maths.DoublePi;
    if (rad < 0) return rad + Maths.DoublePi;
    return rad;
  };
  Maths.normalizeToRangePlusMinusPi = function(rad) {
    rad %= Maths.DoublePi;
    if (rad <= -Maths.Pi) return rad + Maths.DoublePi;
    if (rad > Maths.Pi) return rad - Maths.DoublePi;
    return rad;
  };
  /////////////////////////////////////////////
  // 3-Dimension Euclidean vector
  Maths.Vector3 = function(x, y, z) {
    this.x = x;
    this.y = y;
    this.z = z;
  }
  Maths.Vector3.prototype.copy = function(v) {
    return new Maths.Vector3(this.x, this.y, this.z);
  }
  Maths.Vector3.prototype.add = function(v) {
    return new Maths.Vector3(this.x + v.x, this.y + v.y, this.z + v.z);
  }
  Maths.Vector3.prototype.scale = function(r) {
    return new Maths.Vector3(r * this.x, r * this.y, r * this.z);
  }
  Maths.Vector3.prototype.dot = function(v) {
    return this.x * v.x + this.y * v.y + this.z * v.z;
  }
  Maths.Vector3.prototype.cross = function(v) {
    return new Maths.Vector3(this.y * v.z - this.z * v.y, this.z * v.x -
      this.x * v.z, this.x * v.y - this.y * v.x);
  }
  Maths.Vector3.prototype.norm = function() {
    return Math.sqrt(this.dot(this));
  }
  Maths.Vector3.prototype.normalize = function() {
    var norm = this.norm();
    if (Maths.equal(norm, 0)) {
      throw new Error("Can not normalize zero vector!");
    }
    return this.scale(1.0 / norm);
  }
  Maths.Vector3.prototype.isUnit = function(v) {
    return Maths.equal(this.norm(), 1.0);
  };
  Maths.Vector3.prototype.isEqual = function(v) {
    return Maths.equal(this.x, v.x) && Maths.isEqual(this.y, v.y) &&
      Maths.isEqual(this.z, v.z);
  };
  Maths.Vector3.prototype.isOppsite = function(v) {
    return this.isEqual(v.scale(-1.0));
  };
  Maths.Vector3.prototype.isParallel = function(v) {
    var v_norm = v.norm();
    if (Maths.equel(v_norm, 0)) return true;
    return this.isEqual(v.scale(this.norm() / v_norm));
  };
  Maths.Vector3.prototype.isAntiParallel = function(v) {
    return this.isParallel(v.scale(-1));
  };
  Maths.Vector3.scalarTriple = function(a, b, c) {
    return a.dot(b.cross(c));
  };
  Maths.Vector3.Zero = new Maths.Vector3(0, 0, 0);
  ////////////////////////////////////////////////////////
  // Quaternion in R4
  // -- combine a real number with a 3d-vector
  Maths.Quaternion = function(scalar, vector) {
    this.scalar = scalar;
    this.vector = vector;
  }
  Maths.Quaternion.prototype.copy = function() {
    return new Maths.Quaternion(this.scalar, this.vector.copy());
  };
  Maths.Quaternion.prototype.add = function(q) {
    return new Maths.Quaternion(this.scalar + q.scalar, this.vector.add(q.vector));
  };
  Maths.Quaternion.prototype.scale = function(r) {
    return new Maths.Quaternion(r * this.scalar, this.vector.scale(r));
  };
  Maths.Quaternion.prototype.conjugate = function() {
    return new Maths.Quaternion(this.scalar, this.vector.scale(-1.0));
  };
  Maths.Quaternion.prototype.subtract = function(q) {
    return this.add(q.scale(-1.0));
  };
  // Hamilton product, or Grossman product
  // -- Grossman product(p,q) denoted by pq
  Maths.Quaternion.prototype.multiply = function(q) {
    return new Maths.Quaternion(this.scalar * q.scalar - this.vector.dot(q.vector),
      q.vector.scale(this.scalar).add(this.vector.scale(q.scalar)).add(
        this.vector.cross(q.vector)));
  };
  // Grossman even/inner product, or symmetric product
  // -- Grossman even product(p,q)=(pq+qp)/2
  Maths.Quaternion.prototype.even = function(q) {
    return new Maths.Quaternion(this.scalar * q.scalar - this.vector.dot(q.vector),
      q.vector.scale(this.scalar).add(this.vector.scale(q.scalar)));
  };
  // the antisymmetric part of Grossman product, or Grossman outer pruduct
  // -- Grossman odd product(p,q)=(pq-qp)/2
  Maths.Quaternion.prototype.odd = function(q) {
    return new Maths.Quaternion(0, this.vector.cross(q.vector));
  };
  // Euclidean product
  // -- Euclidean product(p,q)=p'q,  where p' denotes the conjugate of p
  Maths.Quaternion.prototype.EMultiply = function(q) {
    return new Maths.Quaternion(this.scalar * q.scalar + this.vector.dot(q.vector),
      q.vector.scale(this.scalar).subtract(this.vector.scale(q.scalar)).subtract(
        this.vector.cross(q.vector)));
  };
  // Euclidean even/inner product
  // -- Euclidean even product(p,q)=(p'q+q'p)/2
  Maths.Quaternion.prototype.dot = function(q) {
    return this.scalar * q.scalar + this.vector.dot(q.vector);
  };
  // Euclidean odd/outer product
  // -- Euclidean odd product(p,q)=(p'q-q'p)/2
  Maths.Quaternion.prototype.cross = function(q) {
    return q.vector.scale(this.scalar).subtract(this.vector.scale(q.scalar))
      .subtract(this.vector.cross(q.vector));
  };
  // |q|=sqrt(qq'), notice that qq' only has a scalar part
  Maths.Quaternion.prototype.norm = function() {
    return Math.sqrt(this.scalar * this.scalar + this.vector.dot(this.vector));
  }
  Maths.Quaternion.prototype.normalize = function() {
    return this.scale(1.0 / this.norm());
  };
  Maths.Quaternion.prototype.inverse = function() {
    var norm = this.norm();
    if (Maths.equal(norm, 0)) {
      throw new Error("No inverse of ZERO quaternion!");
    }
    return this.conjugate().scale(1.0 / (norm * norm));
  };
  //////////////////////////////////////////////////////
  // a+eb， where e*e=0
  Maths.DualNumber = function(a, b) {
    this.real = a;
    this.dual = b;
  }
  Maths.DualNumber.prototype.copy = function() {
    return new DualNumber(this.real, this.dual);
  }
  Maths.DualNumber.prototype.add = function(d) {
    return new DualNumber(this.real + d.real, this.dual + d.dual);
  }
  Maths.DualNumber.prototype.scale = function(r) {
    return new DualNumber(this.real * r, this.dual * r);
  }
  Maths.DualNumber.prototype.multiply = function(d) {
    return new DualNumber(this.real * d.real, this.real * d.dual + this.dual *
      d.real);
  };
  // just like the complex
  // (a+eb)/(c+ed)=[(a+eb)(c-ed)]/c^2=a/c+e(bc-ad)/c^2, where e denotes nilpotent
  Maths.DualNumber.prototype.divide = function(d) {
    if (Maths.equal(d.real, 0)) {
      throw new Error("Divided by Zero!");
    }
    return new DualNumber(this.real / d.real, (this.dual * d.real - this.real *
      d.dual) / Maths.sqr(d.real));
  };
  // (1+e0)/(a+eb)=1/a-eb/a^2, where e denotes nilpotent
  Maths.DualNumber.prototype.inverse = function() {
    if (Maths.equal(this.real, 0)) {
      throw new Error("Non-inverse!");
    }
    return new DualNumber(1.0 / this.real, -this.dual / Maths.sqr(this.real));
  };
  // c+ed=(a+eb)^2=a^2+2eab --> a=sqrt(c), b=d/2a=d/(2sqrt(c))
  Maths.DualNumber.sqrt = function(d) {
    if (Maths.lessThan(this.real, 0)) {
      throw new Error("Illegal usage of DualNumber.sqrt!");
    }
    return new DualNumber(Math.sqrt(this.real), 0.5 * this.dual / Maths.sqr(
      this.real));
  };
  ///////////////////////////////////////////////////////////
  // DualQuaternion, combine dual number with quaternion
  // -- p + eq, where p,q is Quaternion and e*e=0
  Maths.DualQuaternion = function(real, dual) {
    this.real = real || new Quaternion(0, new Maths.Vector3(0,
      0, 1));
    this.dual = dual || new Quaternion(0, new Vector3(0, 0, 0));
  }
  Maths.DualQuaternion.prototype.copy = function(Q) {
    return new Maths.DualQuaternion(this.real.acopy(), this.dual.copy());
  }
  Maths.DualQuaternion.prototype.add = function(Q) {
    return new Maths.DualQuaternion(this.real.add(Q.real), this.dual.add(Q.dual));
  }
  Maths.DualQuaternion.prototype.multiply = function(Q) {
    return new Maths.DualQuaternion(
      this.real.multiply(Q.real),
      this.real.multiply(Q.dual).add(this.dual.multiply(Q.real))
    );
  }
  Maths.DualQuaternion.prototype.scale = function(r) {
    return new Maths.DualQuaternion(this.real.scale(r), this.dual.scale(r));
  }
  Maths.DualQuaternion.prototype.dot = function(Q) {
    return this.real.dot(Q.real);
  }
  Maths.DualQuaternion.prototype.conjugate1 = function() {
    return new Maths.DualQuaternion(this.real, this.dual.scale(-1.0));
  }
  Maths.DualQuaternion.prototype.conjugate2 = function() {
    return new Maths.DualQuaternion(this.real.conjugate(), this.dual.conjugate());
  }
  Maths.DualQuaternion.prototype.conjugate3 = function() {
    return new Maths.DualQuaternion(this.real.conjugate(), this.dual.conjugate()
      .scale(-1.0));
  }
  Maths.DualQuaternion.prototype.conjugate = Maths.DualQuaternion.prototype.conjugate2;
  // TODO:
  // Actually, we should make DualNumber a template. Then, DualNumber<Quaternion>
  // will generate all the rules for DualQuaternion.
  // Maybe append the same method of Quaternion to Number.prototype can solve this problem,
  // because the template need a uniform implentation for all data-types.

  /* norm(Q)
   * = sqrt(QQ*)
   * = sqrt(q1q1*+e(q1q2*+q2q1*))
   * = sqrt(q1q1*)+e[(q1q2*+q2q1*)/2]/sqrt(q1q1*)
   * = norm(q1)+e(q1.q2/norm(q1))
   */
  Maths.DualQuaternion.prototype.norm = function() {
    var real_norm = this.real.norm;
    return new DualNumber(real_norm, this.real.dot(this.dual) / real_norm);
  };
  /* Q/norm(Q)
   * = (q1+eq2)/[norm(q1)+e(q1.q2/norm(q1))]
   * = (q1+eq2)[norm(q1)-e(q1.q2/norm(q1))]/norm(q1)^2
   * = {q1norm(q1)+e[q2norm(q1)-q1(q1.q2/norm(q1))]}/norm(q1)^2
   * = q1/norm(q1)+e[q2/norm(q1)-q1(q1.q2/norm(q1)^3)]
   */
  Maths.DualQuaternion.prototype.normalize = function() {
    var inv = 1.0 / this.real.norm();
    return new Maths.DualQuaternion(this.real.scale(inv), this.dual
      .scale(inv).add(this.real.scale(-this.real.dot(this.dual) * inv *
        inv * inv)));
  }

  Maths.Polynomial = function(A) {
    this.A = A || [];
  };
  Maths.Polynomial.prototype.value = function(x, truncation) {
    var A = this.A,
      value = 0,
      i = this.A.length - 1;
    if (truncation && truncation < i) i = truncation;
    for (; i >= 0; --i) {
      value = value * x + A[i];
    }
    return value;
  };
  Number.prototype.toFixed = function(m) {
    var n = this,
      f = '',
      p = Math.pow(10, m); //p为10进制移位量;
    if (n < 0) n = -n, f = '-'; //把负数转为正数
    var a = Math.floor(n),
      b = n - a; //分离整数与小数
    b = Math.round(b * p); //移位并四舍五入
    if (b >= p) a++, b -= p; //进位
    if (m) b = '.' + (p + b + '').substr(1, m); //小数部分左边补0
    else b = '';
    return f + a + b;
  }
  Maths.Angle = function(dh, m, s) {
    if (dh * m < 0 || m * s < 0 || dh * s < 0 || m > 60 || s > 60) {
      throw new Error("Illegal angle expression!");
    }
    this.dh = dh || 0;
    this.m = m || 0;
    this.s = s || 0;
  };
  // format: 'hms' for hour-angle, and 'dms' for degree
  // for degree, if not contants the 'd', it should be started with '_'.
  Maths.Angle.prototype.toString = function(format, fixed) {
    var sign = (this.dh < 0 || this.m < 0 || this.s < 0) ? '-' : '+';
    format = format || "dms";
    fixed = fixed || 2;
    if (false == /^[h[d_]]?m?s?$/.test(format)) {
      throw new Error("Illegal format!");
    }
    var symbol = "hms";
    if (format[0] == 'd' || format[0] == '_') {
      symbol = "\u00b0\u2032\u2033";
    }
    var dh = Math.abs(this.dh);
    var m = Math.abs(this.m);
    var s = Math.abs(this.s);
    if (format == "d" || format == "h") {
      return String.sprintf(sign + "%." + fixed + "f" + symbol[0], dh +
        m /
        60 + s / 3600);
    } else if (format == "_m" || format == "m") {
      return String.sprintf(sign + "%." + fixed + "f" + symbol[1], dh *
        60 +
        m + s / 60);
    } else if (format == "_s" || format == "s") {
      return String.sprintf(sign + "%." + fixed + "f" + symbol[1], dh *
        3600 + m * 60 + s);
    } else if (format == "dm" || format == "hm") {
      m = (m + s / 60).toFixed(fixed);
      if (m >= 60) {
        dh += 1, m -= 60;
      }
      return String.sprintf(sign + "%d" + symbol[0] + "%." + fixed +
        "f" +
        symbol[1], dh, m);
    } else if (format == "_ms" || format == "ms") {
      m = dh * 60 + m;
      s = s.toFixed(fixed);
      if (s >= 60) {
        m += 1, s -= 60;
      }
      return String.sprintf(sign + "%d" + symbol[1] + "%." + fixed +
        "f" +
        symbol[2], m, s);
    } else { // hms, dms
      s = s.toFixed(fixed);
      if (s >= 60) {
        m += 1, s -= 60;
        if (m >= 60) {
          dh += 1, m -= 60;
        }
      }
      return String.sprintf(sign + "%d" + symbol[0] + "%02d" + symbol[1] +
        "%02." + fixed + "f" + symbol[2], dh, m, s);
    }
  };
  //将弧度转为字串
  // --fixed为小数保留位数
  // --format=dms 格式示例: -23°59' 48.23"
  // --format=d   格式示例: -23.59999°
  // --format=hms 格式示例:  18h 29m 44.52s
  Maths.formatRadian = function(radian, format, fixed) {
    var sign = 1;
    if (radian < 0) sign = -1, radian = -radian;
    radian *= (/^[d_]/.test(format) ? 180 : 12) / Math.PI;
    var degree_or_hour = Math.floor(radian);
    radian = (radian - degree_or_hour) * 60;
    var minute = Math.floor(radian);
    var second = (radian - minute) * 60;
    var angle = new Maths.Angle(sign * degree_or_hour, sign * minute,
      sign *
      second);
    return angle.toString(format, fixed);
  };
  //角度字符串转为弧度
  //-- 自动识别角度格式
  //-- 考虑到Unicode输入不方便，故“度分秒”符号支持与标点符号的”引号“混用
  Maths.parseAngle = function(string) {
    var re0 = /h|m|s|(-)|(°)|\'|\"|\u00b0|\u2032|\u2033/i;
    var re1 = new RegExp(
      "^\\s*([+-]?)\\s*(" // sign
      + "(\\d*\\.\\d+h)" // 5.5h, .5h
      + "|(\\d+h(\\.\\d*)?)" // 5h, 5h.5
      + "|(\\d*\\.\\d+m)" // 5.5m, .5m
      + "|(\\d+m(\\.\\d*)?)" // 5m, 5m.5
      + "|(\\d*\\.\\d+s)" // 5.5s .5s
      + "|(\\d+s(\\.\\d*)?)" // 5s 5s.5
      + "|((\\d+h)\\s*([0-5]?\\d\\.\\d+m))" // 55h55.5m
      + "|((\\d+h)\\s*([0-5]?\\dm(\\.\\d+)?))" // 55h55m, 55h55m.55
      + "|((\\d+m)\\s?([0-5]?\\d\\.\\d+s))" // 55m55.5s
      + "|((\\d+m)\\s*([0-5]?\\ds(\\.\\d+)?))" // 55m55s, 55m55s.55
      + "|((\\d+h)\\s*([0-5]?\\dm)\\s*([0-5]?\\d\\.\\d+s))" //55h55m55.55s
      + "|((\\d+h)\\s*([0-5]?\\dm)\\s*([0-5]?\\ds(\\.\\d+)?))" // 55h55m55s, 55h55m55s.55
      + ")\\s*$",
      "i");
    var re2 = new RegExp(
      "^\\s*([+-]?)\\s*(" // sign
      + "(\\d*\\.\\d+[°\u00b0])" // 5.5°, .5°
      + "|(\\d+[°\u00b0](\\.\\d*)?)" // 5°, 5°.5
      + "|(\\d*\\.\\d+['\u2032])" // 5.5', .5'
      + "|(\\d+['\u2032](\\.\\d*)?)" // 5', 5'.5
      + "|(\\d*\\.\\d+[\"\u2033])" // 5.5" .5"
      + "|(\\d+[\"\u2033](\\.\\d*)?)" // 5" 5".5
      + "|((\\d+[°\u00b0])\\s*([0-5]?\\d\\.\\d+['\u2032]))" // 55°55.5'
      + "|((\\d+[°\u00b0])\\s*([0-5]?\\d['\u2032](\\.\\d+)?))" // 55°55', 55°55'.55
      + "|((\\d+['\u2032])\\s?([0-5]?\\d\\.\\d+[\"\u2033]))" // 55'55.5"
      + "|((\\d+['\u2032])\\s*([0-5]?\\d[\"\u2033](\\.\\d+)?))" // 55'55", 55'55".55
      +
      "|((\\d+[°\u00b0])\\s*([0-5]?\\d['\u2032])\\s*([0-5]?\\d\\.\\d+[\"\u2033]))" //55°55'55.55"
      +
      "|((\\d+[°\u00b0])\\s*([0-5]?\\d['\u2032])\\s*([0-5]?\\d[\"\u2033](\\.\\d+)?))" // 55°55'55", 55°55'55".55
      + ")\\s*$",
      "i");
    var hour_angle = re1.test(string);
    var degree = re2.test(string);
    if (false == (hour_angle || degree)) {
      throw new TypeError("Parse Failed!");
    } else {
      var groups = hour_angle ? re1.exec(string) : re2.exec(string);
      if (groups) {
        var sign = groups[1] == '-' ? -1 : 1;
        var hour = Number((groups[3] || groups[4] || groups[13] ||
          groups[16] || groups[27] || groups[31] || "0h").replace(
          re0,
          ''));
        var minute = Number((groups[6] || groups[7] || groups[14] ||
          groups[17] || groups[20] || groups[23] || groups[28] ||
          groups[32] || "0m").replace(re0, ''));
        var second = Number((groups[9] || groups[10] || groups[21] ||
          groups[24] || groups[29] || groups[33] || "0s").replace(
          re0, ''));
        return sign * (hour * 3600 + minute * 60 + second) / Maths.SecondPerRadian *
          (hour_angle ? 15 : 1); // 地球每1小时转过15度
      };
    }
  }
})();
