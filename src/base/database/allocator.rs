//! Defines the Allocator trait for StackDB

use std::io::{Read, Seek, Write};
use crate::{base::layer::Layer, errors::Error};

/// The allocator for a StackDB that defines how or where the layers are stored and managed
pub trait Allocator<'l> {
    /// The type of data stream the layers read and write to
    type LayerStream: Write + Read + Seek;
    /// Loads all the read-only layers in the database as `Layers`
    fn load_layers(&self) -> Result<Vec<Layer<'l, Self::LayerStream>>, Error>;
    /// Adds a read-write layer to the database
    fn add_layer(&mut self) -> Result<Layer<'l, Self::LayerStream>, Error>;
    /// Removes the top layer from the database
    fn drop_top_layer(&mut self) -> Result<(), Error>;
    /// Removes all the bottom layers except for the one specified (and above)
    fn rebase(&mut self, top_layer: usize) -> Result<(), Error>;
 }
