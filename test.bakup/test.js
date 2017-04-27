const decimal=require("./decimal.js");
decimal.test();

const basic=require("./basic.js");
basic.test();

const vector=require("./vector.js");
vector.test();

const Atom = require('../lib/astronomy/atomsphere-refraction');
console.log(Angle.show(Atom.appreant(Angle.PI)));