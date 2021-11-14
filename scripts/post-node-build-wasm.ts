import * as fs from 'node:fs/promises'
import { updateManifest } from './update-manifest'

Promise.all([
  fs.writeFile(
    './pkg/node/index.js',
    `module.exports = require('./browserslist').resolveToStrings`
  ),
  updateManifest('./pkg/node/package.json'),
])
