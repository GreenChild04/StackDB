use std::fs::File;
use stack_db::base::image::Image;

pub fn main() {
    let file = File::options()
        .write(true)
        .read(true)
        .append(false)
        .create(true)
        .open("example.skdb")
        .unwrap();
    file.set_len(256).unwrap();
    let mut db = Image::new(file);
    db.write(12, b"hello, world").unwrap();

    assert_eq!(&*db.read(12..24).unwrap(), b"hello, world");
}
