import * as fs from 'node:fs/promises'
import * as path from 'node:path'
import * as process from 'node:process'
import browserslist from 'browserslist'

const dest = path.join(process.cwd(), 'data')

Promise.all([
  fs.writeFile(
    path.join(dest, 'caniuse-lite-version-aliases.json'),
    JSON.stringify(browserslist.versionAliases)
  ),
])
