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
static VS_SRC: &'static str = r#"
#version 130
in vec2 position;
in float idx;
varying vec4 color;
void main() {
   gl_Position = vec4(position, 0.0, 1.0);
   if (idx == 0.0) {
       color = vec4(1.0, 0.0, 0.0, 1.0);
   }
   if (idx == 1.0) {
       color = vec4(0.0, 1.0, 0.0, 1.0);
   }
   if (idx == 2.0) {
       color = vec4(0.3333, 0.3333, 0.3333, 1.0);
   }
}"#;

static FS_SRC: &'static str = r#"
#version 130
varying vec4 color;
out vec4 out_color;
void main() {
   out_color = color;
}"#;


fn compile_shader(src: &str, ty: GLenum) -> GLuint { unsafe {
    let shader = gl::CreateShader(ty);
    // Attempt to compile the shader
    let c_str = CString::new(src.as_bytes()).unwrap();
    gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
    gl::CompileShader(shader);

    // Get the compile status
    let mut status = gl::FALSE as GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

    // Fail on error
    if status != (gl::TRUE as GLint) {
        let mut len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::with_capacity(len as usize);
        buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
        gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        gl::DeleteShader(shader);
        panic!("{}", str::from_utf8(&buf).ok().expect("ShaderInfoLog not valid utf8"));
    }
    shader
} }

fn link_program(vs: GLuint, fs: GLuint) -> GLuint
{ unsafe {
    let program = gl::CreateProgram();
    gl::AttachShader(program, vs);
    gl::AttachShader(program, fs);
    gl::LinkProgram(program);
    // Get the link status
    let mut status = gl::FALSE as GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

    // Fail on error
    if status != (gl::TRUE as GLint) {
        let mut len: GLint = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::with_capacity(len as usize);
        buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
        gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        panic!("{}", str::from_utf8(&buf).ok().expect("ProgramInfoLog not valid utf8"));
    }
    program
} }

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
    let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
    let program = link_program(vs, fs);

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
        gl::DeleteProgram(program);
        gl::DeleteShader(fs);
        gl::DeleteShader(vs);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
    }
}
