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
use crate::model::airport::Airport;
use crate::model::location::Location;
use crate::window::map_utils::Vertex;

pub struct AirportRenderer {
    airport_vertex_buffer: GLuint,
    airport_vertex_arrays: GLuint,
    airport_large_index_buffer: GLuint,
    airport_medium_index_buffer: GLuint,
    airport_small_index_buffer: GLuint,
    airport_large: usize,
    airport_medium: usize,
    airport_small: usize,
}

impl AirportRenderer {
    //todo drop buffers at end of program
    pub fn new() -> Self {
        let airports = earth::get_earth_model().get_airports().read().unwrap();
        let (vertices, indices_large, indices_medium, indices_small) = Self::build_airport_vertices(airports);
        let mut airport_vertex_buffer: GLuint = 0;
        let mut airport_vertex_arrays: GLuint = 0;
        let mut airport_large_index_buffer: GLuint = 0;
        let mut airport_medium_index_buffer: GLuint = 0;
        let mut airport_small_index_buffer: GLuint = 0;

        unsafe {
            gl::GenBuffers(1, &mut airport_vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, airport_vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER, // target
                (vertices.len() * size_of::<Vertex>()) as gl::types::GLsizeiptr, // size of data in bytes
                vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::GenVertexArrays(1, &mut airport_vertex_arrays);
            gl::BindVertexArray(airport_vertex_arrays);

            gl::GenBuffers(1, &mut airport_large_index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, airport_large_index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices_large.len() * size_of::<u32>()) as gl::types::GLsizeiptr,
                indices_large.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::GenBuffers(1, &mut airport_medium_index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, airport_medium_index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices_medium.len() * size_of::<u32>()) as gl::types::GLsizeiptr,
                indices_medium.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::GenBuffers(1, &mut airport_small_index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, airport_small_index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices_small.len() * size_of::<u32>()) as gl::types::GLsizeiptr,
                indices_small.as_ptr() as *const gl::types::GLvoid, // pointer to data
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

        AirportRenderer {
            airport_vertex_buffer,
            airport_vertex_arrays,
            airport_large_index_buffer,
            airport_medium_index_buffer,
            airport_small_index_buffer,
            airport_large: indices_large.len(),
            airport_medium: indices_medium.len(),
            airport_small: indices_small.len(),
        }
    }

    pub fn draw(&self, _area: &GLArea, medium: bool, small: bool, shader_program_id: GLuint) {
        unsafe {
            gl::BindVertexArray(self.airport_vertex_arrays);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.airport_vertex_buffer);
            gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader

            let mut point_size = 2.0;
            if small {
                let c = gl::GetUniformLocation(shader_program_id, b"pointSize\0".as_ptr() as *const gl::types::GLchar);
                gl::ProgramUniform1f(shader_program_id, c, point_size);

                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.airport_small_index_buffer);
                gl::DrawElements(
                    gl::POINTS, // mode
                    self.airport_small as gl::types::GLsizei,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
                point_size += 2.0;
            }

            if medium {
                let c = gl::GetUniformLocation(shader_program_id, b"pointSize\0".as_ptr() as *const gl::types::GLchar);
                gl::ProgramUniform1f(shader_program_id, c, point_size);

                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.airport_medium_index_buffer);
                gl::DrawElements(
                    gl::POINTS, // mode
                    self.airport_medium as gl::types::GLsizei,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
                point_size += 2.0;
            }

            let c = gl::GetUniformLocation(shader_program_id, b"pointSize\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniform1f(shader_program_id, c, point_size);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.airport_large_index_buffer);
            gl::DrawElements(
                gl::POINTS, // mode
                self.airport_large as gl::types::GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );

            gl::DisableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
        }
    }

    pub fn drop_buffers(&self) {
        unsafe {
            gl::DeleteBuffers(1, &self.airport_vertex_buffer.clone());
            gl::DeleteBuffers(1, &self.airport_large_index_buffer.clone());
            gl::DeleteBuffers(1, &self.airport_medium_index_buffer.clone());
            gl::DeleteBuffers(1, &self.airport_small_index_buffer.clone());
            gl::DeleteVertexArrays(1, &self.airport_vertex_arrays);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);  // Vertex buffer
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);  // Index buffer
            gl::BindVertexArray(0);
        }
    }

    fn build_airport_vertices(airports: RwLockReadGuard<Vec<Arc<Airport>>>) -> (Vec<Vertex>, Vec<u32>, Vec<u32>, Vec<u32>) {
        let projector = SphericalProjector::new(1.000);

        let mut vertices = Vec::with_capacity(airports.len());
        let mut indices_large = Vec::with_capacity(1000);
        let mut indices_medium = Vec::with_capacity(6000);
        let mut indices_small = Vec::with_capacity(30000);

        for (i, airport) in airports.iter().enumerate() {
            let position = projector.project(airport.get_lat(), airport.get_long());
            vertices.push(Vertex { position });

            // Now indices
            if airport.get_max_runway_length() > 10000 {
                indices_large.push(i as u32);
            } else if airport.get_max_runway_length() > 5000 {
                indices_medium.push(i as u32);
            } else {
                indices_small.push(i as u32);
            }
        }
        indices_large.shrink_to_fit();
        indices_medium.shrink_to_fit();
        indices_small.shrink_to_fit();
        (vertices, indices_large, indices_medium, indices_small)
    }
}
