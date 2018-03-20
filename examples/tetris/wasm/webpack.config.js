const UglifyJsPlugin = require('uglifyjs-webpack-plugin');

module.exports = {
  mode: 'production',
  entry: './js/index.js',
  devtool: 'source-map',
  output: {
    filename: 'bundle.js'
  },
  plugins: [
    new UglifyJsPlugin({ sourceMap: true })
  ],
};
