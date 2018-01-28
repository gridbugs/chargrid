extern crate prototty;
extern crate serde;
extern crate bincode;

use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use prototty::*;

pub struct FileStorage {
    base_path: PathBuf,
}

impl FileStorage {
    pub fn new<P: AsRef<Path>>(path: P, create: bool) -> Option<Self> {
        if !path.as_ref().exists() {
            if create {
                if fs::create_dir_all(path.as_ref()).is_err() {
                    return None;
                }
            } else {
                return None;
            }
        }

        Some(Self {
            base_path: path.as_ref().to_path_buf(),
        })
    }

    pub fn next_to_exe<P: AsRef<Path>>(relative_path: P, create: bool) -> Option<Self> {
        let mut exe_path = env::current_exe().ok()?;

        // remove binary's name from the path
        exe_path.pop();

        Self::new(exe_path.join(relative_path), create)
    }

    fn full_path<S: AsRef<str>>(&self, path: S) -> PathBuf {
        let relative_path: PathBuf = path.as_ref().into();
        self.base_path.join(relative_path)
    }
}

impl Storage for FileStorage {
    fn load_raw<K>(&self, key: K) -> Result<Vec<u8>, LoadError>
        where K: AsRef<str>
    {
        let mut file = File::open(self.full_path(key)).map_err(|_| LoadError::NoSuchKey)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).map_err(|_| LoadError::IoError)?;
        Ok(contents)
    }


    fn store_raw<K, V>(&mut self, key: K, value: V) -> Result<(), StoreError>
        where K: AsRef<str>,
              V: AsRef<[u8]>
    {
        let mut file = File::create(self.full_path(key)).map_err(|_| StoreError::IoError)?;
        file.write_all(value.as_ref()).map_err(|_| StoreError::IoError)?;
        Ok(())
    }


    fn load<K, T>(&self, key: K) -> Result<T, LoadError>
        where K: AsRef<str>,
              T: DeserializeOwned
    {
        let bytes = self.load_raw(key)?;
        bincode::deserialize(&bytes).map_err(|_| LoadError::InvalidFormat)
    }

    fn store<K, T>(&mut self, key: K, value: &T) -> Result<(), StoreError>
        where K: AsRef<str>,
              T: Serialize
    {
        let bytes = bincode::serialize(value, bincode::Infinite)
            .expect("Failed to serialize data");
        self.store_raw(key, bytes)
    }
}
