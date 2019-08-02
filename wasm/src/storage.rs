use prototty_monolithic_storage::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type JsByteStorage;

    #[wasm_bindgen(method)]
    fn js_load(this: &JsByteStorage) -> Vec<u8>;

    #[wasm_bindgen(method)]
    fn js_store(this: &JsByteStorage, data: &[u8]);
}

impl StoreBytes for JsByteStorage {
    fn store(&mut self, bytes: &[u8]) {
        self.js_store(bytes);
    }
}

pub struct WasmStorage(MonoStorage<JsByteStorage>);

impl WasmStorage {
    pub fn new(js_byte_storage: JsByteStorage) -> Self {
        let existing_data = js_byte_storage.js_load();
        WasmStorage(MonoStorage::from_bytes_or_empty(&existing_data, js_byte_storage))
    }
}

impl Storage for WasmStorage {
    fn exists<K>(&self, key: K) -> bool
    where
        K: AsRef<str>,
    {
        self.0.exists(key)
    }

    fn clear(&mut self) {
        self.0.clear()
    }

    fn remove<K>(&mut self, key: K) -> Result<(), RemoveError>
    where
        K: AsRef<str>,
    {
        self.0.remove(key)
    }

    fn load_raw<K>(&self, key: K) -> Result<Vec<u8>, LoadRawError>
    where
        K: AsRef<str>,
    {
        self.0.load_raw(key)
    }

    fn store_raw<K, V>(&mut self, key: K, value: V) -> Result<(), StoreRawError>
    where
        K: AsRef<str>,
        V: AsRef<[u8]>,
    {
        self.0.store_raw(key, value)
    }
}
