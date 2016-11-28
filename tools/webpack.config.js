const resolvePath = require('path').resolve;

module.exports = {
  target: 'node',
  entry: resolvePath(__dirname, './compileJson.js'),
  output: {
    path: resolvePath(__dirname, './'),
    filename: 'pokemon.js',
    libraryTarget: 'commonjs2',
  },
  module: {
    loaders: [
      {
        test: /\.png$/,
        loaders: [
          'image-xterm-loader?cols=80',
        ],
      },
    ],
  },
  debug: true,
  stats: {
    color: true,
    reasons: true,
  },
};
