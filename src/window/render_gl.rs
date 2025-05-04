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
extern crate nalgebra_glm as glm;

use std::cell::{Cell, RefCell};
use std::ffi::{CStr, CString};
use std::rc::Rc;
use std::time::Duration;

use adw::glib::timeout_add_local_once;
use glm::*;
use gtk::GLArea;
use adw::prelude::WidgetExt;
use crate::earth::coordinate::Coordinate;
use crate::earth::spherical_projector::SphericalProjector;
use crate::model::plan::Plan;
use crate::util::fg_link::AircraftPositionInfo;
use crate::window::aircraft_renderer::AircraftRenderer;
use crate::window::airport_renderer::AirportRenderer;
use crate::window::navaid_renderer::NavaidRenderer;
use crate::window::plan_renderer::PlanRenderer;
use crate::window::shoreline_renderer::ShorelineRenderer;
use crate::window::sphere_renderer::SphereRenderer;

pub struct Program {
    id: gl::types::GLuint,
}

impl Program {
    pub fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl::CreateProgram() };

        for shader in shaders {
            unsafe { gl::AttachShader(program_id, shader.id); }
        }

        unsafe { gl::LinkProgram(program_id); }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl::GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe { gl::DetachShader(program_id, shader.id); }
        }

        Ok(Program { id: program_id })
    }

    pub fn gl_use(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

#[derive(Clone)]
pub struct Shader {
    id: gl::types::GLuint,
}

impl Shader {
    pub fn from_source(
        source: &CStr,
        kind: gl::types::GLenum,
    ) -> Result<Shader, String> {
        let id = shader_from_source(source, kind)?;
        Ok(Shader { id })
    }

    pub fn from_vert_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::VERTEX_SHADER)
    }

    pub fn from_frag_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::FRAGMENT_SHADER)
    }
}

fn shader_from_source(
    source: &CStr,
    kind: gl::types::GLenum,
) -> Result<gl::types::GLuint, String> {
    let id = unsafe { gl::CreateShader(kind) };
    unsafe {
        gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = create_whitespace_cstring_with_len(len as usize);

        unsafe {
            gl::GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar,
            );
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill it with len spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert buffer to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}

pub struct Renderer {
    shader_program: Program,
    shadow_program: Program,
    sphere_renderer: SphereRenderer,
    world_renderer: ShorelineRenderer,
    lake_renderer: ShorelineRenderer,
    island_renderer: ShorelineRenderer,
    antarctic_renderer: ShorelineRenderer,
    airport_renderer: RefCell<AirportRenderer>,
    navaid_renderer: RefCell<NavaidRenderer>,
    plan_renderer: RefCell<Option<PlanRenderer>>,
    aircraft_renderer: RefCell<AircraftRenderer>,

    zoom_level: Cell<f32>,
    sun_direction: Cell<[f32; 3]>,
    map_centre: RefCell<Coordinate>,
    last_map_centre: RefCell<Coordinate>,
}

impl Renderer {
    pub fn new() -> Self {
        let vert_shader = Shader::from_vert_source(
            &CString::new(include_str!("program.vert")).unwrap()
        ).unwrap();

        let frag_shader = Shader::from_frag_source(
            &CString::new(include_str!("program.frag")).unwrap()
        ).unwrap();

        let shader_program = Program::from_shaders(
            &[vert_shader.clone(), frag_shader]
        ).unwrap();

        let frag_shader = Shader::from_frag_source(
            &CString::new(include_str!("shadow_program.frag")).unwrap()
        ).unwrap();

        let shadow_program = Program::from_shaders(
            &[vert_shader, frag_shader]
        ).unwrap();

        let sphere_renderer = SphereRenderer::new();
        let world_renderer = ShorelineRenderer::new("GSHHS_l_L1.shp");
        let lake_renderer = ShorelineRenderer::new("GSHHS_l_L2.shp");
        let island_renderer = ShorelineRenderer::new("GSHHS_l_L3.shp");
        let antarctic_renderer = ShorelineRenderer::new("GSHHS_l_L6.shp");
        let airport_renderer = AirportRenderer::new();
        let navaid_renderer = NavaidRenderer::new();
        let aircraft_renderer = AircraftRenderer::new();

        Renderer {
            shader_program,
            shadow_program,
            sphere_renderer,
            world_renderer,
            lake_renderer,
            island_renderer,
            antarctic_renderer,
            airport_renderer: RefCell::new(airport_renderer),
            navaid_renderer: RefCell::new(navaid_renderer),
            plan_renderer: RefCell::new(None),
            aircraft_renderer: RefCell::new(aircraft_renderer),
            zoom_level: Cell::new(1.0),
            sun_direction: Cell::new([0.5, 0.5, 0.0]),
            map_centre: RefCell::new(Coordinate::new(0.0, 0.0)),
            last_map_centre: RefCell::new(Coordinate::new(0.0, 0.0)),
        }
    }

    pub fn airports_loaded(&self) {
        self.airport_renderer.replace(AirportRenderer::new());
    }
    pub fn navaids_loaded(&self) {
        self.navaid_renderer.replace(NavaidRenderer::new());
    }

    pub fn set_plan(&self, plan: Rc<RefCell<Plan>>) {
        if let Some(old_pr) = self.plan_renderer.replace(Some(PlanRenderer::new(plan))) {
            old_pr.drop_buffers();
        }
    }

    pub fn plan_changed(&self) {
        if let Some(plan_renderer) = self.plan_renderer.borrow().as_ref() {
            plan_renderer.plan_changed();
        }
    }

    pub fn set_zoom_level(&self, zoom: f32) {
        self.zoom_level.replace(zoom);
        self.aircraft_renderer.borrow().set_zoom_level(zoom);
    }

    pub fn get_map_centre(&self) -> Coordinate {
        self.map_centre.borrow().clone()
    }

    pub fn set_map_centre(&self, centre: Coordinate, fast: bool) {
        self.map_centre.replace(centre.clone());
        if fast {
            self.last_map_centre.replace(centre.clone());
        }
    }

    pub fn set_aircraft_position(&self, aircraft_position: Option<AircraftPositionInfo>) {
        self.aircraft_renderer.borrow().set_aircraft_position(aircraft_position);
    }

    pub fn set_sub_solar_point(&self, sub_solar_point: (f64, f64)) {
        let projector = SphericalProjector::new(1.0);
        let sun_direction = projector.project(sub_solar_point.0, sub_solar_point.1);
        self.sun_direction.replace(sun_direction);
    }

    pub fn draw(&self, area: &GLArea, with_airports: bool, with_navaids: bool) {
        unsafe {
            gl::Enable(gl::POINT_SIZE);
            // Enable line smoothing - Not actually supported under GTK
            gl::Enable(gl::LINE_SMOOTH);
            // Disable depth testing
            // * We don't need this as we push the far side of Earth outside the clipping ares
            // * We don't want it as without we can draw everything in the same surface as
            //   long as we do it in the correct order.
            gl::Disable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::DepthMask(gl::FALSE);

            gl::ClearColor(0.15, 0.095, 0.155, 1.);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let width = area.width();
        let height = area.height();

        let aspect_ratio = if height < width {
            [height as f32 / width as f32, 1.0]
        } else {
            [1.0, width as f32 / height as f32]
        };
        let zoom = self.zoom_level.get();

        let true_centre = self.increment_to_centre();
        let trans = self.build_matrix(aspect_ratio, zoom);

        self.shadow_program.gl_use();

        let point_size = 1.0f32;
        unsafe {
            let c = gl::GetUniformLocation(self.shadow_program.id(), b"pointSize\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniform1f(self.shadow_program.id(), c, point_size);

            let mat = gl::GetUniformLocation(self.shadow_program.id(), b"matrix\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniformMatrix4fv(self.shadow_program.id(), mat, 1, false as gl::types::GLboolean, trans.as_ptr() as *const gl::types::GLfloat);

            let c = gl::GetUniformLocation(self.shadow_program.id(), b"sun_direction\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniform3fv(self.shadow_program.id(), c, 1, self.sun_direction.as_ptr() as *const gl::types::GLfloat);

            let shadow_strength = 0.25f32;
            let c = gl::GetUniformLocation(self.shadow_program.id(), b"shadow_strength\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniform1f(self.shadow_program.id(), c, shadow_strength);
        }

        let color = [0.00, 0.5, 1.0f32];
        unsafe {
            let c = gl::GetUniformLocation(self.shadow_program.id(), b"color\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniform3fv(self.shadow_program.id(), c, 1, color.as_ptr() as *const gl::types::GLfloat);
        }
        self.sphere_renderer.draw(area);

        let color = [0.652, 0.697, 0.138f32];
        unsafe {
            let c = gl::GetUniformLocation(self.shadow_program.id(), b"color\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniform3fv(self.shadow_program.id(), c, 1, color.as_ptr() as *const gl::types::GLfloat);
        }
        self.world_renderer.draw(area);

        let color = [0.3, 0.65, 1.0f32];
        unsafe {
            let c = gl::GetUniformLocation(self.shadow_program.id(), b"color\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniform3fv(self.shadow_program.id(), c, 1, color.as_ptr() as *const gl::types::GLfloat);
        }
        self.lake_renderer.draw(area);

        let color = [0.652, 0.697, 0.138f32];
        unsafe {
            let c = gl::GetUniformLocation(self.shadow_program.id(), b"color\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniform3fv(self.shadow_program.id(), c, 1, color.as_ptr() as *const gl::types::GLfloat);
        }
        self.island_renderer.draw(area);

        let color = [0.95, 0.95, 0.95f32];
        unsafe {
            let c = gl::GetUniformLocation(self.shadow_program.id(), b"color\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniform3fv(self.shadow_program.id(), c, 1, color.as_ptr() as *const gl::types::GLfloat);
        }
        self.antarctic_renderer.draw(area);

        self.shader_program.gl_use();

        unsafe {
            let c = gl::GetUniformLocation(self.shader_program.id(), b"pointSize\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniform1f(self.shader_program.id(), c, point_size);

            let mat = gl::GetUniformLocation(self.shader_program.id(), b"matrix\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniformMatrix4fv(self.shader_program.id(), mat, 1, false as gl::types::GLboolean, trans.as_ptr() as *const gl::types::GLfloat);
        }

        if with_navaids {
            let color = [0.2, 0.2, 1.0f32];
            unsafe {
                let c = gl::GetUniformLocation(self.shader_program.id(), b"color\0".as_ptr() as *const gl::types::GLchar);
                gl::ProgramUniform3fv(self.shader_program.id(), c, 1, color.as_ptr() as *const gl::types::GLfloat);
            }
            self.navaid_renderer.borrow().draw(area, zoom > 3.0, self.shader_program.id());
        }

        if with_airports {
            let color = [0.64, 0.0, 0.0f32];
            unsafe {
                let c = gl::GetUniformLocation(self.shader_program.id(), b"color\0".as_ptr() as *const gl::types::GLchar);
                gl::ProgramUniform3fv(self.shader_program.id(), c, 1, color.as_ptr() as *const gl::types::GLfloat);
            }
            self.airport_renderer.borrow().draw(area, zoom > 3.0, zoom > 6.0, self.shader_program.id());
        }

        if let Some(plan_renderer) = self.plan_renderer.borrow().as_ref() {
            let color = [0.0, 0.0, 0.0f32];
            unsafe {
                let c = gl::GetUniformLocation(self.shader_program.id(), b"color\0".as_ptr() as *const gl::types::GLchar);
                gl::ProgramUniform3fv(self.shader_program.id(), c, 1, color.as_ptr() as *const gl::types::GLfloat);
            }
            plan_renderer.draw(area, self.shader_program.id());
        }

        let color = [1.0, 0.1, 0.1f32];
        unsafe {
            let c = gl::GetUniformLocation(self.shader_program.id(), b"color\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniform3fv(self.shader_program.id(), c, 1, color.as_ptr() as *const gl::types::GLfloat);
        }
        self.aircraft_renderer.borrow().draw(area);


        if !true_centre {
            let my_area = area.clone();
            timeout_add_local_once(Duration::from_millis(20), move || {
                my_area.queue_draw();
            });
        }
    }

    fn build_matrix(&self, aspect_ratio: [f32; 2], zoom: f32) -> TMat4<f32> {
        // Create the model matrix (handling scaling, rotation, and translation)
        let mut model = mat4(1.0, 0.0, 0.0, 0.0,
                             0.0, 1.0, 0.0, 0.0,
                             0.0, 0.0, 1.0, 0.0,
                             0.0, 0.0, 0.0, 1.0);

        // The translation is not strictly necessary, but as all the points with z > 0.0
        // are on the far side of the earth, this pushes the model back and the points can
        // be ignored by the renderer.
        model = translate(&model, &vec3(0., 0., 1.0));
        model = scale(&model, &vec3(aspect_ratio[0], aspect_ratio[1], 1.0));
        model = scale(&model, &vec3(zoom, zoom, 1.0));
        model = rotate(&model, -self.last_map_centre.borrow().get_latitude().to_radians() as f32, &vec3(1.0, 0.0, 0.0));
        model = rotate(&model, self.last_map_centre.borrow().get_longitude().to_radians() as f32, &vec3(0.0, 1.0, 0.0));
        model
    }

    pub fn drop_buffers(&self) {
        unsafe {
            gl::DeleteProgram(self.shader_program.id);
            gl::DeleteProgram(self.shadow_program.id);
        }
        self.sphere_renderer.drop_buffers();
        self.world_renderer.drop_buffers();
        self.lake_renderer.drop_buffers();
        self.lake_renderer.drop_buffers();
        self.antarctic_renderer.drop_buffers();
        self.airport_renderer.borrow().drop_buffers();
        self.navaid_renderer.borrow().drop_buffers();
        if let Some(plan_renderer) = self.plan_renderer.borrow().as_ref() {
            plan_renderer.drop_buffers();
        }
        self.aircraft_renderer.borrow().drop_buffers();
    }
    pub fn get_cord_from_win(&self, area: &GLArea, x: f32, y: f32, zoom: f32) -> Result<[f32; 2], String> {
        // We need to calculate the Z depth where the point meets the earth
        // Get the earth radius
        let height = area.height() as f32;
        let width = area.width() as f32;
        let side = width.min(height);
        let earth_radius = side * zoom / 2.0;

        // Get the true projected x, y coordinates
        let x1 = (width / 2.0) - x;
        let y1 = (height / 2.0) - y;

        let r_squared = x1 * x1 + y1 * y1;
        let earth_r_squared = earth_radius.powi(2);
        // Ensure within earth radius
        if r_squared.sqrt() > earth_r_squared {
            return Err("not in map".to_string());
        }
        let z = earth_radius - (earth_r_squared - r_squared).sqrt();
        // This is the Z-depth of the clicked point.
        let normal_x = -x1 / (width / 2.0);  // using width brings in the aspect ratio
        let normal_y = y1 / (height / 2.0); // using height brings in the aspect ratio
        let normal_z = z / earth_radius;

        // Now we need to transform this into model coordinates.
        // get_matrix_and_unwind();
        let aspect_ratio = if height < width {
            [height / width, 1.0]
        } else {
            [1.0, width / height]
        };
        let mat = self.build_matrix(aspect_ratio, zoom);

        match mat.try_inverse() {
            Some(transform) => {
                let pt = TVec4::new(
                    normal_x,
                    normal_y,
                    normal_z,
                    1.,
                );
                let result = transform * pt;
                let vertex = result.fixed_rows::<3>(0) / result.w;
                let projector = SphericalProjector::new(1.0);
                projector.un_project(vertex.x, vertex.y, vertex.z)
            }
            None => {
                Err("not in map".to_string())
            }
        }
    }
    fn increment_to_centre(&self) -> bool {
        // This updates the last_centre position which is where we actually draw
        // and returns true if we have reached the true centre as requested

        let req_lat = self.map_centre.borrow().get_latitude();
        let mut last_lat = self.last_map_centre.borrow().get_latitude();
        let mut r_lat_inc = (req_lat - last_lat) / 20.0;

        let req_long = self.map_centre.borrow().get_longitude();
        let mut last_long = self.last_map_centre.borrow().get_longitude();

        let mut r_long_inc = req_long - last_long;
        if r_long_inc < -180.0 {
            r_long_inc += 360.0;
        }
        if r_long_inc > 180.0 {
            r_long_inc -= 360.0;
        }
        r_long_inc /= 20.0;

        if r_lat_inc.abs() < 0.001 && r_long_inc.abs() < 0.001 {
            true
        } else {
            if r_lat_inc.abs() < 2. && r_long_inc.abs() < 2. {
                let max_inc = r_lat_inc.abs().max(r_long_inc.abs());
                let rescale = 2. / max_inc;
                r_lat_inc *= rescale;
                r_long_inc *= rescale;
            }
            let lat_inc = r_lat_inc * self.zoom_level.get().sqrt() as f64;
            let long_inc = r_long_inc * self.zoom_level.get().sqrt() as f64;

            if (req_lat - last_lat).abs() > lat_inc.abs() {
                last_lat += lat_inc;
            } else {
                last_lat = req_lat;
            }
            if (req_long - last_long).abs() > long_inc.abs() {
                last_long += long_inc;
            } else {
                last_long = req_long;
            }
            if last_long < -180.0 {
                last_long += 360.0;
            }
            if last_long > 180.0 {
                last_long -= 360.0;
            }
            self.last_map_centre.replace(Coordinate::new(last_lat, last_long));
            false
        }
    }
}
