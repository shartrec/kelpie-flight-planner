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

use std::sync::{Arc, RwLockReadGuard};

use gl::types::GLuint;
use gtk::GLArea;

use crate::earth;
use crate::earth::spherical_projector::SphericalProjector;
use crate::model::location::Location;
use crate::model::navaid::{Navaid, NavaidType};
use crate::window::map_utils::Vertex;

pub struct NavaidRenderer {
    navaid_vertex_buffer: GLuint,
    navaid_vertex_arrays: GLuint,
    navaid_vor_index_buffer: GLuint,
    navaid_ndb_index_buffer: GLuint,
    navaid_vor: usize,
    navaid_ndb: usize,
}

impl NavaidRenderer {
    //todo drop buffers at end of program
    pub fn new() -> Self {
        let navaids = earth::get_earth_model().get_navaids().read().unwrap();
        let (vertices, indices_vor, indices_ndb) = Self::build_navaid_vertices(navaids);
        let mut navaid_vertex_buffer: GLuint = 0;
        let mut navaid_vertex_arrays: GLuint = 0;
        let mut navaid_vor_index_buffer: GLuint = 0;
        let mut navaid_ndb_index_buffer: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut navaid_vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, navaid_vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER, // target
                (vertices.len() * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr, // size of data in bytes
                vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::GenVertexArrays(1, &mut navaid_vertex_arrays);
            gl::BindVertexArray(navaid_vertex_arrays);

            gl::GenBuffers(1, &mut navaid_vor_index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, navaid_vor_index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices_vor.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                indices_vor.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::GenBuffers(1, &mut navaid_ndb_index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, navaid_ndb_index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices_ndb.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                indices_ndb.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        NavaidRenderer {
            navaid_vertex_buffer,
            navaid_vertex_arrays,
            navaid_vor_index_buffer,
            navaid_ndb_index_buffer,
            navaid_vor: indices_vor.len(),
            navaid_ndb: indices_ndb.len(),
        }
    }

    pub fn draw(&self, _area: &GLArea, ndb: bool) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.navaid_vertex_buffer);
            gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl::VertexAttribPointer(
                0, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (3 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null(), // offset of the first component
            );

            let mut point_size = 2.0;
            if ndb {
                gl::PointSize(point_size);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.navaid_ndb_index_buffer);
                gl::BindVertexArray(self.navaid_ndb_index_buffer);
                gl::DrawElements(
                    gl::POINTS, // mode
                    self.navaid_ndb as gl::types::GLsizei,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
                point_size *= 2.0;
            }

            gl::PointSize(point_size);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.navaid_vor_index_buffer);
            gl::BindVertexArray(self.navaid_vor_index_buffer);
            gl::DrawElements(
                gl::POINTS, // mode
                self.navaid_vor as gl::types::GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );

            gl::DisableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
        }
    }

    pub fn drop_buffers(&self) {
        unsafe {
            gl::DisableVertexAttribArray(0);
            gl::DeleteBuffers(1, &self.navaid_vertex_buffer.clone());
            gl::DeleteBuffers(1, &self.navaid_vor_index_buffer.clone());
            gl::DeleteBuffers(1, &self.navaid_ndb_index_buffer.clone());
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);  // Vertex buffer
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);  // Index buffer
            gl::BindVertexArray(0);
            gl::DeleteVertexArrays(1, &self.navaid_vertex_arrays);
        }
    }

    fn build_navaid_vertices(navaids: RwLockReadGuard<Vec<Arc<Navaid>>>) -> (Vec<Vertex>, Vec<u32>, Vec<u32>) {
        let projector = SphericalProjector::new(1.000);

        let mut vertices: Vec<Vertex> = Vec::with_capacity(1000);
        let mut indices_vor: Vec<u32> = Vec::with_capacity(100);
        let mut indices_ndb: Vec<u32> = Vec::with_capacity(100);

        for navaid in navaids.iter() {
            let position = projector.project(navaid.get_lat(), navaid.get_long());
            vertices.push(Vertex { position: position });

            // Now indices
            match navaid.get_type() {
                NavaidType::VOR => {
                    indices_vor.push(vertices.len() as u32 - 1);
                }
                _ => {
                    indices_ndb.push(vertices.len() as u32 - 1);
                }
            }
        }
        (vertices, indices_vor, indices_ndb)
    }
}
