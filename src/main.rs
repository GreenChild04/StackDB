use stack_db::prelude::*;

fn main() {
    let allocator = SkdbDirAlloc::new("db.skdb").unwrap(); // or `SkdbDiskAlloc::new()`
    let mut database = StackDB::new(allocator).unwrap();

    // writing
    database.write(256, b"hello, ").unwrap();
    database.write(256+7, b"world").unwrap();

    // reading
    assert_eq!(&*database.read(256..268).unwrap(), b"hello, world");

    // flush to save all changes
    database.flush().unwrap();

    // over-writting
    database.write(256, b"H").unwrap();
    database.write(256+7, b"W").unwrap();
    database.write(268, b"!").unwrap();

    // flush again
    database.flush().unwrap();

    let mut database = StackDB::new(SkdbDirAlloc::load("db.skdb").unwrap()).unwrap();

    // reading
    assert_eq!(&*database.read(256..269).unwrap(), b"Hello, World!");

    // rebase to save space
    // database.rebase(256).unwrap(); // rebase with a 256 byte buffer
}
