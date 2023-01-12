const readline = require('readline')

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false,
})

rl.on('line', (line) => {
  // console.error(line)
  try {
    new RegExp(line, 'u')
  } catch (e) {
    const message = e.message.replace(/[\n\\]/g, (c) =>
      c === '\\' ? '\\\\' : '\\n'
    )
    console.log(message)
    return
  }
  console.log('success')
})

rl.on('close', () => {
  process.exit(0)
})
