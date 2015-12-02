use std::ops::{Add, Mul};

#[derive(Copy, Clone)]
pub struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    #[inline(always)]
    pub fn dot(&self) -> f64 {
        (self.re * self.re + self.im * self.im)
    }

    #[allow(dead_code)]
    pub fn abs(&self) -> f64 {
        self.dot().sqrt()
    }
}

#[inline(always)]
pub fn cplx(re: f64, im: f64) -> Complex {
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
