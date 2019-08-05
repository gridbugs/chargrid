import { JsByteStorage} from 'prototty_storage';
import('./pkg/fib_wasm').catch(console.error).then(wasm => {
  let storage = new JsByteStorage("app");
  window.reset = function() {
    storage.js_clear();
  };
  wasm.run(storage);
});
