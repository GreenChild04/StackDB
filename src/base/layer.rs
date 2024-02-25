//! A layer/frame of which gets *stacked* to form the database
pub mod mapper;

use std::{borrow::Cow, io::{BufWriter, Read, Seek, Write}, ops::Range};
use crate::errors::Error;
use mapper::Mapper;

pub type Section<'l> = (Range<u64>, Cow<'l, [u8]>);

/// Represents a layer (either in the heap or disk) in the stack-db that *stacks*
#[derive(Debug)]
pub struct Layer<'l, Stream: Write + Read + Seek> {
    /// The bounds of the layer; the range of the layer
    pub bounds: Option<Range<u64>>,
    /// The mapper that maps to either the heap or disk
    mapper: Mapper<'l>,
    /// The total size of all the writes in the layer
    pub size: u64,
    /// The current read cursor to speed up sequential reads
    pub read_cursor: (u64, usize),
    /// The underlying file reader/writer
    stream: Stream,
}

/// Grabs a u64 from a buffer
#[inline]
fn get_u64(buffer: &[u8], range: Range<usize>) -> Result<u64, Error> {
    Ok(u64::from_be_bytes(
        if let Some(Ok(x)) = buffer.get(range).map(|x| x.try_into())
            { x }
        else { 
            return Err(Error::DBCorrupt(Box::new(Error::InvalidLayer)));
        }
    ))
}

/// used for error handling in iterators
#[inline]
fn until_err<T, E>(err: &mut &mut Result<(), E>, item: Result<T, E>) -> Option<T> {
    match item {
        Ok(item) => Some(item),
        Err(e) => {
            **err = Err(e);
            None
        }
    }
}

impl<'l,  Stream: Write + Read + Seek> Layer<'l, Stream> {
    #[inline]
    pub fn new(stream: Stream) -> Self {
        Self {
            bounds: None,
            mapper: Mapper::new(),
            size: 0,
            read_cursor: (0, 0),
            stream,
        }
    }

    #[inline]
    pub fn load(mut stream: Stream) -> Result<Self, Error> {
        let mut buffer = [0u8; (u64::BITS as usize/8) * 3]; // buffer for three `u64` values: `size`, `bounds.start`, `bounds.end`
        stream.read_exact(&mut buffer)?;


        // read metadata; return corruption error if failure
        let size = get_u64(&buffer, 0..8)?;
        let bounds = get_u64(&buffer, 8..16)?..get_u64(&buffer, 16..24)?;

        Ok(Self {
            bounds: Some(bounds),
            mapper: Mapper::Disk,
            size,
            read_cursor: (0, 0),
            stream,
        })
    }

    /// Checks for collisions on the current layer
    #[inline]
    pub fn check_collisions(&mut self, range: Range<u64>) -> Result<Box<[Range<u64>]>, Error> {
        let mut err = Ok(());
        let out = self.mapper.iter(&mut self.stream, self.size)
            .scan(&mut err, until_err)
            .filter(|(r, _)| range.start < r.end && r.start < range.end)
            .map(|(r, _)| range.start.max(r.start)..std::cmp::min(range.end, r.end))
            .collect();
        err?;
        Ok(out)
    }

    /// Takes in the output of the `check_collisions` function to find the inverse
    #[inline]
    pub fn check_non_collisions(&self, range: Range<u64>, collisions: &[Range<u64>]) -> Box<[Range<u64>]> { // find a bettr purely functional solution
        let mut current_end = range.start;
        let mut output = Vec::new();

        for r in collisions.iter() {
            output.push(current_end..r.start);
            current_end = r.start;
        } output.push(current_end..range.end);

        output.into_boxed_slice()
    }

    /// Writes to the heap layer without checking for collisions
    ///
    /// **WARNING:** the layer will be corrupt if there are any collisions; this function is meant to be used internally
    #[inline]
    pub fn write_unchecked(&mut self, idx: u64, data: Cow<'l, [u8]>) -> Result<(), Error> {
        // cannot write on read-only
        let (mapper, write_cursor) = self.mapper.get_writer()?;
        let range = idx..idx+data.len() as u64;

        // get the idx ni the map to insert to
        let map_idx = if write_cursor.0 == idx {
            write_cursor.1
        } else {
            mapper
                .iter()
                .enumerate()
                .find(|(_, (r, _))| r.start > idx)
                .map(|(i, _)| i)
                .unwrap_or(0) // if map is empty write to the first index
        };

        // insert data into the map and update write cursor & size
        mapper.insert(map_idx, (range.clone(), data));
        *write_cursor = (range.end, map_idx+1);
        self.size += range.end - range.start;

        // Update bounds
        self.bounds = Some(match self.bounds {
            Some(ref x) => std::cmp::min(x.start, range.start)..std::cmp::max(x.end, range.end),
            None => range,
        });

        Ok(())
    }

    /// Moves the layer from the **heap** to **disk**
    pub fn flush(self) -> Result<(), Error> {
        const BUFFER_SIZE: usize = 1024 * 1024 * 4; // 4MiB buffer size
        
        // don't flush if it's an empty layer or in read-only mode
        let (bounds, mapper) = if let (Some(b), Mapper::Heap { mapper, .. }) = (self.bounds, self.mapper) { (b, mapper) } else {  return Ok(()) };
        let mut file = BufWriter::with_capacity(BUFFER_SIZE, self.stream);

        // write the bounds & size of the layer
        file.write_all(&self.size.to_be_bytes())?;
        file.write_all(&bounds.start.to_be_bytes())?;
        file.write_all(&bounds.end.to_be_bytes())?;

        // we assume that the map is already sorted
        for (range, data) in mapper {
            file.write_all(&range.start.to_be_bytes())?;
            file.write_all(&range.end.to_be_bytes())?;
            file.write_all(&data)?;
        }

        file.flush()?;
        Ok(())
    }
}
