/*
 * Copyright (c) 2003-2003-2023. Trevor Campbell and others.
 *
 * This file is part of Kelpie Flight Planner.
 *
 * Kelpie Flight Planner is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * Kelpie Flight Planner is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Kelpie Flight Planner; if not, write to the Free Software
 * Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
 *
 * Contributors:
 *      Trevor Campbell
 *
 */

//
// Module to calculate magnetic declination, magnetic field strength,
// inclination etc. for any point on the earth.</p>
// <p>Adapted from the geomagc software and World Magnetic Model of the NOAA
// Satellite and Information Service, National Geophysical Data Center</p>
// http://www.ngdc.noaa.gov/geomag/WMM/DoDWMM.shtml
// <p>© Deep Pradhan, 2017</p>*/
// Converted from original Java code

use std::str::SplitWhitespace;

use chrono::{Datelike, DateTime, Utc};

/** The input string array which contains each line of input for the wmm.cof input file.
* The columns in this file are as follows:    n,    m,    gnm,    hnm,    dgnm,    dhnm*/
const WMM_COF: [&str; 91] = [
    "    2020.0            WMM-2020        12/10/2019",
    "  1  0  -29404.5       0.0        6.7        0.0",
    "  1  1   -1450.7    4652.9        7.7      -25.1",
    "  2  0   -2500.0       0.0      -11.5        0.0",
    "  2  1    2982.0   -2991.6       -7.1      -30.2",
    "  2  2    1676.8    -734.8       -2.2      -23.9",
    "  3  0    1363.9       0.0        2.8        0.0",
    "  3  1   -2381.0     -82.2       -6.2        5.7",
    "  3  2    1236.2     241.8        3.4       -1.0",
    "  3  3     525.7    -542.9      -12.2        1.1",
    "  4  0     903.1       0.0       -1.1        0.0",
    "  4  1     809.4     282.0       -1.6        0.2",
    "  4  2      86.2    -158.4       -6.0        6.9",
    "  4  3    -309.4     199.8        5.4        3.7",
    "  4  4      47.9    -350.1       -5.5       -5.6",
    "  5  0    -234.4       0.0       -0.3        0.0",
    "  5  1     363.1      47.7        0.6        0.1",
    "  5  2     187.8     208.4       -0.7        2.5",
    "  5  3    -140.7    -121.3        0.1       -0.9",
    "  5  4    -151.2      32.2        1.2        3.0",
    "  5  5      13.7      99.1        1.0        0.5",
    "  6  0      65.9       0.0       -0.6        0.0",
    "  6  1      65.6     -19.1       -0.4        0.1",
    "  6  2      73.0      25.0        0.5       -1.8",
    "  6  3    -121.5      52.7        1.4       -1.4",
    "  6  4     -36.2     -64.4       -1.4        0.9",
    "  6  5      13.5       9.0       -0.0        0.1",
    "  6  6     -64.7      68.1        0.8        1.0",
    "  7  0      80.6       0.0       -0.1        0.0",
    "  7  1     -76.8     -51.4       -0.3        0.5",
    "  7  2      -8.3     -16.8       -0.1        0.6",
    "  7  3      56.5       2.3        0.7       -0.7",
    "  7  4      15.8      23.5        0.2       -0.2",
    "  7  5       6.4      -2.2       -0.5       -1.2",
    "  7  6      -7.2     -27.2       -0.8        0.2",
    "  7  7       9.8      -1.9        1.0        0.3",
    "  8  0      23.6       0.0       -0.1        0.0",
    "  8  1       9.8       8.4        0.1       -0.3",
    "  8  2     -17.5     -15.3       -0.1        0.7",
    "  8  3      -0.4      12.8        0.5       -0.2",
    "  8  4     -21.1     -11.8       -0.1        0.5",
    "  8  5      15.3      14.9        0.4       -0.3",
    "  8  6      13.7       3.6        0.5       -0.5",
    "  8  7     -16.5      -6.9        0.0        0.4",
    "  8  8      -0.3       2.8        0.4        0.1",
    "  9  0       5.0       0.0       -0.1        0.0",
    "  9  1       8.2     -23.3       -0.2       -0.3",
    "  9  2       2.9      11.1       -0.0        0.2",
    "  9  3      -1.4       9.8        0.4       -0.4",
    "  9  4      -1.1      -5.1       -0.3        0.4",
    "  9  5     -13.3      -6.2       -0.0        0.1",
    "  9  6       1.1       7.8        0.3       -0.0",
    "  9  7       8.9       0.4       -0.0       -0.2",
    "  9  8      -9.3      -1.5       -0.0        0.5",
    "  9  9     -11.9       9.7       -0.4        0.2",
    " 10  0      -1.9       0.0        0.0        0.0",
    " 10  1      -6.2       3.4       -0.0       -0.0",
    " 10  2      -0.1      -0.2       -0.0        0.1",
    " 10  3       1.7       3.5        0.2       -0.3",
    " 10  4      -0.9       4.8       -0.1        0.1",
    " 10  5       0.6      -8.6       -0.2       -0.2",
    " 10  6      -0.9      -0.1       -0.0        0.1",
    " 10  7       1.9      -4.2       -0.1       -0.0",
    " 10  8       1.4      -3.4       -0.2       -0.1",
    " 10  9      -2.4      -0.1       -0.1        0.2",
    " 10 10      -3.9      -8.8       -0.0       -0.0",
    " 11  0       3.0       0.0       -0.0        0.0",
    " 11  1      -1.4      -0.0       -0.1       -0.0",
    " 11  2      -2.5       2.6       -0.0        0.1",
    " 11  3       2.4      -0.5        0.0        0.0",
    " 11  4      -0.9      -0.4       -0.0        0.2",
    " 11  5       0.3       0.6       -0.1       -0.0",
    " 11  6      -0.7      -0.2        0.0        0.0",
    " 11  7      -0.1      -1.7       -0.0        0.1",
    " 11  8       1.4      -1.6       -0.1       -0.0",
    " 11  9      -0.6      -3.0       -0.1       -0.1",
    " 11 10       0.2      -2.0       -0.1        0.0",
    " 11 11       3.1      -2.6       -0.1       -0.0",
    " 12  0      -2.0       0.0        0.0        0.0",
    " 12  1      -0.1      -1.2       -0.0       -0.0",
    " 12  2       0.5       0.5       -0.0        0.0",
    " 12  3       1.3       1.3        0.0       -0.1",
    " 12  4      -1.2      -1.8       -0.0        0.1",
    " 12  5       0.7       0.1       -0.0       -0.0",
    " 12  6       0.3       0.7        0.0        0.0",
    " 12  7       0.5      -0.1       -0.0       -0.0",
    " 12  8      -0.2       0.6        0.0        0.1",
    " 12  9      -0.5       0.2       -0.0       -0.0",
    " 12 10       0.1      -0.9       -0.0       -0.0",
    " 12 11      -1.1      -0.0       -0.0        0.0",
    " 12 12      -0.3       0.5       -0.1       -0.1",
];

/** Mean radius of IAU-66 ellipsoid, in km*/
const IAU66_RADIUS: f64 = 6371.2;

/** Semi-major axis of WGS-1984 ellipsoid, in km*/
const WGS84_A: f64 = 6378.137;

/** Semi-minor axis of WGS-1984 ellipsoid, in km*/
const WGS84_B: f64 = 6356.7523142;

/** The maximum number of degrees of the spherical harmonic model*/
const MAX_DEG: usize = 12;

pub struct Geomagnetism {
    /** Geomagnetic declination (decimal degrees) [opposite of variation, positive Eastward/negative Westward]*/
    declination: f64,

    /** Geomagnetic inclination/dip angle (degrees) [positive downward]*/
    inclination: f64,

    /** Geomagnetic field intensity/strength (nano Teslas)*/
    intensity: f64,

    /** Geomagnetic horizontal field intensity/strength (nano Teslas)*/
    bh: f64,

    /** Geomagnetic vertical field intensity/strength (nano Teslas) [positive downward]*/
    bz: f64,

    /** Geomagnetic North South (northerly component) field intensity/strength (nano Tesla)*/
    bx: f64,

    /** Geomagnetic East West (easterly component) field intensity/strength (nano Teslas)*/
    by: f64,

    /** The maximum order of spherical harmonic model*/
    maxord: usize,

    /** The Gauss coefficients of main geomagnetic model (nt)*/
    c: [[f64; 13]; 13],

    /** The Gauss coefficients of secular geomagnetic model (nt/yr)*/
    cd: [[f64; 13]; 13],

    /** The time adjusted geomagnetic gauss coefficients (nt)*/
    tc: [[f64; 13]; 13],

    /** The theta derivative of p(n,m) (unnormalized)*/
    dp: [[f64; 13]; 13],

    /** The Schmidt normalization factors*/
    snorm: [f64; 169],

    /** The sine of (m*spherical coordinate longitude)*/
    sp: [f64; 13],

    /** The cosine of (m*spherical coordinate longitude)*/
    cp: [f64; 13],

    fn_: [f64; 13],

    fm: [f64; 13],

    /** The associated Legendre polynomials for m = 1 (unnormalized)*/
    pp: [f64; 13],

    k: [[f64; 13]; 13],

    /** The variables otime (old time), oalt (old altitude),
        * olat (old latitude), olon (old longitude), are used to
        * store the values used from the previous calculation to
        * save on calculation time if some inputs don't change*/
    otime: f64,
    oalt: f64,
    olat: f64,
    olon: f64,

    /** The date in years, for the start of the valid time of the fit coefficients*/
    epoch: f64,

    r: f64,
    d: f64,
    ca: f64,
    sa: f64,
    ct: f64,
    st: f64,
}

#[allow(dead_code)]
impl Geomagnetism {
    /** Initialise the instance without calculations*/
    fn create() -> Geomagnetism {
        let maxord = MAX_DEG;
        let sp = [0.0; 13];
        let mut cp = [1.0; 13];
        cp[0] = 1.0;
        let mut snorm = [0.0; 169];
        snorm[0] = 1.0;
        let mut pp = [0.0; 13];
        pp[0] = 1.0;
        let dp = [[0.0; 13]; 13];
        let mut c = [[0.0; 13]; 13];
        let mut cd = [[0.0; 13]; 13];

        let mut k = [[0.0; 13]; 13];
        let mut fn_ = [0.0; 13];
        let mut fm = [0.0; 13];
        let tc = [[0.0; 13]; 13];

        let epoch = WMM_COF[0]
            .split_whitespace()
            .next()
            .unwrap_or("")
            .parse::<f64>()
            .unwrap_or(0.0);
        let mut tokens: SplitWhitespace;
        let mut gnm: f64;
        let mut hnm: f64;
        let mut dgnm: f64;
        let mut dhnm: f64;
        {
            let mut i: usize = 1;
            let mut m: usize;
            let mut n: usize;
            while i < WMM_COF.len() {
                {
                    tokens = WMM_COF[i].split_whitespace();
                    n = tokens.next().unwrap_or("").parse::<usize>().unwrap_or(0);
                    m = tokens.next().unwrap_or("").parse::<usize>().unwrap_or(0);
                    gnm = tokens.next().unwrap_or("").parse::<f64>().unwrap_or(0.0);
                    hnm = tokens.next().unwrap_or("").parse::<f64>().unwrap_or(0.0);
                    dgnm = tokens.next().unwrap_or("").parse::<f64>().unwrap_or(0.0);
                    dhnm = tokens.next().unwrap_or("").parse::<f64>().unwrap_or(0.0);
                    if m <= n {
                        c[m][n] = gnm;
                        cd[m][n] = dgnm;
                        if m != 0 {
                            c[n][m - 1] = hnm;
                            cd[n][m - 1] = dhnm;
                        }
                    }
                }
                i += 1;
            }
        }

        // Convert schmidt normalized gauss coefficients to unnormalized
        snorm[0] = 1.0;
        let mut flnmj: f64;
        {
            let mut j: usize;
            let mut n: usize = 1;
            while n <= maxord {
                {
                    snorm[n] = snorm[n - 1] * (2.0 * n as f64 - 1.0) / n as f64;
                    j = 2;
                    {
                        let mut m: usize = 0;
                        let d1: usize = 1;
                        let mut d2: usize = (n - m + d1) / d1;
                        while d2 > 0 {
                            {
                                k[m][n] = (((n as i32 - 1) * (n as i32 - 1)) - (m * m) as i32)
                                    as f64
                                    / ((2 * n as i32 - 1) * (2 * n as i32 - 3)) as f64;
                                if m > 0 {
                                    flnmj = ((n - m + 1) * j) as f64 / (n + m) as f64;
                                    snorm[n + m * 13] = snorm[n + (m - 1) * 13] * flnmj.sqrt();
                                    j = 1;
                                    c[n][m - 1] *= snorm[n + m * 13];
                                    cd[n][m - 1] *= snorm[n + m * 13];
                                }
                                c[m][n] *= snorm[n + m * 13];
                                cd[m][n] *= snorm[n + m * 13];
                            }
                            d2 -= 1;
                            m += d1;
                        }
                    }

                    fn_[n] = (n + 1) as f64;
                    fm[n] = n as f64;
                }
                n += 1;
            }
        }

        k[1][1] = 0.0;
        fm[0] = 0.0;
        Self {
            declination: 0.0,
            inclination: 0.0,
            intensity: 0.0,
            bh: 0.0,
            bz: 0.0,
            bx: 0.0,
            by: 0.0,
            maxord,
            c,
            cd,
            tc,
            dp,
            snorm,
            sp,
            cp,
            fn_,
            fm,
            pp,
            k,
            otime: -1000.0,
            oalt: -1000.0,
            olat: -1000.0,
            olon: -1000.0,
            epoch,
            r: 0.0,
            d: 0.0,
            ca: 0.0,
            sa: 0.0,
            ct: 0.0,
            st: 0.0,
        }
    }

    /** Initialise the instance and calculate for given location, altitude and date
        * @param latitude Latitude in decimal degrees
        * @param longitude Longitude in decimal degrees
        * @param altitude Altitude in metres (with respect to WGS-1984 ellipsoid)
        * @param calendar Calendar for date of calculation*/
    pub fn new(
        latitude: f64,
        longitude: f64,
        altitude: Option<f64>,
        calendar: Option<DateTime<Utc>>,
    ) -> Geomagnetism {
        let mut geo = Self::create();
        geo.calculate(latitude, longitude, altitude, calendar);
        geo
    }

    /** Calculate for given location, altitude and date
        * @param latitude Latitude in decimal degrees
        * @param longitude Longitude in decimal degrees
        * @param altitude Altitude in metres (with respect to WGS-1984 ellipsoid)
        * @param calendar Calendar for date of calculation*/
    fn calculate(
        &mut self,
        latitude: f64,
        longitude: f64,
        altitude: Option<f64>,
        calendar: Option<DateTime<Utc>>,
    ) {
        let current_date = Utc::now();

        let date = calendar.unwrap_or(current_date);

        let max_days = if date.naive_utc().date().leap_year() {
            366.0
        } else {
            365.0
        };

        let altitude = altitude.unwrap_or(0.0);
        let rlon: f64 = longitude.to_radians();
        let rlat: f64 = latitude.to_radians();
        let altitude_km: f64 = altitude / 1000.0;
        let year_fraction: f64 = date.year() as f64 + date.ordinal() as f64 / max_days;
        let dt: f64 = year_fraction - self.epoch;
        let srlon: f64 = rlon.sin();
        let srlat: f64 = rlat.sin();
        let crlon: f64 = rlon.cos();
        let crlat: f64 = rlat.cos();
        let srlat2: f64 = srlat * srlat;
        let crlat2: f64 = crlat * crlat;
        let a2: f64 = WGS84_A * WGS84_A;
        let b2: f64 = WGS84_B * WGS84_B;
        let c2: f64 = a2 - b2;
        let a4: f64 = a2 * a2;
        let b4: f64 = b2 * b2;
        let c4: f64 = a4 - b4;
        self.sp[1] = srlon;
        self.cp[1] = crlon;
        // Convert from geodetic coords. to spherical coords.
        if altitude_km != self.oalt || latitude != self.olat {
            let q: f64 = (a2 - c2 * srlat2).sqrt();
            let q1: f64 = altitude_km * q;
            let q2: f64 = ((q1 + a2) / (q1 + b2)) * ((q1 + a2) / (q1 + b2));
            let r2: f64 = (altitude_km * altitude_km) + 2.0 * q1 + (a4 - c4 * srlat2) / (q * q);
            self.ct = srlat / (q2 * crlat2 + srlat2).sqrt();
            self.st = (1.0 - (self.ct * self.ct)).sqrt();
            self.r = r2.sqrt();
            self.d = (a2 * crlat2 + b2 * srlat2).sqrt();
            self.ca = (altitude_km + self.d) / self.r;
            self.sa = c2 * crlat * srlat / (self.r * self.d);
        }
        if longitude != self.olon {
            {
                let mut m: usize = 2;
                while m <= self.maxord {
                    {
                        self.sp[m] = self.sp[1] * self.cp[m - 1] + self.cp[1] * self.sp[m - 1];
                        self.cp[m] = self.cp[1] * self.cp[m - 1] - self.sp[1] * self.sp[m - 1];
                    }
                    m += 1;
                }
            }
        }
        let aor: f64 = IAU66_RADIUS / self.r;
        let mut ar: f64 = aor * aor;
        let mut br: f64 = 0.0;
        let mut bt: f64 = 0.0;
        let mut bp: f64 = 0.0;
        let mut bpp: f64 = 0.0;
        let mut par: f64;
        let mut parp: f64;
        let mut temp1: f64;
        let mut temp2: f64;
        {
            let mut n: usize = 1;
            while n <= self.maxord {
                {
                    ar *= aor;
                    {
                        let mut m: usize = 0;
                        let d3: usize = 1;
                        let mut d4: usize = (n + m + d3) / d3;
                        while d4 > 0 {
                            {
                                // Compute unnormalized associated legendre polynomials and derivatives via recursion relations
                                if altitude_km != self.oalt || latitude != self.olat {
                                    if n == m {
                                        self.snorm[n + m * 13] =
                                            self.st * self.snorm[n - 1 + (m - 1) * 13];
                                        self.dp[m][n] = self.st * self.dp[m - 1][n - 1]
                                            + self.ct * self.snorm[n - 1 + (m - 1) * 13];
                                    }
                                    if n == 1 && m == 0 {
                                        self.snorm[n + m * 13] =
                                            self.ct * self.snorm[n - 1 + m * 13];
                                        self.dp[m][n] = self.ct * self.dp[m][n - 1]
                                            - self.st * self.snorm[n - 1 + m * 13];
                                    }
                                    if n > 1 && n != m {
                                        if m > n - 2 {
                                            self.snorm[n - 2 + m * 13] = 0.0;
                                        }

                                        if m > n - 2 {
                                            self.dp[m][n - 2] = 0.0;
                                        }

                                        self.snorm[n + m * 13] = self.ct
                                            * self.snorm[n - 1 + m * 13]
                                            - self.k[m][n] * self.snorm[n - 2 + m * 13];
                                        self.dp[m][n] = self.ct * self.dp[m][n - 1]
                                            - self.st * self.snorm[n - 1 + m * 13]
                                            - self.k[m][n] * self.dp[m][n - 2];
                                    }
                                }
                                // Time adjust the gauss coefficients
                                if year_fraction != self.otime {
                                    self.tc[m][n] = self.c[m][n] + dt * self.cd[m][n];
                                    if m != 0 {
                                        self.tc[n][m - 1] =
                                            self.c[n][m - 1] + dt * self.cd[n][m - 1];
                                    }
                                }
                                // Accumulate terms of the spherical harmonic expansions
                                par = ar * self.snorm[n + m * 13];
                                if m == 0 {
                                    temp1 = self.tc[m][n] * self.cp[m];
                                    temp2 = self.tc[m][n] * self.sp[m];
                                } else {
                                    temp1 =
                                        self.tc[m][n] * self.cp[m] + self.tc[n][m - 1] * self.sp[m];
                                    temp2 =
                                        self.tc[m][n] * self.sp[m] - self.tc[n][m - 1] * self.cp[m];
                                }
                                bt -= ar * temp1 * self.dp[m][n];
                                bp += self.fm[m] * temp2 * par;
                                br += self.fn_[n] * temp1 * par;
                                // Special case: north/south geographic poles
                                if self.st == 0.0 && m == 1 {
                                    if n == 1 {
                                        self.pp[n] = self.pp[n - 1];
                                    } else {
                                        self.pp[n] = self.ct * self.pp[n - 1]
                                            - self.k[m][n] * self.pp[n - 2];
                                    }

                                    parp = ar * self.pp[n];
                                    bpp += self.fm[m] * temp2 * parp;
                                }
                            }
                            d4 -= 1;
                            m += d3;
                        }
                    }
                }
                n += 1;
            }
        }

        if self.st == 0.0 {
            bp = bpp;
        } else {
            bp /= self.st;
        }

        // Rotate magnetic vector components from spherical to geodetic coordinates
        // bx must be the east-west field component
        // by must be the north-south field component
        // bz must be the vertical field component.
        self.bx = -bt * self.ca - br * self.sa;
        self.by = bp;
        self.bz = bt * self.sa - br * self.ca;
        // Compute declination (dec), inclination (dip) and total intensity (ti)
        self.bh = ((self.bx * self.bx) + (self.by * self.by)).sqrt();
        self.intensity = ((self.bh * self.bh) + (self.bz * self.bz)).sqrt();
        //	Calculate the declination.
        self.declination = self.by.atan2(self.bx).to_degrees();
        self.inclination = self.bz.atan2(self.bh).to_degrees();
        self.otime = year_fraction;
        self.oalt = altitude_km;
        self.olat = latitude;
        self.olon = longitude;
    }

    /** @return Geomagnetic declination (degrees) [opposite of variation, positive Eastward/negative Westward]*/
    pub fn get_declination(&self) -> f64 {
        self.declination
    }

    /** @return Geomagnetic inclination/dip angle (degrees) [positive downward]*/
    pub fn get_inclination(&self) -> f64 {
        self.inclination
    }

    /** @return Geomagnetic field intensity/strength (nano Teslas)*/
    pub fn get_intensity(&self) -> f64 {
        self.intensity
    }

    /** @return Geomagnetic horizontal field intensity/strength (nano Teslas)*/
    pub fn get_horizontal_intensity(&self) -> f64 {
        self.bh
    }

    /** @return Geomagnetic vertical field intensity/strength (nano Teslas) [positive downward]*/
    pub fn get_vertical_intensity(&self) -> f64 {
        self.bz
    }

    /** @return Geomagnetic North South (northerly component) field intensity/strength (nano Tesla)*/
    pub fn get_north_intensity(&self) -> f64 {
        self.bx
    }

    /** @return Geomagnetic East West (easterly component) field intensity/strength (nano Teslas)*/
    pub fn get_east_intensity(&self) -> f64 {
        self.by
    }
}

#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use chrono::Utc;

    use super::Geomagnetism;

    #[test]
    fn test_declination() {
        let date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).latest();

        let geo = Geomagnetism::new(-34.1356, 150.9231, None, date);
        assert_between(geo.declination, 12.75, 12.85);
        let geo = Geomagnetism::new(51.48, 0.5, None, date);
        assert_between(geo.declination, 1.0, 1.1);
        let geo = Geomagnetism::new(35.55, 139.76, None, date);
        assert_between(geo.declination, -8.0, -7.8);
        let geo = Geomagnetism::new(13.24, -16.65, None, date);
        assert_between(geo.declination, -6.2, -6.0);
        let geo = Geomagnetism::new(40.6, -73.8, None, date);
        assert_between(geo.declination, -13.0, -12.5);
    }

    fn assert_between(variable: f64, bottom: f64, top: f64) -> bool {
        let result = variable >= bottom && variable <= top;
        if !result {
            assert!(
                result,
                "Variable {} not between {} and {}",
                variable, bottom, top
            );
        }
        result
    }
}
