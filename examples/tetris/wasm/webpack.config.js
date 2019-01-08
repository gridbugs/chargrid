const path = require('path');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const WebpackShellPlugin = require('webpack-shell-plugin');

const RUST_MODE = (typeof process.env.RUST_MODE === 'undefined') ? "debug" : process.env.RUST_MODE;
const WEBPACK_MODE = RUST_MODE === "debug" ? "development" : "production";

module.exports = {
    mode: WEBPACK_MODE,
    entry: "./js/index.js",
    devtool: "source-map",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "bundle.js",
    },
    plugins: [
        new CopyWebpackPlugin([{ from: "static_web" }]),
        new WebpackShellPlugin({ onBuildStart: [`./build_wasm.sh ${RUST_MODE}`]}),
    ],
};
