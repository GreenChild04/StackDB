use std::{io::{Read, Seek, SeekFrom, Write}, ops::Range};

use crate::errors::Error;

/// The base binary database
pub struct Image<D: Write + Read + Seek> {
    data: D,
}

impl<D: Write + Read + Seek> Image<D> {
    #[inline]
    pub fn new(stream: D) -> Self {
        Self {
            data: stream,
        }
    }

    /// Writes the specified bytes to the image
    #[inline]
    pub fn write(&mut self, addr: u64, bytes: &[u8]) -> Result<(), Error> {
        self.data.seek(SeekFrom::Start(addr))?;
        self.data.write_all(bytes)?;

        Ok(())
    }

    /// Reads the specified bytes from the image
    #[inline]
    pub fn read(&mut self, addrs: Range<u64>) -> Result<Box<[u8]>, Error> {
        let mut buffer = vec![0u8; (addrs.end-addrs.start) as usize].into_boxed_slice();
        
        self.data.seek(SeekFrom::Start(addrs.start))?;
        self.data.read_exact(&mut buffer)?;

        Ok(buffer)
    }
}
