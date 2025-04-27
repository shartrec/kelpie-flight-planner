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

use gl::types::{GLint, GLuint};
use gtk::GLArea;
use log::info;
use crate::window::map_utils;
use crate::window::map_utils::Vertex;

pub struct ShorelineRenderer {
    shoreline_vertex_buffer: GLuint,
    shoreline_index_buffer: GLuint,
    shoreline_vertex_array: GLuint,
    shoreline_triangles: usize,
}

impl ShorelineRenderer {
    pub fn new(shape_data: &str) -> Self {
        let mut shoreline_builder = map_utils::GLShorelineBuilder::new(shape_data, 1.0);
        let (vertices, indices) = shoreline_builder.draw_shoreline();

        info!("Shoreline vertices: {}", vertices.len());

        let mut shoreline_vertex_buffer: GLuint = 0;
        let mut shoreline_index_buffer: GLuint = 0;
        let mut shoreline_vertex_array: GLuint = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut shoreline_vertex_array);
            gl::BindVertexArray(shoreline_vertex_array);

            gl::GenBuffers(1, &mut shoreline_vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, shoreline_vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER, // target
                (vertices.len() * size_of::<Vertex>()) as gl::types::GLsizeiptr, // size of data in bytes
                vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::GenBuffers(1, &mut shoreline_index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, shoreline_index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * size_of::<u32>()) as gl::types::GLsizeiptr,
                indices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (3 * size_of::<f32>()) as GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null(), // offset of the first component
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        ShorelineRenderer {
            shoreline_vertex_buffer,
            shoreline_index_buffer,
            shoreline_vertex_array,
            shoreline_triangles: indices.len(),
        }
    }

    pub fn draw(&self, _area: &GLArea) {
        unsafe {
            gl::BindVertexArray(self.shoreline_vertex_array);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.shoreline_index_buffer);

            gl::DrawElements(
                gl::TRIANGLES, // mode
                self.shoreline_triangles as gl::types::GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
    }

    pub fn drop_buffers(&self) {
        unsafe {
            gl::DeleteBuffers(1, &self.shoreline_vertex_buffer.clone());
            gl::DeleteBuffers(1, &self.shoreline_index_buffer.clone());
            gl::DeleteVertexArrays(1, &self.shoreline_vertex_array.clone());
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);  // Vertex buffer
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);  // Index buffer
            gl::BindVertexArray(0);  // Index buffer
        }
    }
}
