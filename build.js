const fs = require('fs/promises')
const path = require('path')
const browserslist = require('browserslist')
const e2c = require('electron-to-chromium/versions')

async function main() {
  await fs.writeFile(
    path.join(process.env.OUT_DIR, 'caniuse-lite-browsers.json'),
    JSON.stringify(browserslist.data)
  )

  await fs.writeFile(
    path.join(process.env.OUT_DIR, 'caniuse-lite-usage.json'),
    JSON.stringify(browserslist.usage.global)
  )

  await fs.writeFile(
    path.join(process.env.OUT_DIR, 'caniuse-lite-version-aliases.json'),
    JSON.stringify(browserslist.versionAliases)
  )

  await fs.writeFile(
    path.join(process.env.OUT_DIR, 'electron-to-chromium.json'),
    JSON.stringify(
      Object.entries(e2c).map(([k, v]) => [Number.parseFloat(k), v])
    )
  )
}

main()
