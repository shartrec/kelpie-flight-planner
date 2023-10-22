/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
extern crate nalgebra_glm as glm;

use std;
use std::cell::RefCell;
use std::ffi::{CStr, CString};

use gl;
use gtk::GLArea;
use gtk::prelude::WidgetExt;
use glm::{*};
use log::error;
use crate::earth::coordinate::Coordinate;
use crate::window::airport_renderer::AirportRenderer;

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

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
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
    sphere_renderer: SphereRenderer,
    shore_line_renderer: ShorelineRenderer,
    airport_renderer: AirportRenderer,

    zoom_level: RefCell<f32>,
    map_centre: RefCell<Coordinate>,
}

impl Renderer {
    //todo drop buffers at end of program
    pub fn new() -> Self {
        let vert_shader = Shader::from_vert_source(
            &CString::new(include_str!("program.vert")).unwrap()
        ).unwrap();

        let frag_shader = Shader::from_frag_source(
            &CString::new(include_str!("program.frag")).unwrap()
        ).unwrap();

        let shader_program = Program::from_shaders(
            &[vert_shader, frag_shader]
        ).unwrap();

        let sphere_renderer = SphereRenderer::new();
        let shore_line_renderer = ShorelineRenderer::new();
        let airport_renderer = AirportRenderer::new();

        Renderer {
            shader_program,
            sphere_renderer,
            shore_line_renderer,
            airport_renderer,
            zoom_level: RefCell::new(1.0),
            map_centre: RefCell::new(Coordinate::new(0.0, 0.0)),
        }
    }

    pub fn set_zoom_level(&self, zoom: f32) {
        self.zoom_level.replace(zoom);
    }

    pub fn get_map_centre(&self) -> Coordinate{
        self.map_centre.borrow().clone()
    }

    pub fn set_map_centre(&self, centre: Coordinate) {
        self.map_centre.replace(centre);
    }


    pub fn draw(&self, area: &GLArea) {

        unsafe {
            gl::Enable(gl::POINT_SIZE);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::LINE_SMOOTH);
            gl::DepthFunc(gl::LESS);
            gl::DepthMask(gl::TRUE);

            gl::ClearColor(0.26, 0.19, 0.31, 1.);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let width = area.width();
        let height = area.height();

        let aspect_ratio = if height < width {
            [height as f32 / width as f32, 1.0]
        } else {
            [1.0, width as f32 / height as f32]
        };

        let trans = self.build_matrix(aspect_ratio);

        self.shader_program.gl_use();

        unsafe {
            let mat = gl::GetUniformLocation(self.shader_program.id(), b"matrix\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniformMatrix4fv(self.shader_program.id(), mat, 1, false as gl::types::GLboolean, trans.as_ptr() as *const  gl::types::GLfloat);
        }

        let color = vec!(0.75, 1.0, 1.0f32);
        unsafe {
            let c = gl::GetUniformLocation(self.shader_program.id(), b"color\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniform3fv(self.shader_program.id(), c, 1, color.as_ptr() as *const gl::types::GLfloat);
        }
        self.sphere_renderer.draw(area);

        let color = vec!(0.3, 0.0, 0.0f32);
        unsafe {
            let c = gl::GetUniformLocation(self.shader_program.id(), b"color\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniform3fv(self.shader_program.id(), c, 1, color.as_ptr() as *const gl::types::GLfloat);
        }
        self.shore_line_renderer.draw(area);

        let color = vec!(1.0, 0.0, 0.0f32);
        unsafe {
            let c = gl::GetUniformLocation(self.shader_program.id(), b"color\0".as_ptr() as *const gl::types::GLchar);
            gl::ProgramUniform3fv(self.shader_program.id(), c, 1, color.as_ptr() as *const gl::types::GLfloat);
        }
        //tod0 set which airports to show based on zoom level
        self.airport_renderer.draw(area, false, false);
    }

    fn build_matrix(&self, aspect_ratio: [f32; 2]) -> Mat4 {
        let mut trans = mat4(1.0, 0.0, 0.0, 0.0,
                             0.0, 1.0, 0.0, 0.0,
                             0.0, 0.0, 1.0, 0.0,
                             0.0, 0.0, 0.0, 1.0);

        trans = scale(&trans, &vec3(aspect_ratio[0], aspect_ratio[1], 1.0));
        trans = rotate(&trans, self.map_centre.borrow().get_latitude().to_radians() as f32, &vec3(1., 0., 0.));
        trans = rotate(&trans, self.map_centre.borrow().get_longitude().to_radians() as f32, &vec3(0., 1., 0.));
        trans
    }

    pub fn drop_buffers(&self) {
        self.sphere_renderer.drop_buffers();
        self.shore_line_renderer.drop_buffers();
        self.airport_renderer.drop_buffers();
    }

    pub fn get_glPoint_from_win(&self, area: &GLArea, x: f64, y: f64) {
        // We need to calculate the Z depth where the point meets the earth
// Get the earth radius
        let height = area.height() as f64;
        let width = area.width() as f64;
        let side = width.min(height);
        let earth_radius = (side as f64 / self.zoom_level.borrow().ceil() as f64 ) / 2.0;

        let depth = earth_radius;

        let mut v_scroll = 0.0;
        let mut h_scroll = 0.0;

        let win_x = x;
        let win_y = height - y - 1.0;

// Adjust x, y for scroll
//         if zoom < 1.0 {
//             let sb = canvas.get_vertical_bar();
//             let v_range_scroll = sb.get_maximum() - sb.get_thumb();
//             if v_range_scroll > 0 {
//                 let v_range_px = earth_radius * 2.0 - bounds.height as f64;
//                 v_scroll = sb.get_selection() as f64 / v_range_scroll as f64 * v_range_px;
//             }
//
//             let sb = canvas.get_horizontal_bar();
//             let h_range_scroll = sb.get_maximum() - sb.get_thumb();
//             if h_range_scroll > 0 {
//                 let h_range_px = earth_radius * 2.0 - bounds.width as f64;
//                 h_scroll = sb.get_selection() as f64 / h_range_scroll as f64 * h_range_px;
//             }
//         }

// Get the true projected x, y coordinates
        let x1 = earth_radius as i32 - (x as f64 + h_scroll) as i32;
        let y1 = earth_radius as i32 - (y as f64 + v_scroll) as i32;

        let r_squared = (x1 * x1 + y1 * y1) as f64;
        let earth_r_squared = (earth_radius * earth_radius) as f64;
        if r_squared > earth_r_squared {
            // You might want to define and use a custom NotInMapException here.
            error!("NotInMapException");
        }
        let z = (earth_r_squared - r_squared).sqrt();
        // This is the Z-depth of the clicked point.
        let normal_z = (depth - z) / earth_radius;

        // Now we need to transform this into model coordinates.
        // get_matrix_and_unwind();
        let aspect_ratio = if height < width {
            [height as f32 / width as f32, 1.0]
        } else {
            [1.0, width as f32 / height as f32]
        };
        let mat = &self.build_matrix(aspect_ratio);



    }



}
