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
#![allow(unsafe_code)]

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
    navaid_index_buffers: [u32; 2],
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
        let mut navaid_index_buffers = [0u32; 2];
        unsafe {
            gl::GenVertexArrays(1, &mut navaid_vertex_arrays);
            gl::BindVertexArray(navaid_vertex_arrays);

            gl::GenBuffers(1, &mut navaid_vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, navaid_vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER, // target
                (vertices.len() * size_of::<Vertex>()) as gl::types::GLsizeiptr, // size of data in bytes
                vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::GenBuffers(2, navaid_index_buffers.as_mut_ptr());
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, navaid_index_buffers[0]);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices_vor.len() * size_of::<u32>()) as gl::types::GLsizeiptr,
                indices_vor.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, navaid_index_buffers[1]);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices_ndb.len() * size_of::<u32>()) as gl::types::GLsizeiptr,
                indices_ndb.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl::VertexAttribPointer(
                0, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (3 * size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null(), // offset of the first component
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        NavaidRenderer {
            navaid_vertex_buffer,
            navaid_vertex_arrays,
            navaid_index_buffers,
            navaid_vor: indices_vor.len(),
            navaid_ndb: indices_ndb.len(),
        }
    }

    pub fn draw(&self, _area: &GLArea, ndb: bool, shader_program_id: GLuint) {
        unsafe {
            gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl::BindVertexArray(self.navaid_vertex_arrays);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.navaid_vertex_buffer);

            gl::VertexAttribPointer(
                0, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (3 * size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null(), // offset of the first component
            );

            let mut point_size = 2.0;
            if ndb {
                let c = gl::GetUniformLocation(shader_program_id, b"pointSize\0".as_ptr() as *const gl::types::GLchar);
                gl::ProgramUniform1f(shader_program_id, c, point_size);

                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.navaid_index_buffers[1]);
                gl::DrawElements(
                    gl::POINTS, // mode
                    self.navaid_ndb as gl::types::GLsizei,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
                point_size += 2.0;
            }

            let c = gl::GetUniformLocation(shader_program_id, b"pointSize\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniform1f(shader_program_id, c, point_size);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.navaid_index_buffers[0]);
            gl::DrawElements(
                gl::POINTS, // mode
                self.navaid_vor as gl::types::GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
    }

    pub fn drop_buffers(&self) {
        unsafe {
            gl::DeleteBuffers(1, &self.navaid_vertex_buffer);
            gl::DeleteBuffers(2, self.navaid_index_buffers.as_ptr());
            gl::DeleteVertexArrays(1, &self.navaid_vertex_arrays);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);  // Vertex buffer
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);  // Index buffer
            gl::BindVertexArray(0);
        }
    }

    fn build_navaid_vertices(navaids: RwLockReadGuard<Vec<Arc<Navaid>>>) -> (Vec<Vertex>, Vec<u32>, Vec<u32>) {
        let projector = SphericalProjector::new(1.000);

        let mut vertices = Vec::with_capacity(navaids.len());
        let mut indices_vor = Vec::with_capacity(4000);
        let mut indices_ndb = Vec::with_capacity(10000);

        for (i, navaid) in navaids.iter().enumerate() {
            let position = projector.project(navaid.get_lat(), navaid.get_long());
            vertices.push(Vertex { position });

            // Now indices
            match navaid.get_type() {
                NavaidType::Vor => indices_vor.push(i as u32),
                _ => indices_ndb.push(i as u32),
            }
        }
        indices_vor.shrink_to_fit();
        indices_ndb.shrink_to_fit();
        (vertices, indices_vor, indices_ndb)
    }
}
