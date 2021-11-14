import * as fs from 'node:fs/promises'
import { updateManifest } from './update-manifest'

Promise.all([
  fs.writeFile(
    './pkg/web/index.js',
    `export * from './browserslist.js';\nexport { resolveToStrings as default } from './browserslist.js';`
  ),
  updateManifest('./pkg/web/package.json'),
])
