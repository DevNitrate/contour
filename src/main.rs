#![allow(unused)]
use img::Image;
use photon_rs::{native::save_image, PhotonImage};

mod img;

fn main() {
    let photon: PhotonImage = photon_rs::native::open_image("nami.png").unwrap();
    let img: Image = Image::new(&photon, 8);
    let gauss: Image = img.diff_gaussian(2, 40);
    let sobel: Image = gauss.sobel();
    let resized: Image = sobel.resize_sobel();
    let ascii: Vec<String> = resized.to_ascii();
    for i in ascii.iter() {
        println!("{}", i);
    }
    save_image(resized.to_photon(), "out.png").unwrap();
}