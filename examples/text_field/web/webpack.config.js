const path = require('path');
const webpack = require('webpack');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const util = require('util');
const exec = util.promisify(require('child_process').exec);

module.exports = async (env, argv) => {
  const revision = (await exec('git rev-parse HEAD')).stdout.trim();
  return {
    entry: {
      main: './index.js',
    },
    output: {
      path: path.resolve(__dirname, 'dist'),
      // Various levels of caching on the web will mean that updates to wasm
      // and js files can take a long time to become visible. Prevent this by
      // including the revision hash of the repository in the names of these
      // files.
      filename: `index.${revision}.js`,
      webassemblyModuleFilename: `app.${revision}.wasm`,
    },
    plugins: [
      new HtmlWebpackPlugin({
        template: './index.html',
      }),
      new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, '.'),
        extraArgs: '--no-typescript',
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
    devServer: {
      client: {
        overlay: false,
      },
    },
  };
};
