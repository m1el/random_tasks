extern crate gl;
extern crate glutin;
extern crate libc;

use glutin::*;
use gl::types::*;
use std::ptr;
use std::str;
use std::ffi::CString;

use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender};
use std::mem;

// Vertex data
static VERTEX_DATA: [GLfloat; 18] = [
     0.0,  1.0, 0.0,
     0.0,  0.0, 1.0,
     1.0,  0.0, 2.0,
     0.0,  1.0, 3.0,
     1.0,  0.0, 4.0,
     1.0,  1.0, 5.0,
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

#[derive(Clone)]
struct Texture {
    id: GLuint,
}

#[derive(Clone,PartialEq)]
struct Rect {
    width: u32,
    height: u32,
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

    fn make_texture(&mut self, data: &[u8], size: &Rect)
        -> Texture
    { unsafe {
        let mut tex: GLuint = 0;
        gl::GenTextures(1, &mut tex);
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, size.width as i32, size.height as i32, 0,
                       gl::RGBA, gl::UNSIGNED_BYTE, mem::transmute(data.as_ptr()));
        gl::GenerateMipmap(gl::TEXTURE_2D);
        Texture { id: tex }
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
    gl::DrawArrays(gl::TRIANGLES, 0, 6);
} }

use std::ops::{Add, Mul};

#[derive(Copy, Clone)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    #[inline(always)]
    fn dot(&self) -> f64 {
        (self.re * self.re + self.im * self.im)
    }

    fn abs(&self) -> f64 {
        self.dot().sqrt()
    }
}

#[inline(always)]
fn cplx(re: f64, im: f64) -> Complex {
    Complex { re: re, im: im }
}

impl Add for Complex {
    type Output = Complex;

    #[inline(always)]
    fn add(self, other: Complex) -> Complex {
        cplx(self.re + other.re, self.im + other.im)
    }
}

impl Mul for Complex {
    type Output = Complex;

    #[inline(always)]
    fn mul(self, other: Complex) -> Complex {
        cplx(self.re * other.re - self.im * other.im,
             self.im * other.re + self.re * other.im)
    }
}

fn test_mandelbrot(c: Complex, limit: u64) -> u64 {
    let mut z = cplx(0.0, 0.0);
    for i in 0..limit {
        z = z * z + c;
        if z.dot() > 4.0 {
            return i;
        }
    }
    return limit;
}

fn gen_mandelbrot(rect: &Rect) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::new();
    data.resize((rect.width * rect.height * 4) as usize, 255);
    for y in 0..rect.height {
        for x in 0..rect.width {
            let p = ((y * rect.width + x) * 4) as usize;
            let c = cplx((x as f64)/(rect.width as f64)*4.0-3.0,
                         (y as f64)/(rect.height as f64)*3.0-1.5);
            let v: u8 = (test_mandelbrot(c, 255) % 256) as u8;
            data[p+0] = v;
            data[p+1] = v;
            data[p+2] = v;
        }
    }
    data
}

#[derive(Clone)]
enum ProcStatus {
    Idle,
    Processing(Rect),
    Queue(Rect),
}

impl ProcStatus {
    fn push(&mut self, r: &Rect, tx: &Sender<Rect>) {
        match self.clone() {
            ProcStatus::Idle => {
                let _ = tx.send(r.clone());
                *self = ProcStatus::Processing(r.clone());
            },
            ProcStatus::Processing(ref old) => {
                if *old != *r {
                    *self = ProcStatus::Queue(r.clone());
                }
            },
            ProcStatus::Queue(ref old) => {
                if *old != *r {
                    *self = ProcStatus::Queue(r.clone());
                }
            },
        }
    }

    fn dequeue(&mut self, tx: &Sender<Rect>) {
        match self.clone() {
            ProcStatus::Idle => {
                panic!("invalid state for dequeue!");
            },
            ProcStatus::Processing(_) => {
                *self = ProcStatus::Idle;
            },
            ProcStatus::Queue(ref r) => {
                let _ = tx.send(r.clone());
                *self = ProcStatus::Processing(r.clone());
            },
        }
    }
}

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

    let rect = Rect { width: 256, height: 256 };
    let data = gen_mandelbrot(&rect);
    let mut tex = ctx.make_texture(&data, &rect);

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

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, tex.id);
        let tex_loc = gl::GetUniformLocation(
            program, CString::new("uTexture").unwrap().as_ptr());
        gl::Uniform1i(tex_loc, 0);
    }

    let (size_tx, size_rx) = channel::<Rect>();
    let (data_tx, data_rx) = channel::<(Rect, Vec<u8>)>();
    thread::spawn(move || {
        loop {
            match size_rx.recv() {
                Ok(rect) => {
                    let data = gen_mandelbrot(&rect);
                    let _ = data_tx.send((rect, data));
                },
                Err(_) => (),
            }
        }
    });

    let mut proc_status = ProcStatus::Idle;

    'outer: loop {
        match data_rx.try_recv() {
            Ok((rect, data)) => {
                println!("new texture: {}x{}", rect.width, rect.height);
                let new_tex = ctx.make_texture(&data, &rect);
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_2D, new_tex.id);
                    gl::DeleteTextures(1, &tex.id);
                }
                tex = new_tex;
                proc_status.dequeue(&size_tx);
            },
            Err(_) => (),
        }

        render();
        let _ = window.swap_buffers();

        for event in window.poll_events() {
            match event {
                Event::Closed => break 'outer,
                Event::KeyboardInput(ElementState::Pressed, code, _) => {
                    match code {
                        9 => break 'outer,
                        _ => (),
                    }
                },
                Event::Resized(_, _) => unsafe {
                    if let Some((w, h)) = window.get_inner_size_pixels() {
                        let r = Rect { width: w, height: h };
                        gl::Viewport(0, 0, w as i32, h as i32);
                        proc_status.push(&r, &size_tx);
                    }
                },
                _ => ()
            }
        }
    }

    unsafe {
    // Cleanup
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
    }
}
