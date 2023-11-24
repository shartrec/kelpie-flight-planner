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

use std::cell::{Cell, RefCell};
use std::ops::Deref;

use gl::types::{GLint, GLuint};
use gtk::GLArea;

use crate::earth::spherical_projector::SphericalProjector;
use crate::util::fg_link::AircraftPositionInfo;
use crate::window::map_utils::Vertex;

pub struct AircraftRenderer {
    aircraft_vertex_buffer: GLuint,
    fg_link_up: Cell<bool>,
    aircraft_position: RefCell<Option<AircraftPositionInfo>>,
    zoom_level: Cell<f32>,

}

impl AircraftRenderer {
    pub fn new() -> Self {
        let mut aircraft_vertex_buffer: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut aircraft_vertex_buffer);
        }

        AircraftRenderer {
            aircraft_vertex_buffer,
            fg_link_up: Cell::new(false),
            aircraft_position: RefCell::new(None),
            zoom_level: Cell::new(1.0),
        }
    }

    pub fn set_aircraft_position(&self, aircraft_position: Option<AircraftPositionInfo>) {
        self.aircraft_position.replace(aircraft_position);
        let fg_link_up = self.load_buffers(&self.aircraft_position, self.aircraft_vertex_buffer);
        self.fg_link_up.replace(fg_link_up);
    }
    pub fn set_zoom_level(&self, zoom: f32) {
        self.zoom_level.replace(zoom);
        let index_count = self.load_buffers(&self.aircraft_position, self.aircraft_vertex_buffer);
        self.fg_link_up.replace(index_count);
    }

    fn load_buffers(&self, aircraft_position: &RefCell<Option<AircraftPositionInfo>>, aircraft_vertex_buffer: GLuint) -> bool {
        if let Some(api) = aircraft_position.borrow().deref() {
            let vertices = self.build_aircraft_vertices(api);
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, aircraft_vertex_buffer);
                gl::BufferData(
                    gl::ARRAY_BUFFER, // target
                    (vertices.len() * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr, // size of data in bytes
                    vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                    gl::DYNAMIC_DRAW, // usage
                );


                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            }
            true
        } else {
            false
        }
    }

    pub fn draw(&self, _area: &GLArea) {
        if self.aircraft_position.borrow().deref().is_some() {
            unsafe {
                gl::LineWidth(1.0);
                gl::BindBuffer(gl::ARRAY_BUFFER, self.aircraft_vertex_buffer);
                gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
                gl::VertexAttribPointer(
                    0, // index of the generic vertex attribute ("layout (location = 0)")
                    3, // the number of components per generic vertex attribute
                    gl::FLOAT, // data type
                    gl::FALSE, // normalized (int-to-float conversion)
                    (3 * std::mem::size_of::<f32>()) as GLint, // stride (byte offset between consecutive attributes)
                    std::ptr::null(), // offset of the first component
                );

                gl::DrawArrays(gl::LINES, 0 as GLint, 2 as GLint);
                gl::DrawArrays(gl::TRIANGLES, 2 as GLint, 3 as GLint);

                gl::LineWidth(1.0);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0); //Bind GL_ARRAY_BUFFER to our handle
                gl::DisableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            }
        }
    }

    pub fn drop_buffers(&self) {
        unsafe {
            gl::DisableVertexAttribArray(0);
            gl::DeleteBuffers(1, &self.aircraft_vertex_buffer.clone());
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);  // Vertex buffer
            gl::BindVertexArray(0);
        }
    }

    fn build_aircraft_vertices(&self, api: &AircraftPositionInfo) -> Vec<Vertex> {
        let projector = SphericalProjector::new(1.000);

        let mut vertices: Vec<Vertex> = Vec::new();

        let aircraft_position = api.get_position();

        let heading = api.get_heading();

        let line_length = 120.0 / self.zoom_level.get() as f64;

        let v_start = aircraft_position.coordinate_at(line_length, heading);
        let v_end = aircraft_position.coordinate_at(line_length, (heading + 180.) % 360.);
        let h_start = aircraft_position.coordinate_at(line_length/2., (heading + 90.) % 360.);
        let h_end = aircraft_position.coordinate_at(line_length/2., (heading + 270.) % 360.);

        let v_s1 = projector.project(v_start.get_latitude(), v_start.get_longitude());
        let v_e1 = projector.project(v_end.get_latitude(), v_end.get_longitude());
        let h_s1 = projector.project(h_start.get_latitude(), h_start.get_longitude());
        let h_e1 = projector.project(h_end.get_latitude(), h_end.get_longitude());

        //Vertices for tail
        vertices.push(Vertex { position: v_s1 });
        vertices.push(Vertex { position: v_e1 });
        // Vertices for wing triangle
        vertices.push(Vertex { position: v_s1 });
        vertices.push(Vertex { position: h_s1 });
        vertices.push(Vertex { position: h_e1 });

        vertices
    }
}
