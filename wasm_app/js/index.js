import $ from 'jquery';
import prototty from 'prototty-terminal-js';

const env = {
    quit: () => console.log("not implemented"),
    cell_style: {
        "font-family": "PxPlus_IBM_CGAthin",
        "font-size": "16px",
        "line-height": "0px",
    },
    bold_style: {
        "font-family": "PxPlus_IBM_CGA",
    },
};

$(() => {
    console.log("Loading...");
    prototty.loadProtottyApp("wasm_app.wasm", 20, 20, protottyTerminal, env).then(app => {
        console.log("Done!");
        app.start();
    });
});
