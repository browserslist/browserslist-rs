import { loadBinding } from '@node-rs/helper'

const bindings = loadBinding(__dirname, 'browserslist-rs', 'browserslist-rs')

interface Opts {
  mobileToDesktop?: boolean
  ignoreUnknownVersions?: boolean
  config?: string
  env?: string
  path?: string
  throwOnMissing?: boolean
}

interface Execute {
  (query: string | string[], opts?: Opts): string[]
}

const execute: Execute = bindings.execute

export = execute
