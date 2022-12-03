use crate::config::{EPS, RGB_EPS};
use crate::error::{RecolorError, Result};
use color_space::{Lab, Rgb};
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Vec3(pub [f64; 3]);

pub fn normalize<const N: usize>(mut v: [f64; N]) -> [f64; N] {
    let sum: f64 = v.iter().sum();
    assert!(sum > 0f64);
    v.iter_mut().for_each(|x| *x /= sum);
    v
}

pub fn gauss<const N: usize>(a: &mut [[f64; N + 1]; N]) -> Result<()> {
    for i in 0..N {
        for j in i + 1..N {
            if f64::abs(a[j][i]) > f64::abs(a[i][i]) {
                a.swap(i, j);
            }
        }
        if f64::abs(a[i][i]) < EPS {
            return Err(RecolorError::GaussError);
        }
        for j in i + 1..N {
            if f64::abs(a[j][i]) > EPS {
                let c = a[j][i] / a[i][i];
                for k in i..(N + 1) {
                    a[j][k] -= a[i][k] * c;
                }
            }
        }
    }
    for i in (0..N).rev() {
        for j in 0..i {
            if f64::abs(a[j][i]) > EPS {
                let c = a[j][i] / a[i][i];
                for k in i..(N + 1) {
                    a[j][k] -= a[i][k] * c;
                }
            }
        }
        a[i][N] /= a[i][i];
    }
    Ok(())
}

impl Vec3 {
    pub fn sqr(self) -> f64 {
        self.0[0] * self.0[0] + self.0[1] * self.0[1] + self.0[2] * self.0[2]
    }
    pub fn as_lab_out_of_rgb(&self) -> bool {
        let lab = Lab::new(self.0[0], self.0[1], self.0[2]);
        let rgb = Rgb::from(lab);
        if rgb.r < -RGB_EPS
            || rgb.r > 255.0 + RGB_EPS
            || rgb.g < -RGB_EPS
            || rgb.g > 255.0 + RGB_EPS
        {
            true
        } else {
            rgb.b < -RGB_EPS || rgb.b > 255.0 + RGB_EPS
        }
    }
    pub fn border_point(&self, mut dir: Vec3) -> Vec3 {
        let mut l = *self + dir;
        assert!(!l.as_lab_out_of_rgb());
        while !(l + dir).as_lab_out_of_rgb() {
            dir = dir + dir;
        }
        let mut r = l + dir;
        while Vec3::sqr(r - l) > RGB_EPS {
            let m = (l + r) / 2f64;
            if m.as_lab_out_of_rgb() {
                r = m;
            } else {
                l = m;
            }
        }
        l
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
        ])
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
        ])
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, other: f64) -> Self {
        Vec3([self.0[0] / other, self.0[1] / other, self.0[2] / other])
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Vec3([self.0[0] * other, self.0[1] * other, self.0[2] * other])
    }
}
