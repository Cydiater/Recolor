use color_space::{Lab, Rgb};
use image::DynamicImage;

use crate::config::*;
use crate::error::{RecolorError, Result};
use crate::math::Vec3;

#[derive(Debug, Clone)]
struct PixelInfo {
    l: f64,
    a: f64,
    b: f64,
    w: f64,
    mean: [f64; 3],
}

pub fn gen(image: &DynamicImage) -> Vec<[f64; 3]> {
    let image = image.to_rgb8();

    let mut labws = || -> Vec<PixelInfo> {
        // sum of r, g, b, cnt
        let mut sum = [[[(0f64, 0f64, 0f64, 0usize); BIN_COUNT]; BIN_COUNT]; BIN_COUNT];
        image.pixels().for_each(|p| {
            let [r, g, b] = p.0;
            let lab = Lab::from(Rgb::new(r as f64, g as f64, b as f64));
            let [x, y, z] = [
                r as usize / BIN_WIDTH,
                g as usize / BIN_WIDTH,
                b as usize / BIN_WIDTH,
            ];
            sum[x][y][z].0 += lab.l;
            sum[x][y][z].1 += lab.a;
            sum[x][y][z].2 += lab.b;
            sum[x][y][z].3 += 1;
        });
        let mut labws = vec![];
        sum.iter().for_each(|sum| {
            sum.iter().for_each(|sum| {
                sum.iter().for_each(|sum| {
                    if sum.3 > 0 {
                        let [l, a, b] = [
                            sum.0 / sum.3 as f64,
                            sum.1 / sum.3 as f64,
                            sum.2 / sum.3 as f64,
                        ];
                        labws.push(PixelInfo {
                            l,
                            a,
                            b,
                            w: sum.3 as f64,
                            mean: [0f64, 0f64, 0f64],
                        })
                    }
                })
            })
        });
        labws
    }();

    let km_init_labs = || -> Result<Vec<[f64; 3]>> {
        let mut labws = labws.clone();
        let mut km_init_labs = vec![];
        for _ in 0..K {
            labws.sort_by(|lhs, rhs| rhs.w.partial_cmp(&lhs.w).unwrap());
            let init_lab = match labws.pop() {
                Some(init_lab) => Ok(init_lab),
                None => Err(RecolorError::KMeansInitError),
            }?;
            km_init_labs.push([init_lab.l, init_lab.a, init_lab.b]);
            labws.iter_mut().for_each(|p: &mut PixelInfo| {
                let d2 =
                    Vec3::sqr(Vec3([init_lab.l, init_lab.a, init_lab.b]) - Vec3([p.l, p.a, p.b]));
                p.w *= 1f64 - f64::exp(-d2 / (DELTA_A * DELTA_A));
            });
        }
        Ok(km_init_labs)
    }()
    .unwrap();

    let km_labs = || -> Vec<[f64; 3]> {
        let mut means_with_sum = km_init_labs
            .iter()
            .map(|m| (*m, [0f64; 4]))
            .collect::<Vec<_>>();
        loop {
            labws.iter_mut().for_each(|p| {
                p.mean = [0f64; 3];
                let mut dis = p.l * p.l + p.a * p.a + p.b * p.b;
                let mut idx = -1;
                for (this_idx, (m, _)) in means_with_sum.iter().enumerate() {
                    let this_dis = Vec3::sqr(Vec3([p.l, p.a, p.b]) - Vec3(*m));
                    if this_dis < dis {
                        dis = this_dis;
                        idx = this_idx as isize;
                        p.mean = *m;
                    }
                }
                if idx != -1 {
                    let idx = idx as usize;
                    means_with_sum[idx].1[0] += p.l * p.w;
                    means_with_sum[idx].1[1] += p.a * p.w;
                    means_with_sum[idx].1[2] += p.b * p.w;
                    means_with_sum[idx].1[3] += p.w;
                }
            });
            let mut updated = false;
            means_with_sum.iter_mut().for_each(|m| {
                let [l, a, b] = [m.1[0] / m.1[3], m.1[1] / m.1[3], m.1[2] / m.1[3]];
                if Vec3::sqr(Vec3([l, a, b]) - Vec3(m.0)) > EPS {
                    updated = true;
                    *m = ([l, a, b], [0f64; 4]);
                }
            });
            if !updated {
                break;
            }
        }
        let mut km_labs: Vec<_> = means_with_sum.iter().map(|m| m.0).collect();
        km_labs.sort_by(|lhs, rhs| rhs[0].partial_cmp(&lhs[0]).unwrap());
        km_labs
    }();

    km_labs
        .iter()
        .map(|lab| {
            let rgb = Rgb::from(Lab::new(lab[0], lab[1], lab[2]));
            [rgb.r, rgb.g, rgb.b]
        })
        .collect::<Vec<_>>()
}
