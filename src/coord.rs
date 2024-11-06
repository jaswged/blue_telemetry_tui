// WGS84 parameters for EcEf to Geo conversions
const A: f64 = 6378137.0; // semi-major axis in meters
const E: f64 = 0.081_819_190_842_622; // eccentricity

// pre-square to save calculations
const E_SQ: f64 = 0.006_694_379_990_141_4; // E.powi(2);

#[derive(Debug)]
struct EcefCoord {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct GeoCoord {
    lat: f64,
    lon: f64,
    alt: f64,
}

impl EcefCoord {
    fn new(x: f64, y: f64, z: f64, ) -> Self {
        EcefCoord { x, y, z }
    }

    fn to_geo(&self) -> GeoCoord {
        /*! Based on python code to do the conversion */
        // Compute longitude in radians
        let lon = self.y.atan2(self.x);

        // Compute the distance from the Z-axis (p)
        let p = (self.x.powi(2) + self.y.powi(2)).sqrt();

        // Initial estimate for latitude
        let theta = self.z.atan2(p * (1.0 - E_SQ));
        let mut lat = (self.z + E_SQ * A * theta.sin().powi(3))
            .atan2(p - E_SQ * A * theta.cos().powi(3));

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
        let lat_deg = lat.to_degrees();  // lat * (180.0 / PI)
        let lon_deg = lon.to_degrees();  // lon * (180.0 / PI)

        // (lat_deg, lon_deg, h)
        GeoCoord{ lat: lat_deg, lon: lon_deg, alt }
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;
    use rstest::rstest;
    use super::*;

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
        assert!( approx_eq!(f64, expected.lat, actual.lat, epsilon = 0.00003, ulps = 2));
        assert!( approx_eq!(f64, expected.lon, actual.lon, epsilon = 0.00003, ulps = 2));
        assert!( approx_eq!(f64, expected.alt, actual.alt, epsilon = 0.00003, ulps = 2))
    }
}