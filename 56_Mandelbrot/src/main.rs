use std::io::{self, Write};
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

fn main() {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let size: usize = 8000;
    let mut pixels: Vec<u8> = Vec::with_capacity(size*size*4);
    pixels.resize(size*size*4, 0);
    for y in 0..size {
        let yy = 4.0*(y as f64)/(size as f64) - 2.0;
        for x in 0..size {
            let xx = 4.0*(x as f64)/(size as f64) - 2.0;
            let v = (test_mandelbrot(cplx(xx, yy), 1000) % 256) as u8;
            let start = (y * size + x) * 4;
            pixels[start+0] = v;
            pixels[start+1] = v;
            pixels[start+2] = v;
            pixels[start+3] = 255;
        }
    }
    let _ = handle.write(&pixels);
}
