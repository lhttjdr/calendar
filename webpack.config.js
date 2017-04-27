module.exports = {
  context: __dirname + '/src',
  entry: {
    'calendar': './calendar.js',
  },
  output: {
    path: __dirname + '/dist',
    filename: '[name].js'
  },
  module: {
    loaders: [
      { 
        test: /\.js$/, 
        exclude: /node_modules/, 
        loader: "babel-loader", 
        query:{
          presets: ['es2015']
        }
      }
    ]
  }
};