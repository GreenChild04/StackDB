//! base-database tests

use stack_db::{base::database::StackDB, default::alloc::SkdbMemAlloc};

#[test]
fn database_read_write() {
    let mut db = StackDB::new(SkdbMemAlloc).unwrap();

    // write tests
    db.write(14, b"Hello, ").unwrap();
    db.write(14, b"hello, ").unwrap();
    db.write(21, b"World").unwrap();
    db.flush().unwrap();
    db.write(21, b"world!").unwrap();
    db.flush().unwrap();

    // read tests
    assert_eq!(&*db.read(14..21).unwrap(), b"hello, ");
    assert_eq!(&*db.read(21..26).unwrap(), b"world");
    db.rebase(256).unwrap();
    assert_eq!(&*db.read(14..27).unwrap(), b"hello, world!");
}
