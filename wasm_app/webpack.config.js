const UglifyJsPlugin = require('uglifyjs-webpack-plugin');

module.exports = {
  entry: './js/index.js',
  devtool: 'source-map',
  output: {
    filename: 'dist/bundle.js'
  },
  plugins: [
    new UglifyJsPlugin({ sourceMap: true })
  ],
};
