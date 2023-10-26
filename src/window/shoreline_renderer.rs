/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */

use gl::types::{GLint, GLuint};
use gtk::GLArea;

use crate::window::map_utils;
use crate::window::map_utils::Vertex;

pub struct ShorelineRenderer {
    shoreline_vertex_buffer: GLuint,
    shoreline_rings: Vec<(usize, usize)>,
}

impl ShorelineRenderer {
    //todo drop buffers at end of program
    pub fn new() -> Self {
        let mut shoreline_builder = map_utils::GLShorelineBuilder::new();
        let (vertices, rings) = shoreline_builder.draw_shoreline();

        let mut shoreline_vertex_buffer: GLuint = 0;

        unsafe {
            gl::GenBuffers(1, &mut shoreline_vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, shoreline_vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER, // target
                (vertices.len() * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr, // size of data in bytes
                vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        ShorelineRenderer {
            shoreline_vertex_buffer,
            shoreline_rings: rings,
        }
    }

    pub fn draw(&self, _area: &GLArea) {

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.shoreline_vertex_buffer); //Bind GL_ARRAY_BUFFER to our handle

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (3 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null(), // offset of the first component
            );

            for ring in &self.shoreline_rings {
                gl::DrawArrays(gl::LINE_STRIP, ring.0 as GLint, ring.1 as GLint);
            }
            gl::BindBuffer(gl::ARRAY_BUFFER,0 ); //Bind GL_ARRAY_BUFFER to our handle

        }
    }

    pub fn drop_buffers(&self) {
        unsafe {
            gl::DisableVertexAttribArray(0);
            gl::DeleteBuffers(1, &self.shoreline_vertex_buffer.clone());
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);  // Vertex buffer
        }
    }
}
