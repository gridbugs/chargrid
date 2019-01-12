import { JsByteStorage} from 'prototty_storage';
const wasm = import('../wasm_out/fib_wasm');

wasm.then(async wasm => {
    let storage = await JsByteStorage.make_async("fib");
    let app = new wasm.WebApp(storage);
    console.log(app.run());
    window.clearCounter = () => storage.clear();
});
