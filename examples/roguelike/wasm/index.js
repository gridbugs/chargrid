import { JsByteStorage} from 'prototty_storage';
import('./pkg/roguelike_wasm').catch(console.error).then(async wasm => {
  let storage = await JsByteStorage.make_async("app");
  window.reset = function() {
    storage.js_clear();
  };
  wasm.run(storage);
});
