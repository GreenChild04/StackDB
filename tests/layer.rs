use std::{borrow::Cow, io::Cursor};
use stack_db::base::layer::Layer;

#[test]
fn test_read_write() {
    let mut layer_data = vec![0u8; 256];
    let mut layer = Layer::new(Cursor::new(&mut layer_data));

    layer.write_unchecked(128, Cow::Borrowed(b"hello, world")).unwrap();
    layer.write_unchecked(4, Cow::Borrowed(&[0, 1, 2, 3, 4, 5, 6, 7, 8])).unwrap();

    assert_eq!(&*layer.read_unchecked(128..140).unwrap().1, b"Hello, world");
    layer.flush().unwrap();

    assert_eq!(&*layer.read_unchecked(4..13).unwrap().1, &[0, 1, 2, 3, 4, 5, 6, 7, 8]);
}
