"use strict";

import localforage from 'localforage';

export class JsByteStorage {
    js_load() {
        return this.data;
    }
    js_clear() {
        this.store(new Uint8Array());
    }
    js_store(data) {
        this.data = data;
        return localforage.setItem(this.key, this.data);
    }
}
JsByteStorage.make_async = async function(key) {
    let data = await localforage.getItem(key);
    if (data === null || data.constructor !== Uint8Array) {
        data = new Uint8Array();
    }
    let storage = new JsByteStorage();
    storage.key = key;
    storage.data = data;
    return storage;
}
