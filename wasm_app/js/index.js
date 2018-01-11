import $ from 'jquery';
import prototty from 'prototty-terminal-js';

$(() => {
    console.log("Loading...");
    prototty.runProtottyApp("wasm_app.wasm", 20, 20, protottyTerminal).then(() => {
        console.log("Done!");
    });
});
