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

use crate::window::map_utils;
use crate::window::map_utils::Vertex2;
use adw::gdk::gdk_pixbuf::Pixbuf;
use gl::types::GLuint;
use gtk::GLArea;
use log::info;
use std::ffi::c_void;

pub struct SphereRenderer {
    sphere_vertex_buffer: GLuint,
    sphere_index_buffer: GLuint,
    sphere_vertex_array: GLuint,
    sphere_triangles: usize,
    texture: u32,
}

impl SphereRenderer {
    pub fn new() -> Self {
        let mut sphere_builder = map_utils::GLSphereBuilder::new();
        let (vertices, indices) = sphere_builder.draw_sphere(1.0);

        let mut sphere_vertex_buffer: GLuint = 0;
        let mut sphere_vertex_array: GLuint = 0;
        let mut sphere_index_buffer: GLuint = 0;
        let mut texture = 0;

        let img = Pixbuf::from_resource("/com/shartrec/kelpie_planner/images/world.200406.3x5400x2700.jpg").expect("Failed to load texture image");

        // Load texture image
        let tex_width = img.width();
        let tex_height = img.height();

        // let vertices = map_texture_to_sphere(vertices);

            unsafe {
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture); // all upcoming GL_TEXTURE_2D operations now have effect on this texture object
            // gl::Uniform1i(uTexture, 0);
            // set the texture wrapping parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32); // set texture wrapping to gl::REPEAT (default wrapping method)
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            // set texture filtering parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            // load image, create texture and generate mipmaps

            let data = img.pixels();
            gl::TexImage2D(gl::TEXTURE_2D,
                           0,
                           gl::RGB as i32,
                           tex_width as i32,
                           tex_height as i32,
                           0,
                           gl::RGB,
                           gl::UNSIGNED_BYTE,
                           &data[0] as *const u8 as *const c_void);

            gl::GenerateMipmap(gl::TEXTURE_2D);

                info!("Sphere vertices: {}", vertices.len());

            gl::GenVertexArrays(1, &mut sphere_vertex_array);
            gl::BindVertexArray(sphere_vertex_array);

            gl::GenBuffers(1, &mut sphere_vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, sphere_vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER, // target
                (vertices.len() * size_of::<Vertex2>()) as gl::types::GLsizeiptr, // size of data in bytes
                vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::GenBuffers(1, &mut sphere_index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, sphere_index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * size_of::<u32>()) as gl::types::GLsizeiptr,
                indices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl::VertexAttribPointer(
                0, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (5 * size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null(), // offset of the first component
            );
            gl::EnableVertexAttribArray(1); // this is "layout (location = 1)" in vertex shader
            gl::VertexAttribPointer(
                1, // index of the generic vertex attribute ("layout (location = 1)")
                2, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (5 * size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                (3 * size_of::<f32>()) as *const c_void, // offset of the first component
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        SphereRenderer {
            sphere_vertex_buffer,
            sphere_index_buffer,
            sphere_vertex_array,
            sphere_triangles: indices.len(),
            texture,
        }
    }

    pub fn draw(&self, _area: &GLArea) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture); // all upcoming GL_TEXTURE_2D operations now have effect on this texture object

            gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl::BindVertexArray(self.sphere_vertex_array);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.sphere_vertex_buffer); //Bind GL_ARRAY_BUFFER to our handle


            gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl::VertexAttribPointer(
                0, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (5 * size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null(), // offset of the first component
            );
            gl::EnableVertexAttribArray(1); // this is "layout (location = 1)" in vertex shader
            gl::VertexAttribPointer(
                1, // index of the generic vertex attribute ("layout (location = 1)")
                2, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (5 * size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                (3 * size_of::<f32>()) as *const c_void, // offset of the first component
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.sphere_index_buffer);

            gl::DrawElements(
                gl::TRIANGLES, // mode
                self.sphere_triangles as gl::types::GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
    }

    pub fn drop_buffers(&self) {
        unsafe {
            gl::DeleteBuffers(1, &self.sphere_vertex_buffer.clone());
            gl::DeleteBuffers(1, &self.sphere_index_buffer.clone());
            gl::DeleteVertexArrays(1, &self.sphere_vertex_array.clone());
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);  // Vertex buffer
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);  // Index buffer
            gl::BindVertexArray(0);  // Index buffer
            gl::BindTexture(gl::TEXTURE_2D, 0);  // Index buffer
        }
    }
}

