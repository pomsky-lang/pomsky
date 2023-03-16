const process = require('node:process')

/** @type {RegExp|undefined} */
let regex

process.stdin.on('data', (data) => {
  let line = data.toString()
  if (line.endsWith('\r\n')) {
    line = line.slice(0, line.length - 2)
  } else if (line.endsWith('\n')) {
    line = line.slice(0, line.length - 1)
  }

  if (regex === undefined) {
    try {
      regex = new RegExp(line, 'u')
      console.log('success')
    } catch (e) {
      console.log(substituteLf(e.message))
    }
  } else if (line.startsWith('TEST:')) {
    const test = line.slice(5)

    if (regex.test(test)) {
      console.log('test good')
    } else {
      console.log(
        substituteLf(`Regex '${regex.source}' does not match '${test}'`)
      )
      regex = undefined
    }
  } else {
    regex = undefined
  }
})

function substituteLf(s = '') {
  return s.replace(/[\n\\]/g, (c) => (c === '\\' ? '\\\\' : '\\n'))
}
