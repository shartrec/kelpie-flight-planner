/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */

use std::ffi::CString;
use std::sync::{Arc, RwLockReadGuard};

use gl::types::GLuint;
use gtk::GLArea;

use crate::earth;
use crate::earth::spherical_projector::SphericalProjector;
use crate::model::airport::Airport;
use crate::model::location::Location;
use crate::window::render_gl;
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
                (vertices.len() * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr, // size of data in bytes
                vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::GenVertexArrays(1, &mut airport_vertex_arrays);
            gl::BindVertexArray(airport_vertex_arrays);

            gl::GenBuffers(1, &mut airport_large_index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, airport_large_index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices_large.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                indices_large.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::GenBuffers(1, &mut airport_medium_index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, airport_medium_index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices_medium.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                indices_medium.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::GenBuffers(1, &mut airport_small_index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, airport_small_index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices_small.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                indices_small.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );


            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }


        let airport_vert_shader = render_gl::Shader::from_vert_source(
            &CString::new(include_str!("program.vert")).unwrap()
        ).unwrap();

        let airport_frag_shader = render_gl::Shader::from_frag_source(
            &CString::new(include_str!("program.frag")).unwrap()
        ).unwrap();

        let airport_shader_program = render_gl::Program::from_shaders(
            &[airport_vert_shader, airport_frag_shader]
        ).unwrap();

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

    pub fn draw(&self, area: &GLArea, medium: bool, small: bool) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.airport_vertex_buffer);
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
            if small {
                gl::PointSize(point_size);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.airport_small_index_buffer);
                gl::BindVertexArray(self.airport_small_index_buffer);
                gl::DrawElements(
                    gl::POINTS, // mode
                    self.airport_small as gl::types::GLsizei,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
                point_size *= 2.0;
            }

            if medium {
                gl::PointSize(point_size);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.airport_medium_index_buffer);
                gl::BindVertexArray(self.airport_medium_index_buffer);
                gl::DrawElements(
                    gl::POINTS, // mode
                    self.airport_medium as gl::types::GLsizei,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
                point_size *= 2.0;
            }

            gl::PointSize(point_size);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.airport_large_index_buffer);
            gl::BindVertexArray(self.airport_large_index_buffer);
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
            gl::DisableVertexAttribArray(0);
            gl::DeleteBuffers(1, &self.airport_vertex_buffer.clone());
            gl::DeleteBuffers(1, &self.airport_large_index_buffer.clone());
            gl::DeleteBuffers(1, &self.airport_medium_index_buffer.clone());
            gl::DeleteBuffers(1, &self.airport_small_index_buffer.clone());
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);  // Vertex buffer
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);  // Index buffer
            gl::BindVertexArray(0);
            gl::DeleteVertexArrays(1, &self.airport_vertex_arrays);
        }
    }

    fn build_airport_vertices(airports: RwLockReadGuard<Vec<Arc<Airport>>>) -> (Vec<Vertex>, Vec<u32>, Vec<u32>, Vec<u32>) {
        let projector = SphericalProjector::new(1.000);

        let mut vertices: Vec<Vertex> = Vec::with_capacity(1000);
        let mut indices_large: Vec<u32> = Vec::with_capacity(100);
        let mut indices_medium: Vec<u32> = Vec::with_capacity(100);
        let mut indices_small: Vec<u32> = Vec::with_capacity(100);

        for airport in airports.iter() {
            let position = projector.project(airport.get_lat(), airport.get_long());
            vertices.push(Vertex { position: position });

            // Now indices
            if (airport.get_max_runway_length() > 10000) {
                indices_large.push(vertices.len() as u32 - 1);
            } else if (airport.get_max_runway_length() > 5000) {
                indices_medium.push(vertices.len() as u32 - 1);
            } else {
                indices_small.push(vertices.len() as u32 - 1);
            }
        }
        (vertices, indices_large, indices_medium, indices_small)
    }
}
