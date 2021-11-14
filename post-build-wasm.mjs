import * as fs from 'node:fs/promises'

Promise.all([
  fs.writeFile(
    './pkg/index.js',
    `module.exports = require('./browserslist').resolveToStrings`
  ),
  (async () => {
    const manifestPath = './pkg/package.json'
    const pkg = JSON.parse(await fs.readFile(manifestPath, 'utf8'))

    pkg.author = pkg.collaborators[0]
    pkg.files.push('index.js')
    pkg.main = 'index.js'

    await fs.writeFile(manifestPath, JSON.stringify(pkg, null, 2))
  })(),
])
