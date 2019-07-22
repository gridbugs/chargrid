const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = (env, argv) => {
  return {
    entry: './index.js',
      output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'index.js',
        webassemblyModuleFilename: "app.wasm",
      },
      plugins: [
        new HtmlWebpackPlugin({
          template: './index.html'
        }),
        new WasmPackPlugin({
          crateDirectory: path.resolve(__dirname, ".")
        }),
        // Required to work in Edge
        new webpack.ProvidePlugin({
          TextDecoder: ['text-encoding', 'TextDecoder'],
          TextEncoder: ['text-encoding', 'TextEncoder']
        })
      ],
      mode: argv.mode,
  }
};
