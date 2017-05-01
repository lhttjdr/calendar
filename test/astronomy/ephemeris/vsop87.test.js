import * as VSOP87 from '../../../src/astronomy/ephemeris/vsop87';
import * as Angle from '../../../src/math/angle';
import * as Decimal from '../../../src/math/decimal';


const earth=VSOP87.VSOP87A;

const position=VSOP87.earth_heliocentric_spherical_J2000(2457755,true,false,Angle.angle("1\""),3);

console.log(Angle.show(position[0]));
console.log(Angle.format(position[1],"d",2));
console.log(Decimal.show(position[2]));