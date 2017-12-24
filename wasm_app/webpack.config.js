const UglifyJsPlugin = require('uglifyjs-webpack-plugin');

module.exports = {
  entry: './js/index.js',
  output: {
    filename: 'dist/bundle.js'
  },
  plugins: [
//    new UglifyJsPlugin()
  ],
};
