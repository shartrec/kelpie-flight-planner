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
use log::error;
use shapefile::Polygon;
use shapefile::Shape;
use shapefile::ShapeReader;

// Read shape file into a Vector of shapes for the shorelines of the world.

pub fn read_shapes() -> Option<Vec<Polygon>> {
    let pref = crate::preference::manager();
    let path = pref.get::<String>(crate::preference::GSHHG_PATH);
    match path {
        Some(p) => {

            let mut world: Vec<Polygon> = Vec::new();

            match ShapeReader::from_path(&p) {
                Ok(mut reader) => {
                    for shape in reader.iter_shapes() {
                        match shape {
                            Ok(shape) => {
                                match shape {
                                    Shape::Polygon(pts) => world.push(pts),
                                    _ => {
                                        error!("World shoreline data in file is not polygons as expected: {}", &p);
                                        ()
                                    },
                                }
                            }
                            _ => {
                                error!("World shoreline data in file is not valid: {}", &p);
                                ()
                            },
                        }
                    }
                }
                _ => {
                    error! ("Unable to open file for world shoreline data: {}", p);
                    ()
                }
            }
            Some(world)
        }
        None => {
            error! ("Unable to find the path in preferences for world shoreline data");
            None
        },
    }
}


#[cfg(test)]
mod tests {
    use super::read_shapes;

    #[test]
    fn test_construct() {
        let v  = read_shapes();
        assert!(&v.is_some());
        let vec = v.unwrap();
        assert!(!vec.is_empty());
        println!("Num Polylines - {}", vec.len());
        for poly in vec {
            println!("Bounds {:?}", poly.bbox());
            println!("Poly has {} rings", poly.rings().len());
            for ring in poly.rings() {
                println!("Ring has {} points", ring.len());
                for point in ring.points() {
                    print!("Point - {}", point);
                    break;
                }
                break;
            }
            break;
        }
    }
}