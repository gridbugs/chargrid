extern crate bincode;
extern crate prototty_storage;
extern crate serde;

pub use prototty_storage::{LoadError, Storage, StoreError};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::collections::HashMap;

pub trait StoreBytes {
    fn store(&mut self, bytes: &[u8]);
}

pub struct MonoStorage<S: StoreBytes> {
    table: HashMap<String, Vec<u8>>,
    store_bytes: S,
}

impl<S: StoreBytes> MonoStorage<S> {
    pub fn new(store_bytes: S) -> Self {
        let table = HashMap::new();
        Self { table, store_bytes }
    }
    pub fn from_bytes(bytes: &[u8], store_bytes: S) -> Option<Self> {
        bincode::deserialize(bytes)
            .ok()
            .map(|table| Self { table, store_bytes })
    }
    pub fn from_bytes_or_empty(bytes: &[u8], store_bytes: S) -> Self {
        match bincode::deserialize(bytes).ok() {
            Some(table) => Self { table, store_bytes },
            None => Self::new(store_bytes),
        }
    }
    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self.table).expect("Failed to serialize")
    }
    fn sync(&mut self) {
        let bytes = self.to_bytes();
        self.store_bytes.store(&bytes);
    }
}

impl<S: StoreBytes> Storage for MonoStorage<S> {
    fn load<K, T>(&self, key: K) -> Result<T, LoadError>
    where
        K: AsRef<str>,
        T: DeserializeOwned,
    {
        bincode::deserialize(self.load_raw(key)?.as_slice()).map_err(|_| LoadError::InvalidFormat)
    }

    fn remove<K, T>(&mut self, key: K) -> Result<T, LoadError>
    where
        K: AsRef<str>,
        T: DeserializeOwned,
    {
        let bytes = self.remove_raw(key)?;
        bincode::deserialize(&bytes).map_err(|_| LoadError::InvalidFormat)
    }

    fn load_raw<K>(&self, key: K) -> Result<Vec<u8>, LoadError>
    where
        K: AsRef<str>,
    {
        self.table
            .get(key.as_ref())
            .cloned()
            .ok_or(LoadError::NoSuchKey)
    }

    fn store_raw<K, V>(&mut self, key: K, value: V) -> Result<(), StoreError>
    where
        K: AsRef<str>,
        V: AsRef<[u8]>,
    {
        {
            let slice = value.as_ref();
            let buf = self
                .table
                .entry(key.as_ref().to_string())
                .or_insert_with(Vec::new);
            buf.resize(slice.len(), 0);
            buf.copy_from_slice(slice);
        }

        self.sync();

        Ok(())
    }

    fn remove_raw<K>(&mut self, key: K) -> Result<Vec<u8>, LoadError>
    where
        K: AsRef<str>,
    {
        let ret = self.table.remove(key.as_ref()).ok_or(LoadError::NoSuchKey);

        self.sync();

        ret
    }

    fn exists<K>(&self, key: K) -> bool
    where
        K: AsRef<str>,
    {
        self.table.contains_key(key.as_ref())
    }

    fn clear(&mut self) {
        self.table.clear();
        self.sync();
    }

    fn store<K, T>(&mut self, key: K, value: &T) -> Result<(), StoreError>
    where
        K: AsRef<str>,
        T: Serialize,
    {
        let bytes = bincode::serialize(value).expect("Failed to serialize data");

        self.store_raw(key, bytes)?;

        Ok(())
    }
}
