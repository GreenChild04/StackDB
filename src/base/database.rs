//! The user-facing interface for interacting with multiple layers at once

use std::borrow::Cow;
use crate::errors::Error;
use self::allocator::Allocator;
use super::layer::Layer;
pub mod allocator;

#[derive(Debug)] pub struct StackDB<'l, A: Allocator<'l>> {
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

    #[inline]
    fn get_heap_layer(&mut self) -> Result<&mut Layer<'l, A::LayerStream>, Error> {
        if self.heap_layer {
            return Ok(self.layers.last_mut().unwrap());
        }

        self.layers.push(self.alloc.add_layer()?);
        self.heap_layer = true;
        self.get_heap_layer()
    }

    #[inline]
    pub fn write(&mut self, addr: u64, data: Box<[u8]>) -> Result<(), Error> {
        let layer = self.get_heap_layer()?;
        let range = addr..addr + data.len() as u64;
        let collisions = layer.check_collisions(&range)?;

        let non_collisions = layer.check_non_collisions(&range, &collisions);
        for r in non_collisions.into_iter() {
            let r_normal = (r.start-addr)as usize..(r.end-addr)as usize;
            let mut data = data[r_normal].to_vec();
            data.shrink_to_fit();

            layer.write_unchecked(r.start, Cow::Owned(data))?;
        }

        // if there are collisions while writing; write them to a new layer
        if !collisions.is_empty() {
            self.flush()?;
            for r in collisions.into_iter() {
                let r_normal = (r.start-addr)as usize..(r.end-addr)as usize;
                self.write(r.start, data[r_normal].to_vec().into_boxed_slice())?; // you can make this more efficient by manually rewriting it
            }
        }

        Ok(())
    }

    /// Commits / writes the read-write layer's (on the heap) writes to the database (on the disk)
    #[inline]
    pub fn flush(&mut self) -> Result<(), Error> {
        if !self.heap_layer { return Ok(()) };

        let layer = self.layers.last_mut().take().unwrap();
        // Don't flush if layer is empty
        if layer.bounds.is_none() { return Ok(()) };
        layer.flush()?;

        Ok(())
    }
}
