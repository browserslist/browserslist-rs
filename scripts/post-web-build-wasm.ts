import * as fs from 'node:fs/promises'

Promise.all([
  (async () => {
    const path = './pkg/web/package.json'
    const pkg = JSON.parse(await fs.readFile(path, 'utf8'))

    pkg.author = pkg.collaborators[0]

    await fs.writeFile(path, JSON.stringify(pkg, null, 2))
  })(),
  fs.copyFile('./scripts/pkg-types.d.ts', './pkg/web/types.d.ts'),
  (async () => {
    const path = './pkg/web/browserslist.d.ts'
    const dts = await fs.readFile(path, 'utf8')

    const updatedDts = dts.replace(
      `execute(query: string, opts: any): any;`,
      `execute(query: string, opts?: import('./types').Opts): string[];`
    )
    await fs.writeFile(path, updatedDts)
  })(),
])
