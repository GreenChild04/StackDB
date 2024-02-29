use std::{borrow::Cow, fs::File};
use stack_db::base::{image::Image, layer::Layer};

pub fn main() {
    let file = File::options()
        .write(true)
        .read(true)
        .append(false)
        .create(true)
        .truncate(false)
        .open("example.skdb")
        .unwrap();
    file.set_len(256).unwrap();
    let mut db = Image::new(file);
    db.write(12, b"hello, world").unwrap();

    let file = File::options()
        .write(true)
        .read(true)
        .append(false)
        .create(true)
        .truncate(false)
        .open("example.skly")
        .unwrap();
    let mut layer = Layer::new(file);
    example_write(&mut layer);
    layer.flush().unwrap();
    println!("{:?}", layer.read_unchecked(&(5..8)).unwrap());

    assert_eq!(&*db.read(12..24).unwrap(), b"hello, world");
}

fn example_write(layer: &mut Layer<File>) {
    layer.write_unchecked(4, Cow::Owned(vec![1u8, 2, 3, 4])).unwrap();
    layer.write_unchecked(8, Cow::Borrowed(b"hello, world")).unwrap();
}
