use img::Image;
use photon_rs::{native::save_image, PhotonImage};

mod img;

fn main() {
    let photon: PhotonImage = photon_rs::native::open_image("nami.png").unwrap();
    let img: Image = Image::new(&photon, 8);
    let gauss: Image = img.diff_gaussian(0, 80);
    save_image(gauss.to_photon(), "out.png").unwrap();
}