# stack-db
> A (basically) infinitely stacking database that has both readonly safety and incredible write speeds at the same time.

## Examples
---
### Example of a basic in-memory binary database
Here is a basic in-memory database that only deals with binary indexes and data (that uses the allocators provided by the library)
```rust
use stack_db::prelude::*;

let allocator = SkdbMemAlloc; // or `SkdbDiskAlloc::new()`
let mut database = StackDB::new(allocator);

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

// reading
assert_eq!(&*database.read(256..269).unwrap(), b"Hello, World!");

// rebase to save space
database.rebase(256).unwrap(); // rebase with a 256 byte buffer
```
