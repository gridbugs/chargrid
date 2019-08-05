import { JsByteStorage} from 'prototty_storage';
import('./pkg/roguelike_wasm').catch(console.error).then(wasm => {
  let storage = new JsByteStorage("app");
  window.reset = function() {
    storage.js_clear();
  };
  wasm.run(storage);
});
