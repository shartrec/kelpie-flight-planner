/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */

use std::ffi::CString;

use gl::types::GLuint;
use gtk::GLArea;

use crate::window::{map_utils, render_gl};
use crate::window::map_utils::Vertex;
use crate::window::render_gl::Program;

pub struct SphereRenderer {
    sphere_vertex_buffer: GLuint,
    sphere_vertex_arrays: GLuint,
    sphere_index_buffer: GLuint,
    sphere_triangles: usize,
}

impl SphereRenderer {
    //todo drop buffers at end of program
    pub fn new() -> Self {
        let mut sphere_builder = map_utils::GLSphereBuilder::new();
        let (vertices, indices) = sphere_builder.draw_sphere(0.995);
        let mut sphere_vertex_buffer: GLuint = 0;
        let mut sphere_vertex_arrays: GLuint = 0;
        let mut sphere_index_buffer: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut sphere_vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, sphere_vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER, // target
                (vertices.len() * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr, // size of data in bytes
                vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::GenVertexArrays(1, &mut sphere_vertex_arrays);
            gl::BindVertexArray(sphere_vertex_arrays);

            gl::GenBuffers(1, &mut sphere_index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, sphere_index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len()* std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                indices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

        }


        let sphere_vert_shader = render_gl::Shader::from_vert_source(
            &CString::new(include_str!("program.vert")).unwrap()
        ).unwrap();

        let sphere_frag_shader = render_gl::Shader::from_frag_source(
            &CString::new(include_str!("program.frag")).unwrap()
        ).unwrap();

        let sphere_shader_program = render_gl::Program::from_shaders(
            &[sphere_vert_shader, sphere_frag_shader]
        ).unwrap();

        SphereRenderer {
            sphere_vertex_buffer,
            sphere_vertex_arrays,
            sphere_index_buffer,
            sphere_triangles: indices.len(),
        }
    }

    pub fn draw(&self, area: &GLArea) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.sphere_vertex_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.sphere_index_buffer);
            gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl::VertexAttribPointer(
                0, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (3 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null(), // offset of the first component
            );

            gl::BindVertexArray(self.sphere_index_buffer);
            gl::DrawElements(
                gl::TRIANGLES, // mode
                self.sphere_triangles as gl::types::GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
            gl::DisableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
        }
    }

    pub fn drop_buffers(&self) {
        unsafe {
            gl::DisableVertexAttribArray(0);
            gl::DeleteBuffers(1, &self.sphere_vertex_buffer.clone());
            gl::DeleteBuffers(1, &self.sphere_index_buffer.clone());
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);  // Vertex buffer
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);  // Index buffer
            gl::BindVertexArray(0);
            gl::DeleteVertexArrays(1, &self.sphere_vertex_arrays);
        }
    }
}
