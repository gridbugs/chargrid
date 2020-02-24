const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const CopyWebpackPlugin = require('copy-webpack-plugin');
const util = require('util');
const exec = util.promisify(require('child_process').exec);

module.exports = async (env, argv) => {
  let revision_output = await exec("git rev-parse HEAD");
  let revision = revision_output.stdout.trim();
  return {
    entry: './index.js',
    output: {
      path: path.resolve(__dirname, 'dist'),
      filename: `index.${revision}.js`,
      webassemblyModuleFilename: `app.${revision}.wasm`,
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
      }),
      new CopyWebpackPlugin([{ from: "static_web" }]),
    ],
    devServer: {
      disableHostCheck: true,
    }
  }
};
