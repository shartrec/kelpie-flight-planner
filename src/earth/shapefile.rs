/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use log::error;
use shapefile::ShapeReader;
use shapefile::Shape;
use shapefile::Polygon;

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