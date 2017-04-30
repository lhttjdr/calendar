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
        loaders: [{
            test: /\.js$/,
            exclude: /(node_modules|bower_components)/,
            loader: "babel-loader",
            query: {
                presets: ['es2015'],
                stage: 0,
                retainLines: true,
                cacheDirectory: true
            }
        }, {
            test: /\.js$/,
            exclude: /(node_modules|bower_components)/,
            loader: 'transform/cacheable?brfs',
            cacheable: true
        }]
    }
};