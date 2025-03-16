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
use std::io::Cursor;
use std::ops::Deref;

use log::error;
use rust_embed::RustEmbed;
use shapefile::Polygon;
use shapefile::Shape;
use shapefile::ShapeReader;

// This handles the use of embedded files
#[derive(RustEmbed)]
#[folder = "resources/embedded/"]
struct Asset;

// Read shape file into a Vector of shapes for the shorelines of the world.
pub fn read_shapes(shape_data: &str) -> Option<Vec<Polygon>> {
    let mut world: Vec<Polygon> = Vec::new();

    match Asset::get(shape_data) {
        Some(embedded_file) => {
            let data = embedded_file.data;
            let sb = Cursor::new(data.deref());
            match ShapeReader::new(sb) {
                Ok(reader) => {
                    if let Ok(shapes) = reader.read() {
                        for shape in shapes.iter() {
                            match shape {
                                Shape::Polygon(pts) => world.push(pts.clone()),
                                _ => {
                                    error!("World shoreline data in file is not polygons as expected: ");
                                }
                            }
                        }
                    }
                }
                _ => {
                    error!("Unable to open file for world shoreline data");
                }
            }
        }
        None => {
            error!("Unable to find the path in preferences for world shoreline data");
        }
    }
    Some(world)
}


#[cfg(test)]
mod tests {
    use super::read_shapes;

    #[test]
    fn test_construct() {
        let v = read_shapes("GSHHS_l_L1.shp");
        assert!(&v.is_some());
        let vec = v.unwrap();
        assert!(!vec.is_empty());
        println!("Num Polylines - {}", vec.len());
        if let Some(poly) = vec.into_iter().next() {
            println!("Bounds {:?}", poly.bbox());
            println!("Poly has {} rings", poly.rings().len());
            if let Some(ring) = poly.rings().iter().next() {
                println!("Ring has {} points", ring.len());
                if let Some(point) = ring.points().iter().next() {
                    print!("Point - {}", point);
                }
            }
        };
    }
}