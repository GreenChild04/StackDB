//! Some default `stack-db` allocator implementations

use std::{fs::{self, File}, io::Cursor, path::{Path, PathBuf}};
use crate::{base::{database::allocator::Allocator, layer::Layer}, errors::Error};

/// # In-Memory Allocator
/// ---
/// For a redis-like database for caching, testing the database, etc. Lives only on the heap and gets wiped on program exit.
pub struct SkdbMemAlloc;
impl<'a> Allocator<'a> for SkdbMemAlloc {
    type LayerStream = Cursor<Vec<u8>>;
    #[inline]
    fn load_layers(&self) -> Result<Vec<Layer<'a, Self::LayerStream>>, Error> {
        Ok(Vec::new())
    }
    #[inline]
    fn add_layer(&mut self) -> Result<Layer<'a, Self::LayerStream>, Error> {
        Ok(Layer::new(Cursor::new(Vec::new())))
    }
    #[inline]
    fn drop_top_layer(&mut self) -> Result<(), Error> {
        Ok(())
    }
    #[inline]
    fn rebase(&mut self, _: usize) -> Result<(), Error> {
        Ok(())
    }
}

/// # Directory Allocator
/// ---
/// Allocates within a directory that lives on the file-system with the layer order determined by the layer file names
pub struct SkdbDirAlloc {
    /// the path of the directory database
    pub path: PathBuf,
    /// the (sorted) paths of the layers in the database
    pub layers: Vec<PathBuf>,
    pub cursor: u32,
}
impl SkdbDirAlloc {
    /// Creates a new SkDB
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();

        fs::create_dir_all(path)?;
        Ok(Self {
            path: path.to_path_buf(),
            layers: Vec::new(),
            cursor: 0,
        })
    }
    
    /// Loads a Skdb from a directory
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        // grab file paths
        let mut file_paths = Vec::new();
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                file_paths.push(entry.path());
            }
        }

        // index files and sort them
        let mut layers = file_paths.iter()
            .enumerate()
            .filter_map(|(i, file)| file.file_name().map(|x| (x.to_string_lossy(), i)))
            .filter_map(|(name, i)| name.parse::<u32>().ok().map(|x| (x, i)))
            .collect::<Vec<(u32, usize)>>();
        layers.sort_unstable_by_key(|x| x.0);

        let cursor = layers.last().map(|x| x.0 + 1).unwrap_or(0);
        let layers = layers.into_iter()
            .map(|(_, i)| std::mem::take(&mut file_paths[i]))
            .collect::<Vec<_>>();

        // return self
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            layers,
            cursor,
        })
    }
}
impl<'a> Allocator<'a> for SkdbDirAlloc {
    type LayerStream = File;

    /// Loads the layer files from the directory
    fn load_layers(&self) -> Result<Vec<Layer<'a, Self::LayerStream>>, Error> {
        let mut layers = Vec::with_capacity(self.layers.len());
        for path in self.layers.iter() {
            let file = File::options()
                .read(true)
                .write(true)
                .append(false)
                .truncate(false)
                .open(path)?;
            layers.push(Layer::load(file)?);
        } Ok(layers)
    }

    fn add_layer(&mut self) -> Result<Layer<'a, Self::LayerStream>, Error> {
        let path = self.path.join(self.cursor.to_string());
        let file = File::options()
            .read(true)
            .write(true)
            .append(false)
            .truncate(false)
            .create_new(true)
            .open(&path)?;
        self.cursor += 1;
        self.layers.push(path);
        Ok(Layer::new(file))
    }

    fn drop_top_layer(&mut self) -> Result<(), Error> {
        let path = if let Some(x) = self.layers.pop() { x } else { return Ok(()) };
        self.cursor -= 1;
        fs::remove_file(path)?;

        Ok(())
    }

    fn rebase(&mut self, top_layer: usize) -> Result<(), Error> {
        let mut top = Vec::with_capacity(self.layers.len()-top_layer);
        top.extend(self.layers.drain(top_layer..));

        // delete the other layer files
        for path in self.layers.iter() {
            fs::remove_file(path)?;
        }

        // move the base layer
        self.layers.clear();
        for (i, path) in top.into_iter().enumerate() {
            let new_path = self.path.join(i.to_string());
            fs::rename(path, &new_path)?;
            self.layers.push(new_path);
        } Ok(())
    }
}
