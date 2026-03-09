#!/usr/bin/env python3
# Generate data/jpl/elp_vs_jpl_de406_samples.csv for Rust test elpmpp02_vs_jpl_de406_samples.
# Usage: pip install jplephem, then place DE406 file and run this script.

import math
import os
import sys

ARCSEC_TO_RAD = math.pi / (180.0 * 3600.0)
EPSILON_RAD = (23 * 3600 + 26 * 60 + 21 + 0.41100) * ARCSEC_TO_RAD
PHI_RAD = (-0.05542) * ARCSEC_TO_RAD
AU_KM = 149597870.7

def gcrf_to_ecliptic_j2000_km(x_au, y_au, z_au):
    cz, sz = math.cos(-PHI_RAD), math.sin(-PHI_RAD)
    x1 = x_au * cz - y_au * sz
    y1 = x_au * sz + y_au * cz
    z1 = z_au
    ce, se = math.cos(EPSILON_RAD), math.sin(EPSILON_RAD)
    return (x1 * AU_KM, (y1 * ce + z1 * se) * AU_KM, (-y1 * se + z1 * ce) * AU_KM)

TEST_JDS = [
    (2200000.0,), (2268922.5,), (2305447.5,), (2433282.5,), (2444239.5,),
    (2451545.0,), (2455000.0,), (2473400.5,), (2500000.5,), (2637936.5,),
]

def main():
    try:
        import jplephem
    except ImportError:
        print("pip install jplephem", file=sys.stderr)
        return 1
    ephem_path = os.environ.get("JPLEPHEM_DE406", "de406")
    for name in ["de406.bsp", "linux_p1550p2650.406"]:
        p = os.path.join(os.path.dirname(__file__), "..", "data", "jpl", name)
        if os.path.exists(p):
            ephem_path = p
            break
    if not os.path.exists(ephem_path):
        print("DE406 not found", file=sys.stderr)
        return 1
    eph = jplephem.Ephemeris.open(ephem_path)
    moon_geo = 10
    out_dir = os.path.join(os.path.dirname(__file__), "..", "data", "jpl")
    os.makedirs(out_dir, exist_ok=True)
    out_path = os.path.join(out_dir, "elp_vs_jpl_de406_samples.csv")
    with open(out_path, "w") as f:
        f.write("jd_tdb,x_km,y_km,z_km\n")
        for row in TEST_JDS:
            jd_tdb = row[0]
            pos = eph.position(moon_geo, jd_tdb)
            x, y, z = gcrf_to_ecliptic_j2000_km(pos[0], pos[1], pos[2])
            f.write(f"{jd_tdb},{x:.6f},{y:.6f},{z:.6f}\n")
    print("Wrote", out_path)
    return 0

if __name__ == "__main__":
    sys.exit(main())
