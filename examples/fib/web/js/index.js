import { JsByteStorage} from 'prototty-wasm-storage-js';
const wasm = import('../wasm_out/web');

wasm.then(async wasm => {
    let storage = await JsByteStorage.make_async("fib");
    let app = new wasm.WebApp(storage);
    console.log(app.run());
    window.clearCounter = () => storage.clear();
});
