// WGS84 parameters for EcEf to Geo conversions
const A: f64 = 6_378_137.0; // semi-major axis in meters
const E_SQ: f64 = 0.006_694_379_990_141_4; // eccentricity squared

#[derive(Debug)]
pub struct EcefCoord {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct GeoCoord {
    lat: f64,
    lon: f64,
    pub alt: f64,
}

impl EcefCoord {
    pub fn to_geo(&self) -> GeoCoord {
        /*! Based on python code to do the conversion */
        // Compute longitude in radians
        let lon = self.y.atan2(self.x);

        // Compute the distance from the Z-axis (p)
        let p = (self.x.powi(2) + self.y.powi(2)).sqrt();

        // Initial estimate for latitude
        let theta = self.z.atan2(p * (1.0 - E_SQ));
        let mut lat =
            (self.z + E_SQ * A * theta.sin().powi(3)).atan2(p - E_SQ * A * theta.cos().powi(3));

        // Iterative calculation to refine latitude
        loop {
            let n = A / (1.0 - E_SQ * lat.sin().powi(2)).sqrt();
            let new_lat = (self.z + E_SQ * n * lat.sin()).atan2(p);

            // If the latitude change is small, break the loop
            if (new_lat - lat).abs() < 1e-12 {
                break;
            }
            lat = new_lat;
        }

        // Calculate radius of curvature in the prime vertical
        let n = A / (1.0 - E_SQ * lat.sin().powi(2)).sqrt();
        // Altitude (h) is the radial distance minus the radius of curvature above
        let alt = p / lat.cos() - n;

        // Convert latitude and longitude from radians to degrees
        let lat_deg = lat.to_degrees(); // lat * (180.0 / PI)
        let lon_deg = lon.to_degrees(); // lon * (180.0 / PI)

        // (lat_deg, lon_deg, h)
        GeoCoord {
            lat: lat_deg,
            lon: lon_deg,
            alt,
        }
    }

    pub fn to_geo_olson(&self) -> GeoCoord {
        //! Based on C code from [Planet36](https://github.com/planet36/ecef-geodetic/blob/main/olson_1996/olson_1996.c) 
        let e2 = 6.694_379_990_137_799e-3;
        let a = 6_378_137.0;
        let a1 = 4.269_767_270_715_753e4;
        let a2: f64 = 1.823_091_254_607_545e9;
	    let a3: f64 = 1.429_172_228_981_241e2;
        let a4 = 4.557_728_136_518_863e9;
        let a5 = 4.284_058_993_005_565e4;
	    let a6 = 9.933_056_200_098_622e-1;

        let zp = self.z.abs();
	    let w2 = self.x * self.x + self.y * self.y;
        let w = w2.sqrt();
        let z2 = self.z * self.z;
        let r2 = w2 + z2;
        let r = r2.sqrt();
        if r < 100_000. {
            return GeoCoord {
                lat: 0.0,
                lon: 0.0,
                alt: -1.0e7,
            }
        }

        let lon_deg = self.y.atan2(self.x);
        let s2 = z2/r2;
        let c2 = w2/r2;
        let u = a2/r;
        let v = a3 - a4/r;
        let mut lat_deg: f64;
        let c: f64;
        let ss: f64;
        let s: f64;// = 0.0;
        
        if c2 > 0.4 {
            s = (zp / r) * (1.0 + c2 * (a1 + u + s2 * v) / r);
            lat_deg = s.asin();
            ss = s * s;
            c = (1.0 - ss).sqrt();
        } else {
            c = (w/r)*(1.0 - s2*(a5 - u - c2*v)/r);
            lat_deg = c.acos();
            ss = 1.0 - c*c;
            s = ss.sqrt();
        }

        let g = 1. - e2*ss;
        let rg = a / g.sqrt();
        let rf = a6*rg;
        let u = w - rg*c;
        let v = zp - rf*s;
        let f = c*u + s*v;
        let m = c*v - s*u;
        let p = m/(rf/g + f);
        lat_deg += p;

        let alt: f64 = f + m*p/2.0;
        if self.z < 0.0 {
            lat_deg = -lat_deg;
        }

        GeoCoord {
            lat: lat_deg.to_degrees(),
            lon: lon_deg.to_degrees(),
            alt,
        }
    }
}

/***************************
           Tests
***************************/
#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::approx_eq;
    use rstest::rstest;

    #[rstest]
    #[case(EcefCoord{x: 652954.1006, y: 4774619.7919, z: -4167647.7937},
    GeoCoord{lat: -41.04453, lon: 82.21280, alt: 2274.39966})]
    #[case(EcefCoord{x: 652954.1006, y: 4774619.7919, z: -2217647.7937},
    GeoCoord{lat: -24.88722, lon: 82.212809, alt: -1069542.17232})]
    #[case(EcefCoord{x: -2694044.4111565403, y: -4266368.805493665, z: 3888310.602276871},
    GeoCoord{lat: 37.80437, lon: -122.27080, alt: 0.00000})]
    fn ecef_to_geo_test(#[case] ecef: EcefCoord, #[case] expected: GeoCoord) {
        let actual = ecef.to_geo();
        println!("Actual result: {:?}", actual);
        println!("Expect result: {:?}", expected);

        // Test equality of floats up to 5 digits
        assert!(approx_eq!(
            f64,
            expected.lat,
            actual.lat,
            epsilon = 0.00003,
            ulps = 2
        ));
        assert!(approx_eq!(
            f64,
            expected.lon,
            actual.lon,
            epsilon = 0.00003,
            ulps = 2
        ));
        assert!(approx_eq!(
            f64,
            expected.alt,
            actual.alt,
            epsilon = 0.00003,
            ulps = 2
        ))
    }

    #[rstest]
    #[case(EcefCoord{x: 652954.1006, y: 4774619.7919, z: -4167647.7937},
    GeoCoord{lat: -41.04453, lon: 82.21280, alt: 2274.39966})]
    #[case(EcefCoord{x: 652954.1006, y: 4774619.7919, z: -2217647.7937},
    GeoCoord{lat: -24.88722, lon: 82.212809, alt: -1069542.17232})]
    #[case(EcefCoord{x: -2694044.4111565403, y: -4266368.805493665, z: 3888310.602276871},
    GeoCoord{lat: 37.80437, lon: -122.27080, alt: 0.00000})]
    fn ecef_to_geo_olson_test(#[case] ecef: EcefCoord, #[case] expected: GeoCoord) {
        let actual = ecef.to_geo_olson();
        println!("Actual result: {:?}", actual);
        println!("Expect result: {:?}", expected);

        // Test equality of floats up to 5 digits
        assert!(approx_eq!(
            f64,
            expected.lat,
            actual.lat,
            epsilon = 0.00003,
            ulps = 2
        ));
        assert!(approx_eq!(
            f64,
            expected.lon,
            actual.lon,
            epsilon = 0.00003,
            ulps = 2
        ));
        assert!(approx_eq!(
            f64,
            expected.alt,
            actual.alt,
            epsilon = 0.00003,
            ulps = 2
        ))
    }
}
