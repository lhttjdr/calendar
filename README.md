# Chinese Lunar Calendar
It is a javascript program for Chinese lunnar calendar based on astronomical calculation. (Not completed yet)

A lunar month is the time between two successive syzygies. In Chinese lunar calendar, the time of new moon is used. Hence, the most important thing is to calculate the time of a straight-line configuration of Sun, Earth and Moon.

## Main theories 
### Position of the Sun: VSOP87
The semi-analytic planetary theory VSOP87 (French: Variations Séculaires des Orbites Planétaires, abbreviated as VSOP) was developed by the scientists at the Bureau des Longitudes in Paris, France.

### Position of Moon: ELP/MPP02
Éphéméride Lunaire Parisienne is a lunar theory developed by Jean Chapront, Michelle Chapront-Touzé, and others at the Bureau des Longitudes in the 1970s to 1990s.
- ELP 2000-82 (Chapront-Touze, Chapront, 1983), 
- ELP 2000-85 (Chapront-Touze, Chapront, 1988),
- ELP 2000-96, version used for analysing Lunar Laser Ranging (LLR).
It is not sufficiently accurate to predict the Moon's position. An attempt was made to improve the planetary terms with the ELP/MPP02 lunar theory (Chapront, Francou, 2003), which is a semi-analytical solution for the 
orbital motion of the Moon. The main differences from ELP2000-82B is the use of the new planetary perturbations MPP01 (Bidart, 2000) and the contribution 
of the LLR observations provided since 1970.

**Advatages of ELP**: It can be truncated to a lower level of accuracy for faster computation.

### Precession and nutation: IAU1980 (nutation, 1982) & IAU2000 (precession)
IAU(International Astronomical Union) precession-nutation model/theory.

### ΔT: Polynomial expressions published by NASA
In precise timekeeping, ΔT (Delta T, delta-T, deltaT, or DT) is the time difference obtained by subtracting Universal Time (UT) from Terrestrial Time (TT): ΔT = TT − UT.

### Atmospheric refraction
Bennett, G.G. (1982). "The Calculation of Astronomical Refraction in Marine Navigation". Journal of Navigation. 35: 255–259
Sæmundsson, Þorsteinn (1986). "Astronomical Refraction". Sky and Telescope. 72: 70.

## TODO
### Position of the Sun
Change to models of the Solar System produced at the Jet Propulsion Laboratory in Pasadena, California.
 - DE200: 1984~2002
 - DE405: 2003~2014
 - DE430: 2015~current
 Note: The full name is Jet Propulsion Laboratory Development Ephemeris (followed by a number), whose abbreviation is JPL DE(number), or just DE(number).

### Position of Moon
None

### Precession and nutation
IAU 2006

### ΔT: Polynomial expressions published by NASA
None

### Atmospheric refraction
Standards Of Fundamental Astronomy; SOFA Astrometry Tools (PDF) (Software version 11; Document 1.6 ed.), International Astronomical Union, 2014, pp. 12, 71–73, retrieved 23 June 2016
