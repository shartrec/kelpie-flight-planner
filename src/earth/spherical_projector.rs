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

use std::f32::consts::PI;

pub struct SphericalProjector {
    r: f32,
    _90rad: f32,
}

impl SphericalProjector {
    pub fn new(size: f32) -> Self {
        Self {
            r: size,
            _90rad: PI / 2.0,
        }
    }
    /**
     * @param lat
     * @param lon
     * @return
     */
    pub fn project(&self, lat: &f64, lon: &f64) -> [f32; 3] {
        let lat1: f32 = lat.to_radians() as f32;
        let lon1: f32 = lon.to_radians() as f32;

        let x: f32 = self.r * lat1.cos() * lon1.sin();
        let y: f32 = self.r * lat1.sin();
        let z: f32 = self.r * lat1.cos() * (lon1 + self._90rad).sin();
        // For OpenGL we flip the z-axis
        [x, y, -z]
    }

    pub fn un_project(&self, x: f32, y: f32, z: f32) -> Result<[f32; 2], String> {
        let y1: f32;
        if y > self.r {
            y1 = self.r;
        } else if y < -self.r {
            y1 = -self.r;
        } else {
            y1 = y;
        }
        let lat: f32 = self._90rad - (y1 / self.r).acos();
        let lon: f32 = x.atan2(-z);
        if lat.is_nan() || lon.is_nan() {
            //$NON-NLS-1$
            Err("Not in map".to_string())
        } else {
            Ok([lat.to_degrees(), lon.to_degrees()])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SphericalProjector;

    #[test]
    fn test_full_circle() {
        let proj = SphericalProjector::new(50000.0);
        let _3dpoint = proj.project(&-34.0, &151.0);
        println!("Projected point: {:?}", _3dpoint);

        let _coords = proj.un_project(_3dpoint[0], _3dpoint[1], _3dpoint[2]);
        println!("Projected point: {:?}", _coords);

        let x = _coords.unwrap();
        assert_eq!(x[0].round(), -34.0);
        assert_eq!(x[1].round(), 151.0);
    }
}