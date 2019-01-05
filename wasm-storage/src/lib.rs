extern crate prototty_mono_storage;
extern crate serde;
extern crate wasm_bindgen;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;

pub use prototty_mono_storage::{LoadError, Storage, StoreError};
use prototty_mono_storage::{MonoStorage, StoreBytes};

use wasm_bindgen::prelude::*;
#[wasm_bindgen(module = "prototty-wasm-storage-js")]
extern "C" {
    pub type JsByteStorage;

    #[wasm_bindgen(method)]
    fn load(this: &JsByteStorage) -> Vec<u8>;

    #[wasm_bindgen(method)]
    fn store(this: &JsByteStorage, data: &[u8]);
}

impl StoreBytes for JsByteStorage {
    fn store(&mut self, bytes: &[u8]) {
        JsByteStorage::store(self, bytes);
    }
}

pub struct WasmStorage(MonoStorage<JsByteStorage>);

impl WasmStorage {
    pub fn new(js_byte_storage: JsByteStorage) -> Self {
        let existing_data = js_byte_storage.load();
        WasmStorage(MonoStorage::from_bytes_or_empty(
            &existing_data,
            js_byte_storage,
        ))
    }
}

impl Storage for WasmStorage {
    fn load_raw<K>(&self, key: K) -> Result<Vec<u8>, LoadError>
    where
        K: AsRef<str>,
    {
        self.0.load_raw(key)
    }

    fn store_raw<K, V>(&mut self, key: K, value: V) -> Result<(), StoreError>
    where
        K: AsRef<str>,
        V: AsRef<[u8]>,
    {
        self.0.store_raw(key, value)
    }

    fn remove_raw<K>(&mut self, key: K) -> Result<Vec<u8>, LoadError>
    where
        K: AsRef<str>,
    {
        self.0.remove_raw(key)
    }

    fn exists<K>(&self, key: K) -> bool
    where
        K: AsRef<str>,
    {
        self.0.exists(key)
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn load<K, T>(&self, key: K) -> Result<T, LoadError>
    where
        K: AsRef<str>,
        T: DeserializeOwned,
    {
        self.0.load(key)
    }

    fn store<K, T>(&mut self, key: K, value: &T) -> Result<(), StoreError>
    where
        K: AsRef<str>,
        T: Serialize,
    {
        self.0.store(key, value)
    }

    fn remove<K, T>(&mut self, key: K) -> Result<T, LoadError>
    where
        K: AsRef<str>,
        T: DeserializeOwned,
    {
        self.0.remove(key)
    }
}
