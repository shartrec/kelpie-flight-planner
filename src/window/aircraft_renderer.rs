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

    fn load_buffers(&self, aircraft_position: &RefCell<Option<AircraftPositionInfo>>, aircraft_vertex_buffer: GLuint) {
        if let Some(api) = aircraft_position.borrow().deref() {
            let vertices = self.build_aircraft_vertices(api);
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, aircraft_vertex_buffer);
                gl::BufferData(
                    gl::ARRAY_BUFFER, // target
                    (vertices.len() * size_of::<Vertex>()) as gl::types::GLsizeiptr, // size of data in bytes
                    vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                    gl::DYNAMIC_DRAW, // usage
                );

                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            }
        }
    }

    pub fn draw(&self, _area: &GLArea) {
        if self.aircraft_position.borrow().is_some() {
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.aircraft_vertex_buffer);
                gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
                gl::VertexAttribPointer(
                    0, // index of the generic vertex attribute ("layout (location = 0)")
                    3, // the number of components per generic vertex attribute
                    gl::FLOAT, // data type
                    gl::FALSE, // normalized (int-to-float conversion)
                    (3 * size_of::<f32>()) as GLint, // stride (byte offset between consecutive attributes)
                    std::ptr::null(), // offset of the first component
                );

                // Draw the fuselage
                gl::DrawArrays(gl::TRIANGLES, 0 as GLint, 6 as GLint);
                // Draw the wings
                gl::DrawArrays(gl::TRIANGLES, 6 as GLint, 6 as GLint);
                // Draw the tail
                gl::DrawArrays(gl::TRIANGLES, 12 as GLint, 6 as GLint);

                gl::BindBuffer(gl::ARRAY_BUFFER, 0); //Bind GL_ARRAY_BUFFER to our handle
                gl::DisableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            }
        }
    }

    pub fn drop_buffers(&self) {
        unsafe {
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

        let line_length = 200.0 / self.zoom_level.get() as f64;

        // Define the length and width of the aircraft
        let fuselage_length = line_length * 1.0; // Full length of the fuselage
        let fuselage_width = line_length * 0.2; // Width of the fuselage
        let wing_length = line_length * 0.9; // Length of the wings
        let wing_width = line_length * 0.3; // wing width
        let tail_length = line_length * 0.5; // Length of the tail section
        let tail_width = line_length * 0.15; // Width of the tail section

// Calculate positions of key points based on aircraft heading
        let nose_r = aircraft_position.coordinate_at(fuselage_length / 2.0, heading).coordinate_at(fuselage_width / 2.0, (heading + 90.0) % 360.0);
        let nose_l = aircraft_position.coordinate_at(fuselage_length / 2.0, heading).coordinate_at(fuselage_width / 2.0, (heading + 270.0) % 360.0);
        let tail_r = aircraft_position.coordinate_at(fuselage_length / 2.0, heading + 180.0).coordinate_at(fuselage_width / 5.0, (heading + 90.0) % 360.0);
        let tail_l = aircraft_position.coordinate_at(fuselage_length / 2.0, heading + 180.0).coordinate_at(fuselage_width / 5.0, (heading + 270.0) % 360.0);
        let left_wing_r = aircraft_position.coordinate_at(wing_length / 2.0, (heading + 270.0) % 360.0);
        let left_wing_f = aircraft_position.coordinate_at(wing_width, (heading) % 360.0);
        let right_wing_r = aircraft_position.coordinate_at(wing_length / 2.0, (heading + 90.0) % 360.0);
        let right_wing_f = aircraft_position.coordinate_at(wing_width, (heading) % 360.0);
        let left_tail_r = aircraft_position.coordinate_at(fuselage_length / 2.0, (heading + 180.0) % 360.0).coordinate_at(tail_length / 2.0, (heading + 270.0) % 360.0);
        let left_tail_f = left_tail_r.coordinate_at(tail_width, heading);
        let right_tail_f = left_tail_f.coordinate_at(tail_length, (heading + 90.0) % 360.0);
        let right_tail_r = left_tail_r.coordinate_at(tail_length, (heading + 90.0) % 360.0);

// Project the coordinates to 2D map space
        let nose_projected_l = projector.project(nose_l.get_latitude(), nose_l.get_longitude());
        let nose_projected_r = projector.project(nose_r.get_latitude(), nose_r.get_longitude());
        let tail_projected_l = projector.project(tail_l.get_latitude(), tail_l.get_longitude());
        let tail_projected_r = projector.project(tail_r.get_latitude(), tail_r.get_longitude());
        let left_wing_projected_f = projector.project(left_wing_f.get_latitude(), left_wing_f.get_longitude());
        let left_wing_projected_r = projector.project(left_wing_r.get_latitude(), left_wing_r.get_longitude());
        let right_wing_projected_f = projector.project(right_wing_f.get_latitude(), right_wing_f.get_longitude());
        let right_wing_projected_r = projector.project(right_wing_r.get_latitude(), right_wing_r.get_longitude());
        let left_tail_projected_f = projector.project(left_tail_f.get_latitude(), left_tail_f.get_longitude());
        let left_tail_projected_r = projector.project(left_tail_r.get_latitude(), left_tail_r.get_longitude());
        let right_tail_projected_f = projector.project(right_tail_f.get_latitude(), right_tail_f.get_longitude());
        let right_tail_projected_r = projector.project(right_tail_r.get_latitude(), right_tail_r.get_longitude());

        // Fuselage (two triangles)
        vertices.push(Vertex { position: nose_projected_l });
        vertices.push(Vertex { position: nose_projected_r });
        vertices.push(Vertex { position: tail_projected_r });
        vertices.push(Vertex { position: tail_projected_r });
        vertices.push(Vertex { position: tail_projected_l });
        vertices.push(Vertex { position: nose_projected_l });

        // Wings (two triangles)
        vertices.push(Vertex { position: left_wing_projected_r });
        vertices.push(Vertex { position: right_wing_projected_r });
        vertices.push(Vertex { position: right_wing_projected_f });
        vertices.push(Vertex { position: right_wing_projected_f });
        vertices.push(Vertex { position: left_wing_projected_f });
        vertices.push(Vertex { position: left_wing_projected_r });

        // Tail (two triangles)
        vertices.push(Vertex { position: left_tail_projected_r });
        vertices.push(Vertex { position: right_tail_projected_r });
        vertices.push(Vertex { position: right_tail_projected_f });
        vertices.push(Vertex { position: right_tail_projected_f });
        vertices.push(Vertex { position: left_tail_projected_f });
        vertices.push(Vertex { position: left_tail_projected_r });

        vertices
    }
}
