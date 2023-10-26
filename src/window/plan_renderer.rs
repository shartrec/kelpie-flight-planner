/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */

use std::f32::consts::PI;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

use gl::types::{GLint, GLuint};
use gtk::GLArea;

use crate::earth::spherical_projector::SphericalProjector;
use crate::model::plan::Plan;
use crate::window::map_utils::Vertex;

pub struct PlanRenderer {
    plan_vertex_buffer: GLuint,
    plan_index_buffer: GLuint,
    waypoints: usize,
}

impl PlanRenderer {
    //todo drop buffers at end of program
    pub fn new(plan: Arc<RwLock<Plan>>) -> Self {
        let (vertices, indices) = Self::build_plan_vertices(plan);
        let mut plan_vertex_buffer: GLuint = 0;
        let mut plan_index_buffer: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut plan_vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, plan_vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER, // target
                (vertices.len() * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr, // size of data in bytes
                vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::GenBuffers(1, &mut plan_index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, plan_index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                indices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        PlanRenderer {
            plan_vertex_buffer,
            plan_index_buffer,
            waypoints: vertices.len(),
        }
    }

    pub fn draw(&self, _area: &GLArea) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.plan_vertex_buffer);
            gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl::VertexAttribPointer(
                0, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (3 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null(), // offset of the first component
            );

            let point_size = 4.0;
            gl::PointSize(point_size);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.plan_index_buffer);
            gl::BindVertexArray(self.plan_index_buffer);
            gl::DrawElements(
                gl::POINTS, // mode
                self.waypoints as gl::types::GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );

            gl::DrawArrays(gl::LINE_STRIP, 0 as GLint, (self.waypoints - 1) as GLint);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0); //Bind GL_ARRAY_BUFFER to our handle
            gl::DisableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
        }
    }

    pub fn drop_buffers(&self) {
        unsafe {
            gl::DisableVertexAttribArray(0);
            gl::DeleteBuffers(1, &self.plan_vertex_buffer.clone());
            gl::DeleteBuffers(1, &self.plan_index_buffer.clone());
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);  // Vertex buffer
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);  // Index buffer
            gl::BindVertexArray(0);
        }
    }

    fn build_plan_vertices(plan: Arc<RwLock<Plan>>) -> (Vec<Vertex>, Vec<u32>) {
        let projector = SphericalProjector::new(1.000);

        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices_airports: Vec<u32> = Vec::with_capacity(100);

        // We create a vertex for each waypoint and treat the whole as a line strip,
        // We also add an index for each airport, which is drawn as a point
        let plan = plan.read().expect("Can't get plan lock");
        for s_ref in plan.get_sectors().deref() {
            let binding = s_ref.borrow();
            let s = binding.deref();
            if let Some(airport) = s.get_start() {
                let position = projector.project(airport.get_lat(), airport.get_long());
                vertices.push(Vertex { position: position });
                indices_airports.push(vertices.len() as u32 - 1);
                let mut last_p = position;

                for wp in s
                    .get_waypoints()
                    .read()
                    .expect("Can't get read lock on sectors")
                    .deref()
                {
                    let position = projector.project(wp.get_lat(), wp.get_long());
                    let arc = Self::draw_arc(last_p, position);
                    for v in arc {
                        vertices.push(v);
                    }
                    vertices.push(Vertex { position: position });
                    last_p = position;
                }

                if let Some(airport) = s.get_end() {
                    let position = projector.project(airport.get_lat(), airport.get_long());
                    let arc = Self::draw_arc(last_p, position);
                    for v in arc {
                        vertices.push(v);
                    }
                    vertices.push(Vertex { position: position });
                    indices_airports.push(vertices.len() as u32 - 1);
                }
            }
        }

        (vertices, indices_airports)
    }

    fn draw_arc(from: [f32; 3], to: [f32; 3]) -> Vec<Vertex> {
        // Draw the lines

        let x1 = to[0];
        let y1 = to[1];
        let z1 = to[2];
        let x2 = from[0];
        let y2 = from[1];
        let z2 = from[2];
        // Angle between the 2 points
        let psi = (x1 * x2 + y1 * y2 + z1 * z2).acos();

        // Calculate the coordinates of the point P3 on the great circle that is psi degrees from the first city P1 in the direction of the second city P2
        // x3 = (x2 - x1 cos psi)/sin psi
        // and similarly with y or z in stead of x.

        let x3 = (x2 - x1 * psi.cos()) / psi.sin();
        let y3 = (y2 - y1 * psi.cos()) / psi.sin();
        let z3 = (z2 - z1 * psi.cos()) / psi.sin();

        // Now draw the actual arc
        // The Cartesian coordinates of the points of the great circle are then, as a function of the angular distance phi from the first city:
        // x = x1 cos phi + x3 sin phi
        // and similarly with y or z in stead of x. If phi= 0, then you are in the first city. If phi = psi, then you are in the second city.

        // The plan is drawn slightly above the globe so as to not disappear behind and shore line bits.
        let mut vertices: Vec<Vertex> = Vec::new();

        let inc = PI / 180.0;
        let mut i = 0.0;
        while i < psi {

            let vx = x1 * i.cos() + x3 * i.sin();
            let vy = y1 * i.cos() + y3 * i.sin();
            let vz = z1 * i.cos() + z3 * i.sin();
            vertices.push(Vertex { position: [vx * 1.00001, vy * 1.00001, vz * 1.00001]});
            i += inc
        }
        vertices
    }
}
