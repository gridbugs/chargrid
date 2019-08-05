'use strict';

function data_to_string(d) {
  return d.join(',');
}

function data_from_string(s) {
  return new Uint8Array(s.split(',').map(x => parseInt(x)));
}

export class JsByteStorage {
  constructor(key) {
    let data_string = localStorage.getItem(key);
    let data;
    if (data === null) {
      data = new Uint8Array();
    } else {
      data = data_from_string(data_string);
    }
    this.key = key;
    this.data = data;

  }
  js_load() {
    return this.data;
  }
  js_clear() {
    this.js_store(new Uint8Array())
  }
  js_store(data) {
    this.data = data;
    let data_string = data_to_string(this.data);
    localStorage.setItem(this.key, data_string);
  }
}
