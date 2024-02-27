//! The mapper of the layer that can either live on the **heap** or **disk**

use std::{borrow::Cow, io::{Read, Seek, Write}};
use crate::{base::layer::get_u64, errors::Error};
use super::Section;

/// The mapper that holds all the writes to the layer and their location mapping in the database
#[derive(Debug)]
pub enum Mapper<'l> {
    /// A **read-write** version of the mapper on the **heap**
    Heap {
        /// The current write cursor to speed up sequential qrites
        write_cursor: (u64, usize),
        /// *self explainitory*
        mapper: Vec<Section<'l>>,
    },
    /// A **read-only** version of the mapper on the **disk**
    Disk,
}

/// A read-only iterator of the mapper that can live on either the heap or disk 
pub struct MapperIter<'l, Stream: Write + Read + Seek> {
    mapper: &'l Mapper<'l>,
    stream: &'l mut Stream,
    size: u64,
    /// the index in the mapper
    idx: usize,
    /// the **actual** location in the layer
    cursor: u64,
}

impl<'l> Default for Mapper<'l> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'l> Mapper<'l> {
    /// Creates a new **heap**-based mapper
    #[inline]
    pub fn new() -> Self {
        Self::Heap {
            write_cursor: (0, 0),
            mapper: Vec::new(),
        }
    }

    /// Grabs the internal heap representation; if on disk, throw the `ReadOnly` error
    #[inline]
    pub fn get_writer(&mut self) -> Result<(&mut Vec<Section<'l>>, &mut (u64, usize)), Error> {
        if let Self::Heap { write_cursor, mapper } = self {
            Ok((mapper, write_cursor))
        } else {
            Err(Error::ReadOnly)
        }
    }

    /// Generates an iterator over the interal mapper, from the stream, size and layer read cursor position
    pub fn iter<'a, Stream: Read + Write + Seek>(&'a self, stream: &'a mut Stream, size: u64, cursor: u64) -> Result<MapperIter<'a, Stream>, Error> {
        stream.seek(std::io::SeekFrom::Start(cursor))?;
        Ok(MapperIter {
            mapper: self,
            stream,
            size,
            idx: 0,
            cursor,
        })
    }
}

/// for unwrapping results within a function that returns an optional result concisely
macro_rules! optres {
    ($expr:expr) => {
        match $expr {
            Ok(x) => x,
            Err(e) => return Some(Err(e.into())),
        }
    }
}

impl<'l, Stream: Write + Read + Seek> Iterator for MapperIter<'l, Stream> {
    type Item = Result<Section<'l>, Error>;

    fn next(&mut self) -> Option<Self::Item> { // probably not a issue but, it loads the entire layer section into memory
        Some(Ok(match self.mapper {
            Mapper::Heap { mapper, .. } => {
                if self.idx == mapper.len() { return None };
                let out = mapper[self.idx].clone();
                self.idx += 1;
                out
            },
            Mapper::Disk => {
                // check for end of layer
                if self.cursor == self.size { return None };
                
                // read bounds
                let mut buffer = [0u8; (u64::BITS as usize/8) * 2]; // buffer for two `u64` values: `bounds.start` & `bounds.end`
                optres!(self.stream.read_exact(&mut buffer));
                let bounds = optres!(get_u64(&buffer, 0..8))..optres!(get_u64(&buffer, 8..16));

                // load layer section data into the heap
                let size = bounds.end - bounds.start;
                let mut data = vec![0u8; size as usize];
                optres!(self.stream.read_exact(&mut data));

                self.cursor += size;
                (bounds, Cow::Owned(data))
            },
        }))
    }
}
