extern crate prototty;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate bincode;

use std::collections::HashMap;
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use prototty::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct MemoryStorage {
    table: HashMap<String, Vec<u8>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        bincode::deserialize(bytes).ok()
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self, bincode::Infinite)
            .expect("Failed to serialize")
    }
}

impl Storage for MemoryStorage {
    fn load_raw<K>(&self, key: K) -> Result<Vec<u8>, LoadError>
        where K: AsRef<str>
    {
        self.table.get(key.as_ref()).cloned().ok_or(LoadError::NoSuchKey)
    }


    fn store_raw<K, V>(&mut self, key: K, value: V) -> Result<(), StoreError>
        where K: AsRef<str>,
              V: AsRef<[u8]>
    {
        let slice = value.as_ref();
        let mut bytes = Vec::with_capacity(slice.len());
        bytes.resize(slice.len(), 0);
        bytes.copy_from_slice(slice);
        self.table.insert(key.as_ref().to_string(), bytes);
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


