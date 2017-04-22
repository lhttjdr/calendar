import * as b03 from './b03.js';
import * as iau1976 from './iau1976.js';
import * as iau2000 from './iau2000.js';
import * as p03 from './p03.js';

export const B03 = b03;
export const P03 = p03;
export const IAU1976 = iau1976;
export const IAU2000 = iau2000;

// default use p03
export const epsilon = p03.epsilon;
