const path = require('path');
const CopyWebpackPlugin = require('copy-webpack-plugin');

const WEBPACK_MODE = (typeof process.env.WEBPACK_MODE === 'undefined') ? "development" : process.env.WEBPACK_MODE;

module.exports = {
    mode: WEBPACK_MODE,
    entry: "./js/index.js",
    devtool: "source-map",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "bundle.js",
        webassemblyModuleFilename: "app.wasm",
    },
    plugins: [
        new CopyWebpackPlugin([{ from: "static_web" }]),
    ],
};
