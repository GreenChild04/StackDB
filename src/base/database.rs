//! The user-facing interface for interacting with multiple layers at once

use crate::errors::Error;

use self::allocator::Allocator;

use super::layer::Layer;
pub mod allocator;

#[derive(Debug)]
pub struct StackDB<'l, A: Allocator<'l>> {
    alloc: A,
    heap_layer: Option<&'l mut Layer<'l, A::LayerStream>>,
    layers: Vec<Layer<'l, A::LayerStream>>,
}

impl<'l, A: Allocator<'l>> StackDB<'l, A> {
    /// creates a database interface; either loads an existing db or creates a new one.
    #[inline]
    pub fn new(alloc: A) -> Self {
        Self {
            alloc,
            heap_layer: None,
            layers: Vec::new(),
        }
    }

    /// Commits / writes the read-write layer's (on the heap) writes to the database (on the disk)
    #[inline]
    pub fn flush(&mut self) -> Result<(), Error> {
        if let Some(layer) = self.heap_layer.take() {
            // Don't flush if layer is empty
            if layer.bounds.is_none() { return Ok(()) };
            layer.flush()?;
        } Ok(())
    }
}
