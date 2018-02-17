use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use bincode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadError {
    IoError,
    NoSuchKey,
    InvalidFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreError {
    IoError,
}

pub trait Storage {
    fn load_raw<K>(&self, key: K) -> Result<Vec<u8>, LoadError>
    where
        K: AsRef<str>;

    fn store_raw<K, V>(&mut self, key: K, value: V) -> Result<(), StoreError>
    where
        K: AsRef<str>,
        V: AsRef<[u8]>;

    fn remove_raw<K>(&mut self, key: K) -> Result<Vec<u8>, LoadError>
    where
        K: AsRef<str>;

    fn exists<K>(&self, key: K) -> bool
    where
        K: AsRef<str>;

    fn clear(&mut self);

    fn load<K, T>(&self, key: K) -> Result<T, LoadError>
    where
        K: AsRef<str>,
        T: DeserializeOwned,
    {
        bincode::deserialize(self.load_raw(key)?.as_slice()).map_err(|_| LoadError::InvalidFormat)
    }

    fn store<K, T>(&mut self, key: K, value: &T) -> Result<(), StoreError>
    where
        K: AsRef<str>,
        T: Serialize,
    {
        let bytes = bincode::serialize(value).expect("Failed to serialize data");
        self.store_raw(key, bytes)
    }

    fn remove<K, T>(&mut self, key: K) -> Result<T, LoadError>
    where
        K: AsRef<str>,
        T: DeserializeOwned,
    {
        let bytes = self.remove_raw(key)?;
        bincode::deserialize(&bytes).map_err(|_| LoadError::InvalidFormat)
    }
}
