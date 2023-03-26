const readline = require('readline')

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false,
})

/** @type {RegExp|undefined} */
let regex

rl.on('line', (line) => {
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
