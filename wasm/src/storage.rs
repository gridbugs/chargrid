use std::slice;
use std::collections::BTreeMap;
use prototty::*;
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use bincode;

pub struct WasmStorage {
    pub table: BTreeMap<String, Vec<u8>>,
}

impl WasmStorage {
    pub fn new() -> Self {
        Self {
            table: BTreeMap::new(),
        }
    }
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        bincode::deserialize(bytes).ok().map(|table| Self { table })
    }
    pub unsafe fn from_ptr(ptr: *const u8, size: usize) -> Self {
        if size == 0 {
            return Self::new();
        }

        let slice = slice::from_raw_parts(ptr, size);

        Self::from_bytes(slice).unwrap_or_else(Self::new)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self.table, bincode::Infinite)
            .expect("Failed to serialize")
    }
}

impl Storage for WasmStorage {
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
        let buf = self.table.entry(key.as_ref().to_string()).or_insert_with(Vec::new);
        buf.resize(slice.len(), 0);
        buf.copy_from_slice(slice);
        Ok(())
    }

    fn load<K, T>(&self, key: K) -> Result<T, LoadError>
        where K: AsRef<str>,
              T: DeserializeOwned
    {
        bincode::deserialize(self.load_raw(key)?.as_slice())
            .map_err(|_| LoadError::InvalidFormat)
    }

    fn store<K, T>(&mut self, key: K, value: &T) -> Result<(), StoreError>
        where K: AsRef<str>,
              T: Serialize
    {
        let bytes = bincode::serialize(value, bincode::Infinite)
            .expect("Failed to serialize data");

        self.store_raw(key, bytes)?;

        let mut bytes = self.to_bytes();

        unsafe {
            ffi::store(bytes.as_mut_ptr(), bytes.len());
        }

        Ok(())
    }
}

mod ffi {
    extern "C" {
        pub fn store(buf: *mut u8, size: usize);
    }
}
