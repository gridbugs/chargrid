const path = require('path');
const webpack = require('webpack');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const CopyWebpackPlugin = require('copy-webpack-plugin');

module.exports = (env, argv) => {
  return {
    mode: 'development',
    entry: {
      main: './index.js',
    },
    output: {
      path: path.resolve(__dirname, 'dist'),
      filename: 'index.js',
      webassemblyModuleFilename: "app.wasm",
    },
    plugins: [
      new HtmlWebpackPlugin({
        template: './index.html',
      }),
      new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, "."),
        extraArgs: "--no-typescript",
      }),
      // Required to work in Edge
      new webpack.ProvidePlugin({
        TextDecoder: ['text-encoding', 'TextDecoder'],
        TextEncoder: ['text-encoding', 'TextEncoder']
      }),
      new CopyWebpackPlugin({
        patterns: [
          { from: 'static_web' },
        ],
      }),
    ],
    experiments: {
      asyncWebAssembly: true,
    },
  };
};
