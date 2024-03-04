# Some ideas that I'll implement soon
---
1. `Phantom Databases`: Versions of the stack-db that only contain a single heap layer and a read-only reference to a database that gets later appeneded to the source database to allow for concurrent writing to and reading from the database while remaining memory-safe.
2. `Database time machines`: Also separates the heap layer from the read-only layers but allows the database to pretend that the above layers don't exist.
