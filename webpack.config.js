module.exports = {
  context: __dirname + '/src',
  entry: {
    'calendar': './Main.purs',
  },
  output: {
    path: __dirname + '/dist',
    filename: '[name].js'
  },
  module: {
    loaders: [
    {
      test: /\.purs$/,
      loader: 'purs-loader',
      exclude: /node_modules/,
      query: {
        psc: 'psa',
        src: ['bower_components/purescript-*/src/**/*.purs', 'src/**/*.purs']
      }
    }
    ]
  }
};