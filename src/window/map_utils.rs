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
#![forbid(unsafe_code)]

use shapefile::Polygon;

use crate::earth::spherical_projector::SphericalProjector;

#[derive(Copy, Clone)]
pub(super) struct Vertex {
    pub(crate) position: [f32; 3],
}


pub(super) struct GLSphereBuilder {
    vdata: [[f32; 3]; 12],
    tindices: [[usize; 3]; 20],
}

// This helper draws a sphere of unit radius. For GL optimisation it creates each Vertex only once and generates an index "buffer" vector
// to describe the triangles. This reduces the Vertex count by 1/3rd.
impl GLSphereBuilder {
    pub fn new() -> Self {
        let x = 0.525_731_1;
        let z = 0.850_650_8;
        Self {
            // Statically define the starting regular isocahedron with "radius" = 1
            vdata: [
                [-x, 0.0, z],
                [x, 0.0, z],
                [-x, 0.0, -z],
                [x, 0.0, -z],
                [0.0, z, x],
                [0.0, z, -x],
                [0.0, -z, x],
                [0.0, -z, -x],
                [z, x, 0.0],
                [-z, x, 0.0],
                [z, -x, 0.0],
                [-z, -x, 0.0],
            ],
            tindices: [
                [0, 4, 1],
                [0, 9, 4],
                [9, 5, 4],
                [4, 5, 8],
                [4, 8, 1],
                [8, 10, 1],
                [8, 3, 10],
                [5, 3, 8],
                [5, 2, 3],
                [2, 7, 3],
                [7, 10, 3],
                [7, 6, 10],
                [7, 11, 6],
                [11, 0, 6],
                [0, 1, 6],
                [6, 1, 10],
                [9, 0, 11],
                [9, 11, 2],
                [9, 2, 5],
                [7, 2, 11],
            ],
        }
    }

    pub fn draw_sphere(&mut self, radius: f32) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices: Vec<Vertex> = Vec::with_capacity(1000);
        let mut indeces: Vec<u32> = Vec::with_capacity(1000);

        // We really draw a polyhedron starting with a regular isocahedron and
        // subdividing its faces iteratively to get the smooth sphere we require
        for i in 0..20 {
            // push the vertex into the vector
            let v1 = self.vdata[self.tindices[i][0]];
            let v2 = self.vdata[self.tindices[i][1]];
            let v3 = self.vdata[self.tindices[i][2]];
            vertices.push(Vertex { position: self.scale(&v1, &radius) });
            let i1 = vertices.len() - 1;
            vertices.push(Vertex { position: self.scale(&v2, &radius) });
            let i2 = vertices.len() - 1;
            vertices.push(Vertex { position: self.scale(&v3, &radius) });
            let i3 = vertices.len() - 1;

            self.subdivide(
                &mut vertices,
                &mut indeces,
                i1,
                i2,
                i3,
                6,
                &radius,
            );
        }

        (vertices, indeces)
    }

    //noinspection RsExternalLinter
    fn subdivide(&mut self, vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, i1: usize, i2: usize, i3: usize, depth: i32, radius: &f32) {
        let mut v12: [f32; 3] = [0.0; 3];
        let mut v23: [f32; 3] = [0.0; 3];
        let mut v31: [f32; 3] = [0.0; 3];

        if depth == 0 {
            self.draw_triangle(indices, i1, i2, i3);
            return;
        }
        let v1 = &vertices.get(i1).unwrap().position;
        let v2 = &vertices.get(i2).unwrap().position;
        let v3 = &vertices.get(i3).unwrap().position;

        for i in 0..3 {
            v12[i] = v1[i] + v2[i];
            v23[i] = v2[i] + v3[i];
            v31[i] = v3[i] + v1[i];
        }
        self.normalize(&mut v12);
        self.normalize(&mut v23);
        self.normalize(&mut v31);

        vertices.push(Vertex { position: self.scale(&v12, radius) });
        let i12 = vertices.len() - 1;
        vertices.push(Vertex { position: self.scale(&v23, radius) });
        let i23 = vertices.len() - 1;
        vertices.push(Vertex { position: self.scale(&v31, radius) });
        let i31 = vertices.len() - 1;

        self.subdivide(vertices, indices, i1, i12, i31, depth - 1, radius);
        self.subdivide(vertices, indices, i2, i23, i12, depth - 1, radius);
        self.subdivide(vertices, indices, i3, i31, i23, depth - 1, radius);
        self.subdivide(vertices, indices, i12, i23, i31, depth - 1, radius);
    }

    fn normalize(&self, v: &mut [f32; 3]) {
        let d = f32::sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
        if d != 0.0 {
            v[0] /= d;
            v[1] /= d;
            v[2] /= d;
        }
    }

    fn draw_triangle(&mut self, indices: &mut Vec<u32>, i1: usize, i2: usize, i3: usize) {
        indices.push(i1 as u32);
        indices.push(i2 as u32);
        indices.push(i3 as u32);
    }

    fn scale(&self, v: &[f32; 3], radius: &f32) -> [f32; 3] {
        [v[0] * radius, v[1] * radius, v[2] * radius]
    }
}

pub(super) struct GLShorelineBuilder {
    shoreline: Option<Vec<Polygon>>,
    projector: SphericalProjector,
}

impl GLShorelineBuilder {
    pub fn new() -> Self {
        Self {
            shoreline: crate::earth::shapefile::read_shapes(),
            projector: SphericalProjector::new(1.000),
        }
    }

    pub fn draw_shoreline(&mut self) -> (Vec<Vertex>, Vec<(usize, usize)>) {
        let mut vertices: Vec<Vertex> = Vec::with_capacity(1000);
        let mut rings: Vec<(usize, usize)> = Vec::with_capacity(1000);

        // We need to build a single structure for each ring and later a vertex buffer for for each
        if let Some(polygons) = &self.shoreline {
            for poly in polygons {
                for ring in poly.rings() {
                    let ring_start = vertices.len();
                    let ring_len = ring.len();
                    for point in ring.points() {
                        let position = self.projector.project(&point.y, &point.x);
                        vertices.push(Vertex { position });
                    }
                    rings.push((ring_start, ring_len));
                }
            }
        }
        (vertices, rings)
    }
}