//! The user-facing interface for interacting with multiple layers at once

use std::{borrow::Cow, ops::Range};
use crate::errors::Error;
use self::allocator::Allocator;
use super::layer::Layer;
pub mod allocator;

#[derive(Debug)]
pub struct StackDB<'l, A: Allocator<'l>> {
    /// The layer allocator for the database
    alloc: A,
    /// If there is a heap layer or not
    heap_layer: bool,
    /// The actual layers in the database
    layers: Vec<Layer<'l, A::LayerStream>>,
}

impl<'l, A: Allocator<'l>> StackDB<'l, A> {
    /// creates a database interface; either loads an existing db or creates a new one.
    #[inline]
    pub fn new(alloc: A) -> Self {
        Self {
            alloc,
            heap_layer: false,
            layers: Vec::new(),
        }
    }

    /// Either grabs the heap layer or creates a new one
    #[inline]
    fn get_heap_layer(&mut self) -> Result<&mut Layer<'l, A::LayerStream>, Error> {
        if self.heap_layer {
            return Ok(self.layers.last_mut().unwrap());
        }

        self.layers.push(self.alloc.add_layer()?);
        self.heap_layer = true;
        self.get_heap_layer()
    }

    /// Reads data from either the heap or disk layers
    #[inline]
    pub fn read(&mut self, addr: Range<u64>) -> Result<Box<[u8]>, Error> {
        let mut data = vec![0u8; (addr.end-addr.start) as usize].into_boxed_slice();
        let mut missing: Vec<Range<u64>> = vec![addr.clone()]; // data that hasn't been read yet

        #[inline]
        fn write_into(data: &[u8], out: &mut [u8]) {
            data.iter()
                .enumerate()
                .for_each(|(i, b)| out[i] = *b);
        }

        for layer in self.layers.iter_mut().rev() {
            if missing.is_empty() { break };
            let mut collisions = Vec::new();
            let mut non_collisions = Vec::new();

            // find the parts of the range that belong to the layer's sections
            for miss in missing.iter() {
                collisions.append(&mut layer.check_collisions(miss)?.into_vec());
                non_collisions.append(&mut layer.check_non_collisions(miss, &collisions).into_vec());
            } missing = non_collisions;

            // actually read the values
            for range in collisions.iter() {
                let read = layer.read_unchecked(range)?;
                write_into(&read.1[read.0], &mut data[(range.start-addr.start) as usize..(range.end-addr.start) as usize]);
            }
        }

        // if !missing.is_empty() { return Err(Error::OutOfBounds) } // note: otherwise it will just return 0s for the areas not covered by layers

        Ok(data)
    }

    /// Rebases and drops overwritten layers (the database history)
    /// by compressing all the layers into one to save space
    ///
    /// **Warning:** will temporarity double database size
    #[inline]
    pub fn rebase(&mut self, buffer_size: u64) -> Result<(), Error> {
        if self.layers.is_empty() || self.layers.last().unwrap().bounds.is_none() { return Ok(()) }; // do nothing if database is empty
        self.flush()?;
        let old_layers = self.layers.len();

        let db_bounds = self.layers.iter()
            .filter_map(|x| x.bounds.as_ref())
            .fold((u64::MAX, u64::MIN), |x, y| (std::cmp::min(x.0, y.start), std::cmp::max(x.1, y.end)));
        let db_bounds = db_bounds.0..db_bounds.1;
        
        // Write all the changes into the top layer
        let mut idx = db_bounds.start;
        while idx < db_bounds.end {
            let end = std::cmp::min(db_bounds.end, idx+buffer_size);
            let buffer = self.read(idx..end)?;
            self.write(idx, &buffer)?;
            self.flush()?; // as to not bomb your memory
            idx = end;
        }

        // Drop all the other layers
        self.alloc.rebase(old_layers)?;
        let mut layers = Vec::with_capacity(self.layers.len()-old_layers);
        layers.extend(self.layers.drain(old_layers..));
        self.layers = layers;

        Ok(())
    }

    /// Writes data to the heap layer (collisions are fine) (`flush` to commit the heap layers to disk)
    #[inline]
    pub fn write(&mut self, addr: u64, data: &[u8]) -> Result<(), Error> {
        let layer = self.get_heap_layer()?;
        let range = addr..addr + data.len() as u64;
        let collisions = layer.check_collisions(&range)?;

        let non_collisions = layer.check_non_collisions(&range, &collisions);
        for r in non_collisions.iter() {
            let r_normal = (r.start-addr)as usize..(r.end-addr)as usize;
            let mut data = data[r_normal].to_vec();
            data.shrink_to_fit();

            layer.write_unchecked(r.start, Cow::Owned(data))?;
        }

        // if there are collisions while writing; write them to a new layer
        if !collisions.is_empty() {
            self.flush()?;
            for r in collisions.iter() {
                let r_normal = (r.start-addr)as usize..(r.end-addr)as usize;
                self.write(r.start, &data[r_normal])?; // you can make this more efficient by manually rewriting it
            }
        }

        Ok(())
    }

    /// Commits / writes the read-write layer's (on the heap) writes to the database (on the disk)
    #[inline]
    pub fn flush(&mut self) -> Result<(), Error> {
        if !self.heap_layer { return Ok(()) };

        let layer = self.layers.last_mut().unwrap();
        // Don't flush if layer is empty
        if layer.bounds.is_none() { return Ok(()) };
        layer.flush()?;
        self.heap_layer = false;

        Ok(())
    }
}
