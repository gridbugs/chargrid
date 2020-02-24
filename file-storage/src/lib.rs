pub use prototty_storage::*;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub struct FileStorage {
    base_path: PathBuf,
}

pub enum IfDirectoryMissing {
    Create,
    Error,
}

#[derive(Debug)]
pub enum Error {
    FailedToCreateDirectory(io::Error),
    DirectoryMissing(PathBuf),
    FailedToFindCurrentExePath(io::Error),
}

impl FileStorage {
    pub fn new<P: AsRef<Path>>(path: P, if_directory_missing: IfDirectoryMissing) -> Result<Self, Error> {
        if !path.as_ref().exists() {
            match if_directory_missing {
                IfDirectoryMissing::Create => {
                    fs::create_dir_all(path.as_ref()).map_err(Error::FailedToCreateDirectory)?
                }
                IfDirectoryMissing::Error => return Err(Error::DirectoryMissing(path.as_ref().to_path_buf())),
            }
        }
        Ok(Self {
            base_path: path.as_ref().to_path_buf(),
        })
    }

    pub fn next_to_exe<P: AsRef<Path>>(
        relative_path: P,
        if_directory_missing: IfDirectoryMissing,
    ) -> Result<Self, Error> {
        let mut exe_path = env::current_exe().map_err(Error::FailedToFindCurrentExePath)?;
        // remove binary's name from the path
        exe_path.pop();
        Self::new(exe_path.join(relative_path), if_directory_missing)
    }

    pub fn full_path<S: AsRef<str>>(&self, path: S) -> PathBuf {
        let relative_path: PathBuf = path.as_ref().into();
        self.base_path.join(relative_path)
    }
}

impl Storage for FileStorage {
    fn exists<K>(&self, key: K) -> bool
    where
        K: AsRef<str>,
    {
        self.full_path(key).exists()
    }

    fn clear(&mut self) {
        fs::remove_dir_all(&self.base_path).expect("Failed to remove base dir");
        fs::create_dir_all(&self.base_path).expect("Failed to create base dir");
    }

    fn remove<K>(&mut self, key: K) -> Result<(), RemoveError>
    where
        K: AsRef<str>,
    {
        let path = self.full_path(key);
        fs::remove_file(path).map_err(|_| RemoveError::IoError)
    }

    fn load_raw<K>(&self, key: K) -> Result<Vec<u8>, LoadRawError>
    where
        K: AsRef<str>,
    {
        let mut file = File::open(self.full_path(key)).map_err(|_| LoadRawError::NoSuchKey)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).map_err(|_| LoadRawError::IoError)?;
        Ok(contents)
    }

    fn store_raw<K, V>(&mut self, key: K, value: V) -> Result<(), StoreRawError>
    where
        K: AsRef<str>,
        V: AsRef<[u8]>,
    {
        let mut file = File::create(self.full_path(key)).map_err(|e| StoreRawError::IoError(Box::new(e)))?;
        file.write_all(value.as_ref())
            .map_err(|e| StoreRawError::IoError(Box::new(e)))?;
        Ok(())
    }
}
