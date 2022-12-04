use crate::config::*;
use crate::error::Result;
use crate::math::*;
use color_space::{Lab, Rgb};
use image::RgbImage;

struct TransferContext {
    pub mappings: [([f64; 3], [f64; 3]); K],
    pub lambda: [[f64; K]; K],
    pub sigma_r: f64,
}

impl TransferContext {
    pub fn new(km_labs: &[[f64; 3]], new_labs: &[[f64; 3]]) -> Result<Self> {
        let mut a = [[0f64; K * K + 1]; K * K];
        let mut sum_dis = 0f64;
        let mut mappings = [([0f64; 3], [0f64; 3]); K];
        for i in 0..K {
            mappings[i].0 = km_labs[i];
            mappings[i].1 = new_labs[i];
        }
        let mut dis = [[0f64; K]; K];
        for i in 0..K {
            for j in 0..K {
                dis[i][j] = f64::sqrt(Vec3::sqr(Vec3(km_labs[i]) - Vec3(km_labs[j])));
                sum_dis += dis[i][j];
            }
        }
        let mean_dis = sum_dis / (K * K) as f64;
        #[allow(clippy::needless_range_loop)]
        for i in 0..K {
            #[allow(clippy::needless_range_loop)]
            for j in 0..K {
                let idx = i * K + j;
                for k in 0..K {
                    let lambda_idx = i * K + k;
                    a[idx][lambda_idx] =
                        f64::exp(-dis[j][k] * dis[j][k] / (2f64 * mean_dis * mean_dis));
                }
                a[idx][K * K] = if i == j { 1f64 } else { 0f64 };
            }
        }
        gauss(&mut a)?;
        let mut lambda = [[0f64; K]; K];
        for i in 0..K {
            for j in 0..K {
                lambda[i][j] = a[i * K + j][K * K];
            }
        }
        Ok(Self {
            mappings,
            lambda,
            sigma_r: mean_dis,
        })
    }
}

fn pixel_transfer(ctx: &TransferContext, rgb: [f64; 3]) -> [f64; 3] {
    let lab = Lab::from(Rgb::new(rgb[0], rgb[1], rgb[2]));
    let mut new_lab_deltas: Vec<Vec3> = vec![];
    for mapping in ctx.mappings {
        let x = Vec3([lab.l, lab.a, lab.b]);
        let c = Vec3(mapping.0);
        let c_tick = Vec3(mapping.1);
        let c_delta = c_tick - c;
        if Vec3::sqr(c_delta) < RGB_EPS {
            new_lab_deltas.push(c_delta);
            continue;
        }
        let c_b = c.border_point(c_delta);
        let x_0 = x + c_delta;
        let delta = if !x_0.as_lab_out_of_rgb() {
            let x_b = x.border_point(c_delta);
            let scale = f64::sqrt(Vec3::sqr(x_b - x)) / f64::sqrt(Vec3::sqr(c_b - c));
            let scale = f64::min(scale, 1f64);
            (c_tick - c) * scale
        } else {
            let mut dir = x_0 - c_tick;
            while (c_tick + dir).as_lab_out_of_rgb() {
                dir = dir / 2f64;
            }
            let x_b = c_tick.border_point(dir);
            let scale = f64::sqrt(Vec3::sqr(c_delta)) / f64::sqrt(Vec3::sqr(c_b - c));
            (x_b - x) / scale
        };
        new_lab_deltas.push(delta);
    }
    let mut delta = Vec3([0f64; 3]);
    let mut weights = [0f64; K];
    #[allow(clippy::needless_range_loop)]
    for i in 0..K {
        let mut w = 0f64;
        for j in 0..K {
            w += ctx.lambda[i][j]
                * f64::exp(
                    -Vec3::sqr(Vec3([lab.l, lab.a, lab.b]) - Vec3(ctx.mappings[j].0))
                        / (2f64 * ctx.sigma_r * ctx.sigma_r),
                );
        }
        assert!(!w.is_nan());
        weights[i] = f64::max(w, 0f64);
    }
    let weights = normalize(weights);
    for i in 0..K {
        assert!(!weights[i].is_nan());
        delta = delta + new_lab_deltas[i] * weights[i];
    }
    let lab = [lab.l + delta.0[0], lab.a + delta.0[1], lab.b + delta.0[2]];
    let rgb = Rgb::from(Lab::new(lab[0], lab[1], lab[2]));
    [rgb.r, rgb.g, rgb.b]
}

pub fn transfer(image: &mut RgbImage, km_labs: &[[f64; 3]], new_labs: &[[f64; 3]]) {
    let ctx = TransferContext::new(km_labs, new_labs).unwrap();
    let mut color_map = [[[[0f64; 3]; TRANSFER_BIN_COUNT]; TRANSFER_BIN_COUNT]; TRANSFER_BIN_COUNT];
    #[allow(clippy::needless_range_loop)]
    for ri in 0..TRANSFER_BIN_COUNT {
        for gi in 0..TRANSFER_BIN_COUNT {
            for bi in 0..TRANSFER_BIN_COUNT {
                let [r, g, b] = [
                    (ri * TRANSFER_BIN_WIDTH) as f64,
                    (gi * TRANSFER_BIN_WIDTH) as f64,
                    (bi * TRANSFER_BIN_WIDTH) as f64,
                ];
                let [nr, ng, gb] = pixel_transfer(&ctx, [r, g, b]);
                color_map[ri][gi][bi] = [nr, ng, gb];
            }
        }
    }
    image.pixels_mut().for_each(|p| {
        let [r, g, b] = p.0;
        let [ri, gi, bi] = [
            r as usize / TRANSFER_BIN_WIDTH,
            g as usize / TRANSFER_BIN_WIDTH,
            b as usize / TRANSFER_BIN_WIDTH,
        ];
        let nrgb = if ri + 1 >= TRANSFER_BIN_COUNT
            || gi + 1 >= TRANSFER_BIN_COUNT
            || bi + 1 >= TRANSFER_BIN_COUNT
        {
            color_map[ri][gi][bi]
        } else {
            let rt = (r as f64 - (ri * TRANSFER_BIN_WIDTH) as f64) / TRANSFER_BIN_WIDTH as f64;
            let gt = (g as f64 - (gi * TRANSFER_BIN_WIDTH) as f64) / TRANSFER_BIN_WIDTH as f64;
            let bt = (b as f64 - (bi * TRANSFER_BIN_WIDTH) as f64) / TRANSFER_BIN_WIDTH as f64;
            let mut c = [[[Vec3([0f64; 3]); 2]; 2]; 2];
            for i in 0..2 {
                for j in 0..2 {
                    for k in 0..2 {
                        c[i][j][k] = Vec3(color_map[ri + i][gi + j][bi + k]);
                    }
                }
            }
            let c00 = c[0][0][0] * (1f64 - rt) + c[1][0][0] * rt;
            let c01 = c[0][0][1] * (1f64 - rt) + c[1][0][1] * rt;
            let c10 = c[0][1][0] * (1f64 - rt) + c[1][1][0] * rt;
            let c11 = c[0][1][1] * (1f64 - rt) + c[1][1][1] * rt;
            let c0 = c00 * (1f64 - gt) + c10 * gt;
            let c1 = c01 * (1f64 - gt) + c11 * gt;
            let c = c0 * (1f64 - bt) + c1 * bt;
            c.0
        };
        p.0 = [nrgb[0] as u8, nrgb[1] as u8, nrgb[2] as u8];
    });
}
