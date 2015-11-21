extern crate gl;
extern crate glutin;
extern crate libc;

use glutin::*;
use gl::types::*;
use std::mem;
use std::ptr;
use std::str;
use std::ffi::CString;

// Vertex data
static VERTEX_DATA: [GLfloat; 9] = [
     0.0,  0.5, 0.0,
     0.5, -0.5, 1.0,
    -0.5, -0.5, 2.0,
];

// Shader sources
static VS_SRC: &'static str = include_str!("vs.glsl");
static FS_SRC: &'static str = include_str!("fs.glsl");

struct GlCtx {
    shaders: Vec<Shader>,
    programs: Vec<Program>,
}

#[derive(Clone)]
struct Shader {
    id: GLuint,
    ty: GLenum,
}

#[derive(Clone)]
struct Program {
    id: GLuint,
}

impl GlCtx {
    fn new() -> GlCtx {
        GlCtx {
            shaders: Vec::new(),
            programs: Vec::new(),
        }
    }
    fn make_program(&mut self, vs_src: &str, fs_src: &str)
        -> Result<Program, String>
    {
        let vs = try!(self.compile_shader(vs_src, gl::VERTEX_SHADER));
        let fs = try!(self.compile_shader(fs_src, gl::FRAGMENT_SHADER));
        return self.link_program(&vs, &fs);
    }

    fn compile_shader(&mut self, src: &str, ty: GLenum)
        -> Result<Shader, String>
    { unsafe {
        let shader = Shader { id: gl::CreateShader(ty), ty: ty };
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader.id, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader.id);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader.id, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader.id, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(shader.id, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            let s =
                str::from_utf8(&buf).ok()
                .expect("ShaderInfoLog not valid utf8")
                .to_string();
            return Err(s);
        } else {
            self.shaders.push(shader.clone());
            return Ok(shader);
        }
    } }

    fn link_program(&mut self, vs: &Shader, fs: &Shader)
        -> Result<Program, String>
    { unsafe {
        let program = Program { id: gl::CreateProgram() };
        gl::AttachShader(program.id, vs.id);
        gl::AttachShader(program.id, fs.id);
        gl::LinkProgram(program.id);
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program.id, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program.id, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(
                program.id, len, ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar);
            let s =
                str::from_utf8(&buf).ok()
                .expect("ShaderInfoLog not valid utf8")
                .to_string();
            return Err(s);
        } else {
            self.programs.push(program.clone());
            return Ok(program);
        }
    } }
}

impl Drop for GlCtx {
    fn drop(&mut self) { unsafe {
        for p in self.programs.iter() {
            gl::DeleteProgram(p.id);
        }
        for s in self.shaders.iter() {
            gl::DeleteShader(s.id);
        }
    } }
}

fn render()
{ unsafe {
    gl::Clear(gl::COLOR_BUFFER_BIT);
    gl::DrawArrays(gl::TRIANGLES, 0, 3);
} }

fn main() {
    let float_sz: i32 = mem::size_of::<GLfloat>() as i32;

    let window = glutin::WindowBuilder::new()
        .with_title("hello".to_string())
        .with_vsync()
        .build().unwrap();

    let _ = unsafe { window.make_current() };

    gl::load_with(|symbol| window.get_proc_address(symbol));

    // Create GLSL shaders
    let mut ctx = GlCtx::new();
    let program = match ctx.make_program(VS_SRC, FS_SRC) {
        Ok(p) => p.id,
        Err(x) => panic!(x),
    };

    let mut vao = 0;
    let mut vbo = 0;

    unsafe {
        gl::Enable(gl::FRAMEBUFFER_SRGB);
        gl::ClearColor(0.3333, 0.3333, 0.3333, 1.0);
        //gl::Enable(gl::BLEND);
        // Create Vertex Array Object
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        // Create a Vertex Buffer Object and copy the vertex data to it
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (VERTEX_DATA.len() * (float_sz as usize)) as GLsizeiptr,
                       mem::transmute(VERTEX_DATA.as_ptr()),
                       gl::STATIC_DRAW);

        // Use shader program
        gl::UseProgram(program);
        gl::BindFragDataLocation(
            program, 0,
            CString::new("out_color").unwrap().as_ptr());

        // Specify the layout of the vertex data
        let pos_attr = gl::GetAttribLocation(
            program, CString::new("position").unwrap().as_ptr());

        gl::EnableVertexAttribArray(pos_attr as GLuint);
        gl::VertexAttribPointer(
            pos_attr as GLuint, 2, gl::FLOAT,
            gl::FALSE as GLboolean, float_sz*3, ptr::null());

        let idx_attr = gl::GetAttribLocation(
            program, CString::new("idx").unwrap().as_ptr());

        gl::EnableVertexAttribArray(idx_attr as GLuint);
        gl::VertexAttribPointer(
            idx_attr as GLuint, 1, gl::FLOAT,
            gl::FALSE as GLboolean, float_sz*3,
            ptr::null().offset((float_sz as isize)*2));
    }

    for event in window.wait_events() {
        render();
        let _ = window.swap_buffers();

        match event {
            Event::Closed => break,
            Event::KeyboardInput(ElementState::Pressed, code, _) => {
                match code {
                    9 => break,
                    _ => (),
                }
            },
            Event::Resized(_, _) => unsafe {
                if let Some((w, h)) = window.get_inner_size_pixels() {
                    gl::Viewport(0, 0, w as i32, h as i32);
                }
            },
            _ => ()
        }
    }

    unsafe {
    // Cleanup
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
    }
}
