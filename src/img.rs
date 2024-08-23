#![allow(unused)]
use core::f32;
use std::{f32::consts::PI, os::raw, vec};

use photon_rs::{conv::gaussian_blur, native::save_image, transform::resize, PhotonImage};

enum Direction {
    Horizontal,
    Vertical,
    Forward,
    Backward,
    None
}

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

    fn pix_direction(&self) -> Direction {
        let r: u8 = self.r;
        let g: u8 = self.g;
        let b: u8 = self.b;

        if r == 255 {
            if b == 255 {
                return Direction::Backward;
            } else {
                return Direction::Vertical;
            }
        } if g == 255 {
            if b == 255 {
                return Direction::Forward;
            } else {
                return Direction::Horizontal
            }
        }

        return Direction::None;
    }
}

pub struct Image {
    pub pixels: Vec<Vec<Pix>>,
    height: u32,
    width: u32,
    pub downscale: u32,
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

    pub fn get(&self, x: i32, y: i32) -> Option<&Pix> {
        if y >= self.height as i32 || x >= self.width as i32 || x < 0 || y < 0 {
            return None;
        }

        return Some(&self.pixels[y as usize][x as usize]);
    }

    pub fn to_photon(&self) -> PhotonImage {
        let mut bytes: Vec<u8> = Vec::new();

        for i in 0..self.height {
            for j in 0..self.width {
                let pix: &Pix = self.get(j as i32, i as i32).unwrap();
                bytes.push(pix.r);
                bytes.push(pix.g);
                bytes.push(pix.b);
                bytes.push(255);
            }
        }

        let img: PhotonImage = PhotonImage::new(bytes, self.width, self.height);
        return img;
    }
    
    pub fn resize(&self) -> Image {
        return Image::new(&resize(&self.to_photon(), self.width/self.downscale, self.height/self.downscale, photon_rs::transform::SamplingFilter::Nearest), 1);
    }

    pub fn resize_sobel(&self) -> Image {
        let mut pixels: Vec<Vec<Pix>> = Vec::new();

        for i in (0..self.height as i32).step_by(self.downscale as usize) {
            let mut pixy: Vec<Pix> = Vec::new();

            for j in (0..self.width as i32).step_by(self.downscale as usize) {
                let mut vertical: u32 = 0;
                let mut horizontal: u32 = 0;
                let mut forward: u32 = 0;
                let mut backward: u32 = 0;

                for y in (j - (self.downscale / 2) as i32)..(j + (self.downscale / 2) as i32) {
                    for x in (i - (self.downscale / 2) as i32)..(i + (self.downscale / 2) as i32) {
                        let default: &Pix = &Pix::new(0, 0, 0);
                        let pixel: &Pix = self.get(y, x).unwrap_or(default);

                        let dir: Direction = pixel.pix_direction();

                        match dir {
                            Direction::Vertical => {
                                vertical += 1;
                            },
                            Direction::Horizontal => {
                                horizontal += 1;
                            },
                            Direction::Backward => {
                                backward += 1;
                            },
                            Direction::Forward => {
                                forward += 1;
                            },
                            Direction::None => {}
                        }
                    }
                }

                let dir_arr: [u32; 4] = [vertical, horizontal, backward, forward];

                let sum: u32 = vertical + horizontal + backward + forward;
                let max_index: usize = dir_arr.iter().enumerate().max_by_key(|(_, &val)| val).unwrap().0;

                let max: Direction;
                
                match max_index {
                    0 => {
                        max = Direction::Vertical;
                    },
                    1 => {
                        max = Direction::Horizontal;
                    },
                    2 => {
                        max = Direction::Backward;
                    },
                    3 => {
                        max = Direction::Forward;
                    },
                    _ => {
                        max = Direction::None;
                    }
                }

                let mut r: u8 = 0;
                let mut g: u8 = 0;
                let mut b: u8 = 0;

                if sum > (self.downscale.pow(2) / 5) {
                    match max {
                        Direction::Vertical => {
                            r = 255;
                        },
                        Direction::Horizontal => {
                            g = 255;
                        },
                        Direction::Backward => {
                            r = 255;
                            b = 255;
                        },
                        Direction::Forward => {
                            g = 255;
                            b = 255;
                        },
                        Direction::None => {}
                    }                    
                }

                let pixel: Pix = Pix::new(r, g, b);
                pixy.push(pixel);
            }
            pixels.push(pixy);
        }

        let img: Image = Image {
            pixels,
            height: self.height / self.downscale,
            width: self.width / self.downscale,
            downscale: 1
        };

        return img;
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

        for i in 0..self.height as i32 {
            let mut pixy: Vec<Pix> = Vec::new();

            for j in 0..self.width as i32 {
                let big_pixel: &Pix = big_img.get(j, i).unwrap();
                let small_pixel: &Pix = small_img.get(j, i).unwrap();

                let r: u8 = small_pixel.r.checked_sub(big_pixel.r).unwrap_or(0);
                let g: u8 = small_pixel.g.checked_sub(big_pixel.g).unwrap_or(0);
                let b: u8 = small_pixel.b.checked_sub(big_pixel.b).unwrap_or(0);

                let mut avg: u8 = ((r as f32 + g as f32 + b as f32) / 3.0).round() as u8;
                // avg = avg.checked_mul(10).unwrap_or(255);
                
                if avg > 6 {
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

    pub fn sobel(&self) -> Image {
        let horizontal_kernel: [[i16; 3]; 3] = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
        let vertical_kernel: [[i16; 3]; 3] = [[1, 2, 1], [0, 0, 0], [-1, -2, -1]];

        let mut pixels: Vec<Vec<Pix>> = Vec::new();

        for i in 0..self.height as i32 {
            let mut pixy: Vec<Pix> = Vec::new();

            for j in 0..self.width as i32 {
                let default: &Pix = self.get(j, i).unwrap();

                let mid: i16 = default.gray as i16;
                let top_l: i16 = self.get(j - 1, i - 1).unwrap_or(default).gray as i16;
                let top_m: i16 = self.get(j, i - 1).unwrap_or(default).gray as i16;
                let top_r: i16 = self.get(j + 1, i - 1).unwrap_or(default).gray as i16;
                let bot_l: i16 = self.get(j - 1, i + 1).unwrap_or(default).gray as i16;
                let bot_m: i16 = self.get(j, i + 1).unwrap_or(default).gray as i16;
                let bot_r: i16 = self.get(j + 1, i + 1).unwrap_or(default).gray as i16;
                let mid_l: i16 = self.get(j - 1, i).unwrap_or(default).gray as i16;
                let mid_r: i16 = self.get(j + 1, i).unwrap_or(default).gray as i16;
                
                let h_mid: i16 = 0;
                let h_top_l: i16 = top_l * horizontal_kernel[0][0];
                let h_top_m: i16 = 0;
                let h_top_r: i16 = top_r * horizontal_kernel[0][2];
                let h_bot_l: i16 = bot_l * horizontal_kernel[2][0];
                let h_bot_m: i16 = 0;
                let h_bot_r: i16 = bot_r * horizontal_kernel[2][2];
                let h_mid_l: i16 = mid_l * horizontal_kernel[1][0];
                let h_mid_r: i16 = mid_r * horizontal_kernel[1][2];

                let v_mid: i16 = 0;
                let v_top_l: i16 = top_l * vertical_kernel[0][0];
                let v_top_m: i16 = top_m * vertical_kernel[0][1];
                let v_top_r: i16 = top_r * vertical_kernel[0][2];
                let v_bot_l: i16 = bot_l * vertical_kernel[2][0];
                let v_bot_m: i16 = bot_m * vertical_kernel[2][1];
                let v_bot_r: i16 = bot_r * vertical_kernel[2][2];
                let v_mid_l: i16 = 0;
                let v_mid_r: i16 = 0;

                let horizontal_sum: i16 = h_mid + h_top_l + h_top_m + h_top_r + h_bot_l + h_bot_m + h_bot_r + h_mid_l + h_mid_r;

                let vertical_sum: i16 = v_mid + v_top_l + v_top_m + v_top_r + v_bot_l + v_bot_m + v_bot_r + v_mid_l + v_mid_r;
                
                let magnitude: i16 = (((horizontal_sum as i32).pow(2) + (vertical_sum as i32).pow(2)) as f32).sqrt().round() as i16;

                let theta: f32 = (vertical_sum as f32).atan2(horizontal_sum as f32);
                let abs_theta: f32 = theta.abs() / PI;
                let norm_angle: f32 = theta / PI * 0.5 + 0.5;

                let mut direction: Direction = Direction::None;

                let mut r: u8 = 0;
                let mut g: u8 = 0;
                let mut b: u8 = 0;
                
                if 0.0 <= abs_theta && abs_theta < 0.1 {
                    direction = Direction::Vertical;
                } else if 0.9 < abs_theta && abs_theta <= 1.0 {
                    direction = Direction::Vertical;
                } else if 0.4 < abs_theta && abs_theta < 0.6 {
                    direction = Direction::Horizontal;
                } else if 0.1 < abs_theta && abs_theta < 0.4 {
                    direction = if theta.is_sign_positive() {Direction::Backward} else {Direction::Forward};
                } else if 0.6 < abs_theta && abs_theta < 0.9 {
                    direction = if theta.is_sign_positive() {Direction::Forward} else {Direction::Backward};
                }

                if magnitude > 100 {
                    match direction {
                        Direction::Vertical => {
                            r = 255;
                        },
                        Direction::Horizontal => {
                            g = 255;
                        },
                        Direction::Backward => {
                            r = 255;
                            b = 255;
                        },
                        Direction::Forward => {
                            g = 255;
                            b = 255;
                        },
                        Direction::None => {}
                    }
                }

                let pixel: Pix = Pix::new(r, g, b);

                pixy.push(pixel);
            }

            pixels.push(pixy);
        }

        let res: Image = Image {
            pixels,
            height: self.height,
            width: self.width,
            downscale: self.downscale
        };

        return res;
    }

    pub fn to_ascii(&self) -> Vec<String> {
        let mut res: Vec<String> = Vec::new();

        for i in 0..self.height as i32 {
            let mut row: String = String::new();

            for j in 0..self.width as i32 {
                let pixel: &Pix = self.get(j, i).unwrap();
                let r: u8 = pixel.r;
                let g: u8 = pixel.g;
                let b: u8 = pixel.b;

                if r == 255 {
                    if b == 255 {
                        row.push('\\');
                    } else {
                        row.push('|');
                    }
                } if g == 255 {
                    if b == 255 {
                        row.push('/');
                    } else {
                        row.push('_');
                    }
                } else {
                    row.push(' ');
                    row.push(' ');
                }
            }

            res.push(row);
        }
        
        return res;
    }
}