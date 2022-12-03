#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use crate::error::{RecolorError, Result};
use crate::palette::gen;
use crate::transfer::transfer;
use color_space::{Lab, Rgb};
use image::DynamicImage;
use std::{env, io};

mod config;
mod error;
mod math;
mod palette;
mod transfer;

fn main() {
    let img_name = match env::args().last() {
        Some(img_name) => Ok(img_name),
        None => Err(RecolorError::MissingInputImage),
    }
    .unwrap();

    let image = || -> Result<DynamicImage> { Ok(image::open(&img_name)?) }().unwrap();

    let km_rgbs = gen(&image);
    let km_labs = km_rgbs
        .iter()
        .map(|rgb| {
            let lab = Lab::from(Rgb::new(rgb[0], rgb[1], rgb[2]));
            [lab.l, lab.a, lab.b]
        })
        .collect::<Vec<_>>();

    // require user to maintain monotonical order for L
    let new_labs = || -> Result<Vec<[f64; 3]>> {
        let mut new_labs = vec![];
        loop {
            new_labs.clear();
            for lab in &km_labs {
                let mut line = String::new();
                println!("Enter new LAB for {:?}", lab);
                io::stdin().read_line(&mut line)?;
                if line.trim().is_empty() {
                    new_labs.push(*lab);
                    continue;
                }
                let floats = line.trim().split(' ').collect::<Vec<_>>();
                if floats.len() != 3 {
                    return Err(RecolorError::LABLineError(line));
                }
                let floats = floats
                    .iter()
                    .map(|s| match s.parse::<f64>() {
                        Ok(f) => Ok(f),
                        Err(_) => Err(RecolorError::LABLineError(line.clone())),
                    })
                    .collect::<Result<Vec<_>>>()?;
                new_labs.push([floats[0], floats[1], floats[2]]);
            }
            let mut last_l = new_labs[0][0];
            let mut ok = true;
            for new_lab in new_labs.iter().skip(1) {
                if new_lab[0] > last_l {
                    ok = false;
                    break;
                }
                last_l = new_lab[0];
            }
            if ok {
                break;
            }
            println!("Monotonical order for L should be maintained, try again")
        }
        Ok(new_labs)
    }()
    .unwrap();

    let mut image = image.to_rgb8();
    transfer(&mut image, &km_labs, &new_labs);
    image.save("result.jpg").unwrap();
}
