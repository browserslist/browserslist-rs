const fs = require('fs/promises')
const path = require('path')
const browserslist = require('browserslist')
const e2c = require('electron-to-chromium/versions')
const nodeVersions = require('node-releases/data/processed/envs.json')
const nodeReleaseSchedule = require('node-releases/data/release-schedule/release-schedule.json')

Promise.all([
  fs.writeFile(
    path.join(process.env.OUT_DIR, 'caniuse-lite-browsers.json'),
    JSON.stringify(browserslist.data)
  ),
  fs.writeFile(
    path.join(process.env.OUT_DIR, 'caniuse-lite-usage.json'),
    JSON.stringify(
      Object.entries(browserslist.usage.global)
        .map(([v, usage]) => {
          const [name, version] = v.split(' ')
          return [name, version, usage]
        })
        .sort(([, , a], [, , b]) => b - a)
    )
  ),
  fs.writeFile(
    path.join(process.env.OUT_DIR, 'caniuse-lite-version-aliases.json'),
    JSON.stringify(browserslist.versionAliases)
  ),
  fs.writeFile(
    path.join(process.env.OUT_DIR, 'electron-to-chromium.json'),
    JSON.stringify(
      Object.entries(e2c).map(([k, v]) => [Number.parseFloat(k), v])
    )
  ),
  fs.writeFile(
    path.join(process.env.OUT_DIR, 'node-versions.json'),
    JSON.stringify(nodeVersions.map(({ version }) => version))
  ),
  fs.writeFile(
    path.join(process.env.OUT_DIR, 'node-release-schedule.json'),
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
