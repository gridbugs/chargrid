use std::collections::BTreeMap;
use prototty::*;
use prototty_memory_storage::*;
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use bincode;

#[derive(Clone, Serialize, Deserialize)]
pub struct WasmStorage {
    table: BTreeMap<String, Vec<u8>>,
}

impl WasmStorage {
    pub fn new() -> Self {
        Self {
            table: BTreeMap::new(),
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
        let mut buf = self.table.entry(key.as_ref().to_string()).or_insert_with(Vec::new);
        buf.resize(slice.len(), 0);
        buf.copy_from_slice(slice);
        Ok(())
    }


    fn load<K, T>(&self, key: K) -> Result<T, LoadError>
        where K: AsRef<str>,
              T: DeserializeOwned
    {
//        let mut bytes = self.load_raw(key)?;
        if let Some(bytes) = self.table.get(key.as_ref()) {
            unsafe {
                ffi::foo(1111);
                ffi::foo(bytes.as_ptr() as u32);
                ffi::foo(bytes.len() as u32);
            }

            bincode::deserialize(&bytes).map_err(|_| LoadError::InvalidFormat)
        } else {
            Err(LoadError::NoSuchKey)
        }
    }

    fn store<K, T>(&mut self, key: K, value: &T) -> Result<(), StoreError>
        where K: AsRef<str>,
              T: Serialize
    {
        let mut bytes = bincode::serialize(value, bincode::Infinite)
            .expect("Failed to serialize data");

        let ptr = bytes.as_mut_ptr();
        let size = bytes.len();

        self.store_raw(key, bytes);

        let mut bytes = self.to_bytes();
        unsafe {
            ffi::store(ptr, size);
            ffi::foo(33333);
            ffi::foo(size as u32);
            ffi::foo(ptr as u32);
            ffi::foo(4444);
        }

        Ok(())
    }
}

mod ffi {
    extern "C" {
        pub fn store(buf: *mut u8, size: usize);
        pub fn foo(x: u32);
    }
}
