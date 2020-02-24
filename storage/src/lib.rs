pub use serde::de::DeserializeOwned;
pub use serde::ser::Serialize;

pub mod format;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoveError {
    IoError,
    NoSuchKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadRawError {
    IoError,
    NoSuchKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadError<E> {
    Raw(LoadRawError),
    FormatError(E),
}

#[derive(Debug)]
pub enum StoreRawError {
    IoError(Box<dyn std::error::Error>),
}

#[derive(Debug)]
pub enum StoreError<E> {
    Raw(StoreRawError),
    FormatError(E),
}

pub trait StorageFormat {
    type SerializeError;
    type DeserializeError;

    fn to_vec<T: ?Sized>(value: &T) -> Result<Vec<u8>, Self::SerializeError>
    where
        T: Serialize;

    fn from_slice<T>(v: &[u8]) -> Result<T, Self::DeserializeError>
    where
        T: DeserializeOwned;
}

pub trait Storage {
    fn exists<K>(&self, key: K) -> bool
    where
        K: AsRef<str>;

    fn clear(&mut self);

    fn remove<K>(&mut self, key: K) -> Result<(), RemoveError>
    where
        K: AsRef<str>;

    fn load_raw<K>(&self, key: K) -> Result<Vec<u8>, LoadRawError>
    where
        K: AsRef<str>;

    fn store_raw<K, V>(&mut self, key: K, value: V) -> Result<(), StoreRawError>
    where
        K: AsRef<str>,
        V: AsRef<[u8]>;

    fn load<K, T, F>(&self, key: K, format: F) -> Result<T, LoadError<F::DeserializeError>>
    where
        K: AsRef<str>,
        T: DeserializeOwned,
        F: StorageFormat,
    {
        let _ = format;
        self.load_raw(key)
            .map_err(LoadError::Raw)
            .and_then(|v| F::from_slice(&v).map_err(LoadError::FormatError))
    }

    fn store<K, T, F>(&mut self, key: K, value: &T, format: F) -> Result<(), StoreError<F::SerializeError>>
    where
        K: AsRef<str>,
        T: Serialize,
        F: StorageFormat,
    {
        let _ = format;
        F::to_vec(value)
            .map_err(StoreError::FormatError)
            .and_then(|v| self.store_raw(key, v).map_err(StoreError::Raw))
    }
}
