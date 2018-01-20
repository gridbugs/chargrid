import prototty from 'prototty-terminal-js';

let protottyTerminal = document.getElementById("protottyTerminal");

prototty.loadProtottyApp("title_wasm.wasm", 20, 20, protottyTerminal).then(app => {
    app.start();
});
