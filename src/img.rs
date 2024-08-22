#![allow(unused)]
use std::{f64::consts::{E, PI}, os::raw, vec};

use photon_rs::{conv::gaussian_blur, native::save_image, PhotonImage};

#[derive(Debug, Clone, Copy)]
pub struct Pix {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub gray: u8
}

impl Pix {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        let gray: u8 = ((r as f32 + g as f32 + b as f32) / 3.0).round() as u8;

        Pix {
            r,
            g,
            b,
            gray
        }
    }
}

pub struct Image {
    pub pixels: Vec<Vec<Pix>>,
    height: u32,
    width: u32,
    downscale: u32,
}

impl Image {
    pub fn new(img: &PhotonImage, downscale: u32) -> Self {
        let mut pixels: Vec<Vec<Pix>> = Vec::new();
        let mut pixels1d: Vec<Pix> = Vec::new();
        let height = img.get_height();
        let width = img.get_width();

        let raw_pixels: Vec<u8> = img.get_raw_pixels();
        
        for i in (0..(height*width)*4).step_by(4) {
            let r: u8 = raw_pixels[i as usize];
            let g: u8 = raw_pixels[(i+1) as usize];
            let b: u8 = raw_pixels[(i+2) as usize];
            let pix: Pix = Pix::new(r, g, b);

            pixels1d.push(pix);
        }

        let mut count: usize = 0;

        for i in 0..height {
            let mut pixy: Vec<Pix> = Vec::new();

            for j in 0..width {
                pixy.push(pixels1d[count]);
                count += 1;
            }

            pixels.push(pixy);
        }

        Image {
            pixels,
            height,
            width,
            downscale
        }
    }

    pub fn get(&self, x: usize, y: usize) -> &Pix {
        return &self.pixels[y][x];
    }

    pub fn to_photon(&self) -> PhotonImage {
        let mut bytes: Vec<u8> = Vec::new();

        for i in 0..self.height {
            for j in 0..self.width {
                let pix: &Pix = self.get(j as usize, i as usize);
                bytes.push(pix.r);
                bytes.push(pix.g);
                bytes.push(pix.b);
                bytes.push(255);
            }
        }

        let img: PhotonImage = PhotonImage::new(bytes, self.width, self.height);
        return img;
    }
    
    pub fn to_grayscale_image(&self, output: &str) {
        todo!()
    }

    pub fn diff_gaussian(&self, sigma: i32, deviation: i32) -> Image {
        let mut big_blur: PhotonImage = self.to_photon();
        let mut small_blur: PhotonImage = big_blur.clone();
        gaussian_blur(&mut big_blur, deviation);
        gaussian_blur(&mut small_blur, sigma);

        // save_image(big_blur.clone(), "big.png");
        // save_image(small_blur.clone(), "small.png");

        let big_img: Image = Image::new(&big_blur, self.downscale);
        let small_img: Image = Image::new(&small_blur, self.downscale);

        let mut res: Vec<Vec<Pix>> = Vec::new();

        for i in 0..self.height as usize {
            let mut pixy: Vec<Pix> = Vec::new();

            for j in 0..self.width as usize {
                let big_pixel: &Pix = big_img.get(j, i);
                let small_pixel: &Pix = small_img.get(j, i);

                let r: u8 = small_pixel.r.checked_sub(big_pixel.r).unwrap_or(0);
                let g: u8 = small_pixel.g.checked_sub(big_pixel.g).unwrap_or(0);
                let b: u8 = small_pixel.b.checked_sub(big_pixel.b).unwrap_or(0);

                let mut avg: u8 = ((r as f32 + g as f32 + b as f32) / 3.0).round() as u8;
                // avg = avg.checked_mul(10).unwrap_or(255);
                
                if avg > 3 {
                    avg = 255;
                } else {
                    ();
                }
                
                let pix: Pix = Pix::new(avg, avg, avg);

                pixy.push(pix);
            }

            res.push(pixy);
        }
        
        Image {
            pixels: res,
            height: self.height,
            width: self.width,
            downscale: self.downscale
        }
    }

    
}
