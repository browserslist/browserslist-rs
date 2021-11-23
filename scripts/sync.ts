import * as fs from 'node:fs/promises'
import * as path from 'node:path'
import * as process from 'node:process'
import browserslist from 'browserslist'
import nodeVersions from 'node-releases/data/processed/envs.json'
import nodeReleaseSchedule from 'node-releases/data/release-schedule/release-schedule.json'

const dest = path.join(process.cwd(), 'data')

Promise.all([
  fs.writeFile(
    path.join(dest, 'caniuse-lite-browsers.json'),
    JSON.stringify(browserslist.data)
  ),
  fs.writeFile(
    path.join(dest, 'caniuse-lite-usage.json'),
    JSON.stringify(
      Object.entries(browserslist.usage.global!)
        .map(([v, usage]) => {
          const [name, version] = v.split(' ')
          return [name, version, usage] as const
        })
        .sort(([, , a], [, , b]) => b - a)
    )
  ),
  fs.writeFile(
    path.join(dest, 'caniuse-lite-version-aliases.json'),
    JSON.stringify(browserslist.versionAliases)
  ),
  fs.writeFile(
    path.join(dest, 'node-versions.json'),
    JSON.stringify(nodeVersions.map(({ version }) => version))
  ),
  fs.writeFile(
    path.join(dest, 'node-release-schedule.json'),
    JSON.stringify(
      Object.fromEntries(
        Object.entries(nodeReleaseSchedule).map(([version, { start, end }]) => [
          version.slice(1),
          [start, end],
        ])
      )
    )
  ),
])
