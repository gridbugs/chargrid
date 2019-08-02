extern crate prototty_storage;
extern crate serde_json;

pub use prototty_storage::*;
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
        serde_json::from_slice(bytes)
            .ok()
            .map(|table| Self { table, store_bytes })
    }
    pub fn from_bytes_or_empty(bytes: &[u8], store_bytes: S) -> Self {
        match serde_json::from_slice(bytes).ok() {
            Some(table) => Self { table, store_bytes },
            None => Self::new(store_bytes),
        }
    }
    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self.table).expect("Failed to serialize")
    }
    fn sync(&mut self) {
        let bytes = self.to_bytes();
        self.store_bytes.store(&bytes);
    }
}

impl<S: StoreBytes> Storage for MonoStorage<S> {
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

    fn remove<K>(&mut self, key: K) -> Result<(), RemoveError>
    where
        K: AsRef<str>,
    {
        self.table.remove(key.as_ref()).ok_or(RemoveError::NoSuchKey)?;
        self.sync();
        Ok(())
    }

    fn load_raw<K>(&self, key: K) -> Result<Vec<u8>, LoadRawError>
    where
        K: AsRef<str>,
    {
        self.table.get(key.as_ref()).cloned().ok_or(LoadRawError::NoSuchKey)
    }

    fn store_raw<K, V>(&mut self, key: K, value: V) -> Result<(), StoreRawError>
    where
        K: AsRef<str>,
        V: AsRef<[u8]>,
    {
        let slice = value.as_ref();
        let buf = self.table.entry(key.as_ref().to_string()).or_insert_with(Vec::new);
        buf.resize(slice.len(), 0);
        buf.copy_from_slice(slice);
        self.sync();
        Ok(())
    }
}
