import { loadBinding } from '@node-rs/helper'

const bindings = loadBinding(__dirname, 'browserslist-rs', 'browserslist-rs')

interface Opts {
  mobileToDesktop?: boolean
  ignoreUnknownVersions?: boolean
}

interface Execute {
  (query: string | string[], opts?: Opts): string[]
}

const execute: Execute = bindings.execute

export = execute
