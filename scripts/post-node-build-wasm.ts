import * as fs from 'node:fs/promises'

Promise.all([
  fs.writeFile(
    './pkg/node/index.js',
    `module.exports = require('./browserslist').execute`
  ),
  (async () => {
    const path = './pkg/node/package.json'
    const pkg = JSON.parse(await fs.readFile(path, 'utf8'))

    pkg.author = pkg.collaborators[0]
    pkg.files.push('index.js')
    pkg.main = 'index.js'
    pkg.types = 'index.d.ts'

    await fs.writeFile(path, JSON.stringify(pkg, null, 2))
  })(),
  fs.copyFile('./scripts/pkg-types.d.ts', './pkg/node/types.d.ts'),
  fs.writeFile(
    './pkg/node/index.d.ts',
    `declare function execute(query: string, opts?: import('./types').Opts): string[];\nexport = execute;`
  ),
])
