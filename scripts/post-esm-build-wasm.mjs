import * as fs from 'node:fs/promises'

Promise.all([
  fs.writeFile(
    './pkg/esm/index.js',
    `export * from './browserslist.js';\nexport { resolveToStrings as default } from './browserslist.js';`
  ),
  (async () => {
    const manifestPath = './pkg/esm/package.json'
    const pkg = JSON.parse(await fs.readFile(manifestPath, 'utf8'))

    pkg.author = pkg.collaborators[0]
    pkg.files.push('index.js')
    pkg.main = 'index.js'
    pkg.module = 'index.js'

    await fs.writeFile(manifestPath, JSON.stringify(pkg, null, 2))
  })(),
])
