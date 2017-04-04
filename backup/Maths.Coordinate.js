(function() {
  var Coordinate = Calendar.createNS("Calendar.Maths.Coordinate");
  var Maths = Calendar.importNS("Calendar.Maths");

  ////////////////////////////////////////////////////////////////
  // Class Sphere
  // -- A Class for point in Sphere Coordinate
  Coordinate.Sphere = function(r, theta, phi) {
    this.r = (r = r || 1.0);
    this.theta = theta || 0;
    this.phi = phi || 0;
    this.Point = this.toRectangular();
  };
  // Public:
  Coordinate.Sphere.prototype.toRectangular = function() {
    var r = this.r,
      theta = this.theta,
      phi = this.phi;
    with(Math) {
      var x = r * sin(theta) * cos(phi);
      var y = r * sin(theta) * sin(phi);
      var z = r * cos(theta);
    }
    return new Coordinate.Rectangular(x, y, z);
  };
  Coordinate.Sphere.prototype.rotateX = function(angle) {
    this.Point.rotate(angle, new Maths.Vector3(1, 0, 0));
    return this;
  };
  Coordinate.Sphere.prototype.rotateZ = function(angle) {
    this.Point.rotate(angle, new Maths.Vector3(0, 0, 1));
    return this;
  };
  Coordinate.Sphere.prototype.translate = function(vector) {
    this.Point.translate(vector);
    return this;
  };
  Coordinate.Sphere.prototype.transform = function() {
    if (arguments.length !== 0) {
      this.Point.transform.apply(this, arguments);
      return this;
    }
    return this.Point.transform().toSphere();
  };
  Coordinate.Sphere.prototype.toString = function() {
    return "r=" + this.r + ", θ=" + this.theta + ", φ=" + this.phi;
  };
  Coordinate.Sphere.prototype.includedAngle = function(point) {
    return this.Point.includedAngle(point.Point);
  };
  // End Class Sphere

  ////////////////////////////////////////////////////////////////
  // Class Rectangular
  // -- A Class for point in Rectangular Coordinate
  Coordinate.Rectangular = function(x, y, z) {
    this.x = x;
    this.y = y;
    this.z = z;
  };
  // Public:
  Coordinate.Rectangular.prototype.transform = function() {
    var _args = [];
    return function() {
      if (arguments.length === 0) {
        var len = _args.length;
        if (len) {
          var dq = _args[len - 1];
          for (var i = len - 2; i >= 0; i -= 1) {
            dq = dq.multiply(_args[i]).normalize();
          }
          var real = new Maths.Quaternion(1, new Maths.Vector3(0, 0, 0));
          var dual = new Maths.Quaternion(0, new Maths.Vector3(this.x,
            this.y, this.z));
          var pointDQ = new Maths.DualQuaternion(real, dual);
          pointDQ = dq.multiply(pointDQ).multiply(dq.conjugate3());
          _args.length = 0;
          return new Coordinate.Rectangular(pointDQ.dual.vector.x,
            pointDQ.dual.vector.y, pointDQ.dual.vector.z);
        } else {
          return new Coordinate.Rectangular(this.x, this.y, this.z);
        }
      } else {
        Array.prototype.push.apply(_args, arguments);
        return this;
      }
    }
  }();
  Coordinate.Rectangular.prototype.rotate = function(theta, vector) {
    var rotation = Maths.Coordinate.Rectangular.Rotation(theta, vector);
    this.transform(rotation);
  }
  Coordinate.Rectangular.prototype.translate = function(vector) {
    var translation = Maths.Coordinate.Rectangular.Translation(vector);
    this.transform(translation);
  }
  Coordinate.Rectangular.prototype.toSphere = function() {
    var x = this.x,
      y = this.y,
      z = this.z;
    with(Math) {
      var r = sqrt(x * x + y * y + z * z);
      var theta = Maths.normalizeToRangePlusMinusPi(Math.atan2(Math.sqrt(
        x * x + y * y), z));
      var phi = Maths.normalizeToRangeZeroDoublePi(Math.atan2(y, x));
    }
    return new Coordinate.Sphere(r, theta, phi);
  };
  Coordinate.Rectangular.prototype.toString = function() {
    return "x=" + this.x + ", y=" + this.y + ", z=" + this.z;
  };
  Coordinate.Rectangular.prototype.includedAngle = function(point) {
    var vector1 = new Maths.Vector3(this.x, this.y, this.z),
      vector2 = new Maths.Vector3(point.x, point.y, point.z);
    return Math.atan2(vector1.cross(vector2).norm(), vector1.dot(vector2));
  };
  // End Class Rectangular


  ////////////////////////////////////////////////////////////////
  // for rotate axis, here set theta=-theta
  Coordinate.Rectangular.Rotation = function(theta, vector) {
    theta = -theta;
    var real = new Maths.Quaternion(Math.cos(0.5 * theta), vector.normalize()
      .scale(Math.sin(0.5 * theta)));
    var dual = new Maths.Quaternion(0, new Maths.Vector3(0, 0, 0));
    return new Maths.DualQuaternion(real, dual);
  };
  // for rotate axis, here set vector=-vector
  Coordinate.Rectangular.Translation = function(vector) {
    vector = vector.scale(-1.0);
    var real = new Maths.Quaternion(1, new Maths.Vector3(0, 0, 0));
    var dual = new Maths.Quaternion(0, vector).scale(0.5);
    return new Maths.DualQuaternion(real, dual);
  }
  Coordinate.Rectangular.TransformBuilder = function() {
    this.transformSequence = [];
  }
  Coordinate.Rectangular.TransformBuilder.prototype.add = function(dq) {
    this.transformSequence.push(dq);
    return this;
  }
  Coordinate.Rectangular.TransformBuilder.prototype.build = function() {
    var len = this.transformSequence.length;
    var real = new Maths.Quaternion(1, new Maths.Vector3(0, 0, 0));
    var dual = new Maths.Quaternion(0, new Maths.Vector3(0, 0, 0));
    var dq = new Maths.DualQuaternion(real, dual);
    for (var i = len - 1; i >= 0; i -= 1) {
      dq = dq.multiply(this.transformSequence[i]).normalize();
    }
    return dq;
  }
})();
