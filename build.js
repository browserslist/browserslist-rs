const fs = require('fs/promises')
const path = require('path')
const e2c = require('electron-to-chromium/versions')

async function main() {
  await fs.writeFile(
    path.join(process.env.OUT_DIR, 'electron-to-chromium.json'),
    JSON.stringify(
      Object.entries(e2c).map(([k, v]) => [Number.parseFloat(k), v])
    )
  )
}

main()
