//! base-database tests

use std::io::Cursor;
use stack_db::base::{database::{allocator::Allocator, StackDB}, layer::Layer};

pub struct Alloc;
impl Allocator<'static> for Alloc {
    type LayerStream = Cursor<Box<[u8]>>;
    
    fn load_layers(&self) -> Result<Vec<stack_db::base::layer::Layer<'static, Self::LayerStream>>, stack_db::errors::Error> {
        Ok(Vec::new())
    }

    fn add_layer(&mut self) -> Result<Layer<'static, Self::LayerStream>, stack_db::errors::Error> {
        Ok(Layer::new(Cursor::new(vec![0u8; 256].into_boxed_slice())))
    }

    fn drop_layer(&mut self) -> Result<(), stack_db::errors::Error> {
        Ok(())
    }
}

#[test]
fn database_read_write() {
    let mut db = StackDB::new(Alloc);

    // write tests
    db.write(14, b"Hello, ").unwrap();
    db.write(14, b"hello, ").unwrap();
    db.write(21, b"World").unwrap();
    db.flush().unwrap();
    db.write(21, b"world!").unwrap();
    db.flush().unwrap();

    // read tests
    assert_eq!(&*db.read(14..21).unwrap(), b"hello, ");
    assert_eq!(&*db.read(21..26).unwrap(), b"world!");
}
