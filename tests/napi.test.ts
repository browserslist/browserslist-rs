import * as assert from 'node:assert'
import browserslist from 'browserslist'
// @ts-ignore
import { execute } from '../browserslist-rs.node'

describe('should match `browserslist` library', () => {
  it('current node', () => {
    assert.deepStrictEqual(
      execute('current node'),
      browserslist('current node')
    )
  })

  it('defaults', () => {
    assert.deepStrictEqual(execute('defaults'), browserslist('defaults'))
  })

  it('queries as array', () => {
    const queries = ['last 1 firefox version', 'last 2 firefox version']
    assert.deepStrictEqual(execute(queries), browserslist(queries))
  })

  it('optional properties', () => {
    assert.deepStrictEqual(
      execute('node 3', { ignoreUnknownVersions: true }),
      browserslist('node 3', { ignoreUnknownVersions: true })
    )
  })
})
