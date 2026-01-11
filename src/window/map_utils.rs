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

use log::error;
use crate::earth::spherical_projector::SphericalProjector;
use std::collections::HashMap;

#[repr(C)]
#[derive(Copy, Clone)]
pub(super) struct Vertex {
    pub(crate) position: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(super) struct Vertex2 {
    pub(crate) position: [f32; 3],
    // u & v are traditional names for texture coordinates
    pub(crate) uv: [f32; 2],
}


pub(super) struct GLSphereBuilder {
    vdata: [[f32; 3]; 12],
    tindices: [[usize; 3]; 20],
    projector: SphericalProjector,
}

// This helper draws a sphere of unit radius. For GL optimisation it creates each Vertex2 only once and generates an index "buffer" vector
// to describe the triangles. This reduces the Vertex2 count by 1/3rd.
impl GLSphereBuilder {
    pub fn new() -> Self {
        let x = 0.525_731_1;
        let z = 0.850_650_8;
        Self {
            // Statically define the starting regular icosahedron with "radius" = 1
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
            projector: SphericalProjector::new(1.0),
        }
    }

    pub fn draw_sphere(&mut self, radius: f32) -> (Vec<Vertex2>, Vec<u32>) {
        let mut vertices: Vec<Vertex2> = Vec::with_capacity(1000);
        let mut indeces: Vec<u32> = Vec::with_capacity(1000);
        let mut edge_cache: HashMap<(usize, usize), usize> = HashMap::with_capacity(2000);

        // We really draw a polyhedron starting with a regular icosahedron and
        // subdividing its faces iteratively to get the smooth sphere we require
        for i in 0..12 {
            // push the vertex into the vector
            let v1 = self.vdata[i];
            let pos =  self.scale(&v1, &radius);
            let uv= self.vertex_to_uv(&pos);
            let vertex = Vertex2 { position: pos, uv };
            vertices.push(vertex);

        }
        for i in 0..20 {
            // subdivide the triangles
            let i1 = self.tindices[i][0];
            let i2 = self.tindices[i][1];
            let i3 = self.tindices[i][2];
            self.subdivide(
                &mut vertices,
                &mut indeces,
                i1,
                i2,
                i3,
                3,
                &radius,
                &mut edge_cache,
            );
        }

        (vertices, indeces)
    }
    //noinspection RsExternalLinter
    fn subdivide(&mut self, vertices: &mut Vec<Vertex2>, indices: &mut Vec<u32>, i1: usize, i2: usize, i3: usize, depth: i32, radius: &f32, edge_cache: &mut HashMap<(usize, usize), usize>) {

        let p1 = vertices.get(i1).unwrap().position.clone();
        let p2 = vertices.get(i2).unwrap().position.clone();
        let p3 = vertices.get(i3).unwrap().position.clone();

        if depth == 0 {
            let mut uv1 = vertices.get(i1).unwrap().uv.clone();
            let mut uv2 = vertices.get(i2).unwrap().uv.clone();
            let mut uv3 = vertices.get(i3).unwrap().uv.clone();

            let u1 = uv1[0];
            let u2 = uv2[0];
            let u3 = uv3[0];

            let mut i1 = i1;
            let mut i2 = i2;
            let mut i3 = i3;
            // Check if the triangle crosses the 180 degree longitude (0.0 / 1.0 UV seam)
            // We check if the difference between any two U coords is larger than 0.5
            let cross_seam = ((u1 - u2).abs() > 0.5) || ((u1 - u3).abs() > 0.5) || ((u2 - u3).abs() > 0.5);

            if cross_seam {
                // Find the "outlier" vertices and shift them
                // If a vertex is near 0.0, but others are near 1.0,
                // we treat the 0.0 vertex as 1.0+ for this triangle only.
                if u1 < 0.5 {
                    uv1[0] += 1.0;
                    vertices.push(Vertex2 {position: p1, uv: uv1});
                    i1 = vertices.len() - 1;
                }
                if u2 < 0.5 {
                    uv2[0] += 1.0;
                    vertices.push(Vertex2 {position: p2, uv: uv2});
                    i2 = vertices.len() - 1;
                }
                if u3 < 0.5 {
                    uv3[0] += 1.0;
                    vertices.push(Vertex2 {position: p3, uv: uv3});
                    i3 = vertices.len() - 1;
                }

                // Note: This requires your OpenGL texture wrapping mode
                // to be set to GL_REPEAT so that a U of 1.1 wraps back to 0.1 correctly.
            }
            self.draw_triangle(indices, i1, i2, i3);
            return;
        }

        // Use edge midpoint cache so each edge midpoint is created only once
        let i12 = self.get_midpoint(i1, i2, *radius, vertices, edge_cache);
        let i23 = self.get_midpoint(i2, i3, *radius, vertices, edge_cache);
        let i31 = self.get_midpoint(i3, i1, *radius, vertices, edge_cache);

        self.subdivide(vertices, indices, i1, i12, i31, depth - 1, radius, edge_cache);
        self.subdivide(vertices, indices, i2, i23, i12, depth - 1, radius, edge_cache);
        self.subdivide(vertices, indices, i3, i31, i23, depth - 1, radius, edge_cache);
        self.subdivide(vertices, indices, i12, i23, i31, depth - 1, radius, edge_cache);
    }

    // Returns the index of the midpoint vertex between iA and iB, creating it if needed.
    fn get_midpoint(
        &mut self,
        i_a: usize,
        i_b: usize,
        radius: f32,
        vertices: &mut Vec<Vertex2>,
        edge_cache: &mut HashMap<(usize, usize), usize>,
    ) -> usize {
        let key = if i_a < i_b { (i_a, i_b) } else { (i_b, i_a) };
        if let Some(&idx) = edge_cache.get(&key) {
            return idx;
        }

        let a = vertices[i_a].position;
        let b = vertices[i_b].position;
        let mut m = [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
        self.normalize(&mut m);
        let pos = self.scale(&m, &radius);
        let uv = self.vertex_to_uv(&pos);
        let idx = vertices.len();
        vertices.push(Vertex2 { position: pos, uv });
        edge_cache.insert(key, idx);
        idx
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

    // Maps texture coordinates onto a sphere's vertices
    fn vertex_to_uv(&self, pos: &[f32; 3]) -> [f32; 2] {

        if let Ok(coord) = &self.projector.un_project(pos[0], pos[1], pos[2]) {
            let u = (coord[1] + 180.0) / 360.0;
            let v = (coord[0] + 90.0) / 180.0;
            [u, 1.0 - v]
        } else {
            error!("Failed to project sphere vertex to UV coordinates");
            [0., 0.]
        }
    }

}
