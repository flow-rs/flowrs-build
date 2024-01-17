import { createEditor as createExampleEditor } from './example'
import { createEditor as createFlowrsEditor } from './flowrs/editor'

const factory = {
  'flowbuilder': createFlowrsEditor,
  'example': createExampleEditor
}
// eslint-disable-next-line no-restricted-globals, no-undef
const query = typeof location !== 'undefined' && new URLSearchParams(location.search)
const name = (query && query.get('template') || 'flowbuilder') as keyof typeof factory

const createEditor = factory[name]

if (!createEditor) {
  throw new Error(`template with name ${name} not found`)
}

export {
  createEditor
}
