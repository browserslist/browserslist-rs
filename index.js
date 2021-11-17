const { loadBinding } = require('@node-rs/helper')

const bindings = loadBinding(__dirname, 'browserslist-rs', 'browserslist-rs')

module.exports = bindings.execute
