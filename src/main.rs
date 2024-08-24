#![allow(unused)]
use img::Image;
use photon_rs::{native::save_image, PhotonImage};

mod img;

fn main() {
    let photon: PhotonImage = photon_rs::native::open_image("test.png").unwrap();
    let img: Image = Image::new(&photon, 8);
    let gauss: Image = img.diff_gaussian(1, 100);
    let sobel: Image = gauss.sobel();
    let resized: Image = sobel.resize_sobel();
    let ascii: Vec<String> = resized.ascii_border();
    for i in ascii.iter() {
        println!("{}", i);
    }
}