#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use crate::palette::gen;
use crate::transfer::transfer as image_transfer;
use color_space::{Lab, Rgb};
use image::DynamicImage;
use js_sys::Float64Array;
use wasm_bindgen::prelude::*;

mod config;
mod error;
mod math;
mod palette;
mod transfer;

fn palette_to_labs(palette: Float64Array) -> Vec<[f64; 3]> {
    let mut labs = vec![];
    let mut idx = 0;
    let palette = palette.to_vec();
    while idx < palette.len() {
        assert!(idx + 3 <= palette.len());
        let lab = Lab::from(Rgb::new(palette[idx], palette[idx + 1], palette[idx + 2]));
        labs.push([lab.l, lab.a, lab.b]);
        idx += 3;
    }
    labs
}

#[wasm_bindgen]
pub fn gen_palette(image_raw: Vec<u8>) -> Float64Array {
    let img = image::load_from_memory(&image_raw).unwrap();
    let rgbs = gen(&img);
    let array = rgbs.iter().flatten().copied().collect::<Vec<f64>>();
    let f64array = Float64Array::new_with_length(array.len() as u32);
    f64array.copy_from(&array);
    f64array
}

#[wasm_bindgen]
pub fn transfer(
    image_raw: Vec<u8>,
    old_palette: Float64Array,
    new_palette: Float64Array,
) -> Vec<u8> {
    let img = image::load_from_memory(&image_raw).unwrap();
    let mut img = img.to_rgb8();
    let km_labs = palette_to_labs(old_palette);
    let new_labs = palette_to_labs(new_palette);
    image_transfer(&mut img, &km_labs, &new_labs);
    let img = DynamicImage::from(img);
    img.to_rgba8().to_vec()
}
