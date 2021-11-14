import * as fs from 'node:fs/promises'

export async function updateManifest(path: string) {
  const pkg = JSON.parse(await fs.readFile(path, 'utf8'))

  pkg.author = pkg.collaborators[0]
  pkg.files.push('index.js')
  pkg.main = 'index.js'
  pkg.module = 'index.js'

  await fs.writeFile(path, JSON.stringify(pkg, null, 2))
}
