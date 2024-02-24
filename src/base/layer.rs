use std::{borrow::Cow, io::{BufWriter, Read, Seek, Write}, ops::Range};
use crate::errors::Error;

// Holds a range that may be partially completed
pub struct ParitialRange(pub Box<[Range<u64>]>);

/// Holds specific values for the writing function of the layer
#[derive(Debug)]
struct MemWriter<'l> {
    /// The current write cursor to speed up sequential qrites
    pub write_cursor: (u64, usize),
    /// The map that holds all the writes to the layer and their location mapping in the database
    pub map: Vec<(Range<u64>, Cow<'l, [u8]>)>, // change this to a linked list if too slow
}

/// Represents a layer (either in-memory or in disk) in the stack-db that *stacks*
#[derive(Debug)]
pub struct Layer<'l, File: Write + Read + Seek> {
    /// The bounds of the layer; the range of the layer
    pub bounds: Option<Range<u64>>,
    /// Optional writer (for mem-layers only as disk layers are read-only)
    ///
    /// - Also indicates if this is a mem-layer or disk-layer
    writer: Option<MemWriter<'l>>,
    /// The total size of all the writes in the layer (limited to `4GiB` currently)
    pub size: u64,
    /// The current read cursor to speed up sequential reads
    pub read_cursor: (u64, usize),
    /// The underlying file reader/writer
    file: File,
}

impl<'l,  File: Write + Read + Seek> Layer<'l, File> {
    #[inline]
    pub fn new(file: File) -> Self {
        Self {
            bounds: None,
            writer: Some(MemWriter {
                write_cursor: (0, 0),
                map: Vec::new(),
            }),
            size: 0,
            read_cursor: (0, 0),
            file,
        }
    }

    /// Checks for collisions on the current layer
    #[inline]
    pub fn check_collisions(&self, range: Range<u64>) -> ParitialRange {
        let map = if let Some(ref writer) = self.writer { &writer.map } else { panic!("will implement disk layers and disk reads later") };

        let ranges: Box<[_]> = map.iter() // I have no clue how this works
            .filter(|(r, _)| range.start < r.end && r.start < range.end)
            .map(|(r, _)| range.start.max(r.start)..std::cmp::min(range.end, r.end))
            .collect();
        ParitialRange(ranges)
    }

    /// Writes to the mem-layer without checking for collisions
    ///
    /// **WARNING:** the layer will be corrupt if there are any collisions; this function is meant to be used internally
    #[inline]
    pub fn write_unchecked(&mut self, idx: u64, data: Cow<'l, [u8]>) -> Result<(), Error> {
        // cannot write on read-only
        let writer = if let Some(ref mut writer) = self.writer { writer } else { return Err(Error::ReadOnly) };
        let range = idx..idx+data.len() as u64;

        // get the idx ni the map to insert to
        let map_idx = if writer.write_cursor.0 == idx {
            writer.write_cursor.1
        } else {
            writer.map
                .iter()
                .enumerate()
                .find(|(_, (r, _))| r.start > idx)
                .map(|(i, _)| i)
                .unwrap_or(0) // if map is empty write to the first index
        };

        // insert data into the map and update write cursor & size
        writer.map.insert(map_idx, (range.clone(), data));
        writer.write_cursor = (range.end, map_idx+1);
        self.size += range.end - range.start;

        // Update bounds
        self.bounds = Some(match self.bounds {
            Some(ref x) => std::cmp::min(x.start, range.start)..std::cmp::max(x.end, range.end),
            None => range,
        });

        Ok(())
    }

    /// Moves the laer from memory to disk
    #[inline]
    pub fn flush(self) -> Result<(), Error> {
        const BUFFER_SIZE: usize = 1024 * 1024 * 4; // 4MiB buffer size
        
        // don't flush if it's an empty layer or in read-only mode
        let (bounds, map) = if let (Some(b), Some(w)) = (self.bounds, self.writer) { (b, w.map) } else {  return Ok(()) };
        let mut file = BufWriter::with_capacity(BUFFER_SIZE, self.file);

        // write the bounds & size of the layer
        file.write_all(&self.size.to_be_bytes())?;
        file.write_all(&bounds.start.to_be_bytes())?;
        file.write_all(&bounds.end.to_be_bytes())?;

        // we assume that the map is already sorted
        for (range, data) in map {
            file.write_all(&range.start.to_be_bytes())?;
            file.write_all(&range.end.to_be_bytes())?;
            file.write_all(&data)?;
        }

        file.flush()?;
        Ok(())
    }
}
