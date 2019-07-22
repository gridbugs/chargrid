import { JsByteStorage} from 'prototty_storage';
import('./pkg/fib_wasm').catch(console.error).then(async wasm => {
  let storage = await JsByteStorage.make_async("app");
  wasm.run(storage);
});

