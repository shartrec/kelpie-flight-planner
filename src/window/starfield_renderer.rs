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

use gl::types::{GLint, GLuint};
use gtk::GLArea;
use crate::window::map_utils::Vertex;

pub struct StarfieldRenderer {
    starfield_vertex_array: GLuint,
    starfield_vertex_buffer: GLuint,

}

impl StarfieldRenderer {
    pub fn new() -> Self {
        let mut starfield_vertex_buffer: GLuint = 0;
        let mut starfield_vertex_array: GLuint = 0;

        // THis is a simple square that covers the entire screen, which we will use to render the starfield background.
        // The vertex shader will then use the vertex positions to determine the direction of the stars and render them accordingly.
        let mut vertices: Vec<Vertex> = Vec::new();
        vertices.push(Vertex{position: [-1.0, -1.0, 0.0]});
        vertices.push(Vertex{position: [1.0, -1.0, 0.0]});
        vertices.push(Vertex{position: [1.0, 1.0, 0.0]});
        vertices.push(Vertex{position: [-1.0, -1.0, 0.0]});
        vertices.push(Vertex{position: [1.0, 1.0, 0.0]});
        vertices.push(Vertex{position: [-1.0, 1.0, 0.0]});

        unsafe {
            gl::GenVertexArrays(1, &mut starfield_vertex_array);
            gl::BindVertexArray(starfield_vertex_array);

            gl::GenBuffers(1, &mut starfield_vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, starfield_vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER, // target
                (vertices.len() * size_of::<Vertex>()) as gl::types::GLsizeiptr, // size of data in bytes
                vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl::VertexAttribPointer(
                0, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (3 * size_of::<f32>()) as GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null(), // offset of the first component
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, starfield_vertex_buffer);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        StarfieldRenderer {
            starfield_vertex_array,
            starfield_vertex_buffer,
        }
    }

    pub fn draw(&self, _area: &GLArea) {
        unsafe {
            gl::EnableVertexAttribArray(0);

            gl::BindVertexArray(self.starfield_vertex_array);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.starfield_vertex_buffer);

            gl::VertexAttribPointer(
                0, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (3 * size_of::<f32>()) as GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null(), // offset of the first component
            );

            gl::DrawArrays(gl::TRIANGLES, 0 as GLint, 6 as GLint);
        }
    }

    pub fn drop_buffers(&self) {
        unsafe {
            gl::DeleteBuffers(1, &self.starfield_vertex_buffer);
            gl::DeleteVertexArrays(1, &self.starfield_vertex_array);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);  // Vertex buffer
            gl::BindVertexArray(0);
        }
    }
}

