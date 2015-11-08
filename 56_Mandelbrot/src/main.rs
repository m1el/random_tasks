use std::io::{self, Write};
use std::ops::{Add, Mul};

#[derive(Copy, Clone)]
struct Complex {
    im: f64,
    re: f64,
}

impl Complex {
    fn abs(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }
}

impl Add for Complex {
    type Output = Complex;
    fn add(self, other: Complex) -> Complex {
        Complex {
            im: self.im + other.im,
            re: self.re + other.re,
        }
    }
}

impl Mul for Complex {
    type Output = Complex;
    fn mul(self, other: Complex) -> Complex {
        Complex {
            im: self.im * other.re + self.re * other.im,
            re: self.re * other.re - self.im * other.im,
        }
    }
}

fn test_mandelbrot(c: Complex, limit: u64) -> u64 {

    let mut z = Complex { im: 0.0 as f64, re: 0.0 as f64 };
    for i in 0..limit {
        z = z * z + c;
        if z.abs() > 2.0 {
            return i;
        }
    }
    return limit;
}

fn main() {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for y in 0..1000 {
        for x in 0..1000 {
            let c = Complex {
                im: (y as f64/ 250.0) - 2.0 ,
                re: (x as f64/ 250.0) - 2.0 };
            let v = (test_mandelbrot(c, 1000) % 256) as u8;
            let _ = handle.write(&vec![255, v,v,v]);
        }
    }
}
