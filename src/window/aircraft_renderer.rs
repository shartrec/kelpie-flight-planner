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

use std::cell::{Cell, RefCell};
use std::ops::Deref;
use gl::types::{GLint, GLuint};
use gtk::GLArea;

use crate::earth::spherical_projector::SphericalProjector;
use crate::earth::coordinate::Coordinate;
use crate::util::fg_link::AircraftPositionInfo;
use crate::window::map_utils::Vertex;

pub struct AircraftRenderer {
    aircraft_vertex_array: GLuint,
    aircraft_vertex_buffer: GLuint,
    aircraft_position: RefCell<Option<AircraftPositionInfo>>,
    zoom_level: Cell<f32>,

}

impl AircraftRenderer {
    pub fn new() -> Self {
        let mut aircraft_vertex_buffer: GLuint = 0;
        let mut aircraft_vertex_array: GLuint = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut aircraft_vertex_array);
            gl::BindVertexArray(aircraft_vertex_array);

            gl::GenBuffers(1, &mut aircraft_vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, aircraft_vertex_buffer);

        }
        AircraftRenderer {
            aircraft_vertex_array,
            aircraft_vertex_buffer,
            aircraft_position: RefCell::new(None),
            zoom_level: Cell::new(1.0),
        }
    }

    pub fn set_aircraft_position(&self, aircraft_position: Option<AircraftPositionInfo>) {
        self.aircraft_position.replace(aircraft_position);
        self.load_buffers(&self.aircraft_position, self.aircraft_vertex_buffer);
    }
    pub fn set_zoom_level(&self, zoom: f32) {
        self.zoom_level.replace(zoom);
        self.load_buffers(&self.aircraft_position, self.aircraft_vertex_buffer);
    }

    fn load_buffers(&self, aircraft_position: &RefCell<Option<AircraftPositionInfo>>, _aircraft_vertex_buffer: GLuint) {
        if let Some(api) = aircraft_position.borrow().deref() {
            let vertices = self.build_aircraft_vertices(api);
            unsafe {
                gl::BindVertexArray(self.aircraft_vertex_array);
                gl::VertexAttribPointer(
                    0, // index of the generic vertex attribute ("layout (location = 0)")
                    3, // the number of components per generic vertex attribute
                    gl::FLOAT, // data type
                    gl::FALSE, // normalized (int-to-float conversion)
                    (3 * size_of::<f32>()) as GLint, // stride (byte offset between consecutive attributes)
                    std::ptr::null(), // offset of the first component
                );

                gl::BindBuffer(gl::ARRAY_BUFFER, self.aircraft_vertex_buffer);
                gl::BufferData(
                    gl::ARRAY_BUFFER, // target
                    (vertices.len() * size_of::<Vertex>()) as gl::types::GLsizeiptr, // size of data in bytes
                    vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                    gl::DYNAMIC_DRAW, // usage
                );

                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                gl::BindVertexArray(0);
            }
        }
    }

    pub fn draw(&self, _area: &GLArea) {
        if self.aircraft_position.borrow().is_some() {
            unsafe {
                gl::EnableVertexAttribArray(0);

                gl::BindVertexArray(self.aircraft_vertex_array);

                gl::BindBuffer(gl::ARRAY_BUFFER, self.aircraft_vertex_buffer);

                gl::VertexAttribPointer(
                    0, // index of the generic vertex attribute ("layout (location = 0)")
                    3, // the number of components per generic vertex attribute
                    gl::FLOAT, // data type
                    gl::FALSE, // normalized (int-to-float conversion)
                    (3 * size_of::<f32>()) as GLint, // stride (byte offset between consecutive attributes)
                    std::ptr::null(), // offset of the first component
                );

                gl::DrawArrays(gl::TRIANGLES, 0 as GLint, 18 as GLint);
            }
        }
    }

    pub fn drop_buffers(&self) {
        unsafe {
            gl::DeleteBuffers(1, &self.aircraft_vertex_buffer);
            gl::DeleteVertexArrays(1, &self.aircraft_vertex_array);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);  // Vertex buffer
            gl::BindVertexArray(0);
        }
    }

    fn build_aircraft_vertices(&self, api: &AircraftPositionInfo) -> Vec<Vertex> {
        let projector = SphericalProjector::new(1.000);

        let mut vertices: Vec<Vertex> = Vec::new();

        let aircraft_position = api.get_position();

        let heading = api.get_heading();

        let line_length = 200.0 / self.zoom_level.get() as f64;

        // Define the length and width of the aircraft
        let fuselage_length = line_length * 1.0;
        let fuselage_width = line_length * 0.2;
        let wing_length = line_length * 0.9;
        let wing_width = line_length * 0.3;
        let tail_length = line_length * 0.5;
        let tail_width = line_length * 0.15;

        // Calculate positions of key points based on aircraft heading
        let nose_r = aircraft_position.coordinate_at(fuselage_length / 2.0, heading).coordinate_at(fuselage_width / 2.0, (heading + 90.0) % 360.0);
        let nose_l = aircraft_position.coordinate_at(fuselage_length / 2.0, heading).coordinate_at(fuselage_width / 2.0, (heading + 270.0) % 360.0);
        let tail_r = aircraft_position.coordinate_at(fuselage_length / 2.0, heading + 180.0).coordinate_at(fuselage_width / 5.0, (heading + 90.0) % 360.0);
        let tail_l = aircraft_position.coordinate_at(fuselage_length / 2.0, heading + 180.0).coordinate_at(fuselage_width / 5.0, (heading + 270.0) % 360.0);
        let left_wing_r = aircraft_position.coordinate_at(wing_length / 2.0, (heading + 270.0) % 360.0);
        let left_wing_f = aircraft_position.coordinate_at(wing_width, heading);
        let right_wing_r = aircraft_position.coordinate_at(wing_length / 2.0, (heading + 90.0) % 360.0);
        let right_wing_f = aircraft_position.coordinate_at(wing_width, heading);
        let left_tail_r = aircraft_position.coordinate_at(fuselage_length / 2.0, (heading + 180.0) % 360.0).coordinate_at(tail_length / 2.0, (heading + 270.0) % 360.0);
        let left_tail_f = left_tail_r.coordinate_at(tail_width, heading);
        let right_tail_f = left_tail_f.coordinate_at(tail_length, (heading + 90.0) % 360.0);
        let right_tail_r = left_tail_r.coordinate_at(tail_length, (heading + 90.0) % 360.0);

        // Fuselage
        let f = [nose_r, nose_l, tail_l, tail_r];
        push_vertices(&mut vertices, &f, &projector);

        // Wings
        let wings = [left_wing_f, left_wing_r, right_wing_r, right_wing_f];
        push_vertices(&mut vertices, &wings, &projector);

        // Tail
        let tail = [left_tail_f, left_tail_r, right_tail_r, right_tail_f];
        push_vertices(&mut vertices, &tail, &projector);

        vertices
    }
}

fn push_vertices(vertices: &mut Vec<Vertex>, quad: &[Coordinate; 4], projector: &SphericalProjector) {
    // Project each coordinate and push as triangles
    let projected: Vec<[f32; 3]> = quad.iter()
        .map(|c| projector.project(c.get_latitude(), c.get_longitude()))
        .collect();

    vertices.push(Vertex { position: projected[0] });
    vertices.push(Vertex { position: projected[1] });
    vertices.push(Vertex { position: projected[2] });
    vertices.push(Vertex { position: projected[2] });
    vertices.push(Vertex { position: projected[3] });
    vertices.push(Vertex { position: projected[0] });
}
